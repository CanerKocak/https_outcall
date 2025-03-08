use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use reqwest::Client;
use log::{info, error};
use std::env;

use crate::api::handlers::ApiResponse;
use crate::websocket;

// Claude API configuration
const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-3-sonnet-20240229";

// Structure for Claude API request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeRequest {
    pub canister_id: String,
    pub request_id: String,
    pub system: Option<String>,
    pub messages: Vec<ClaudeMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

// Structure for Claude API response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ClaudeContent>,
    pub model: String,
    pub role: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<ClaudeUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// Modified cache entry with serializable response data
#[derive(Clone)]
struct CacheEntry {
    response_data: ClaudeResponse,
    expires_at: DateTime<Utc>,
}

// Thread-safe cache for responses
lazy_static::lazy_static! {
    static ref CLAUDE_RESPONSE_CACHE: Arc<Mutex<HashMap<String, CacheEntry>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

// Handle Claude API requests with deduplication
pub async fn handle_claude_request(
    req: HttpRequest, 
    data: web::Json<ClaudeRequest>
) -> HttpResponse {
    // Extract API key from headers for authentication
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    // Validate API key (replace with your actual validation logic)
    if !validate_api_key(api_key) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid API key"));
    }
    
    // Extract request details
    let canister_id = data.canister_id.clone();
    let request_id = data.request_id.clone();
    
    // Create a unique cache key for this request
    let cache_key = format!("{}:{}", canister_id, request_id);
    
    // Check if we've seen this exact request before
    let cached_response = {
        let cache = CLAUDE_RESPONSE_CACHE.lock().unwrap();
        cache.get(&cache_key).map(|entry| entry.response_data.clone())
    };
    
    // If we have a cached response, return it
    if let Some(response) = cached_response {
        info!("Returning cached response for duplicate request: {}", cache_key);
        return HttpResponse::Ok().json(ApiResponse::success(response, "Cached response"));
    }
    
    // First time seeing this request - process it
    info!("Processing new Claude API request from canister: {}", canister_id);
    
    // Get Claude API key from environment variable
    let claude_api_key = match env::var("CLAUDE_API_KEY") {
        Ok(key) => key,
        Err(e) => {
            error!("Failed to get CLAUDE_API_KEY from environment: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("Claude API key not configured")
            );
        }
    };
    
    // Prepare the request to Claude API
    let client = Client::new();
    
    // Prepare the request body for Claude API
    let claude_api_request = serde_json::json!({
        "model": CLAUDE_MODEL,
        "system": data.system.clone().unwrap_or_else(|| "You are Claude, a helpful AI assistant.".to_string()),
        "messages": data.messages,
        "max_tokens": data.max_tokens.unwrap_or(1000),
        "temperature": data.temperature.unwrap_or(0.7),
    });
    
    // Make the request to Claude API
    let response = match client.post(CLAUDE_API_URL)
        .header("x-api-key", claude_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&claude_api_request)
        .send()
        .await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to send request to Claude API: {}", e);
                return HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error(&format!("Failed to send request to Claude API: {}", e))
                );
            }
        };
    
    // Check if the request was successful
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Claude API returned error: {}", error_text);
        return HttpResponse::BadGateway().json(
            ApiResponse::<()>::error(&format!("Claude API returned error: {}", error_text))
        );
    }
    
    // Parse the response
    let claude_response: ClaudeResponse = match response.json().await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to parse Claude API response: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error(&format!("Failed to parse Claude API response: {}", e))
            );
        }
    };
    
    // Cache the response (expires after 30 minutes)
    {
        let mut cache = CLAUDE_RESPONSE_CACHE.lock().unwrap();
        cache.insert(
            cache_key, 
            CacheEntry {
                response_data: claude_response.clone(),
                expires_at: Utc::now() + Duration::minutes(30),
            }
        );
    }
    
    // Broadcast the response to WebSocket clients
    websocket::broadcast_notification(
        "claude_response", 
        serde_json::json!({
            "canister_id": canister_id,
            "request_id": request_id,
            "response": claude_response
        })
    );
    
    // Clean up expired cache entries
    clean_expired_cache_entries();
    
    // Return the response
    HttpResponse::Ok().json(ApiResponse::success(claude_response, "Claude API response"))
}

// Validate API key
fn validate_api_key(_api_key: &str) -> bool {
    // For now, we'll accept any key as requested by the user
    true
}

// Clean up expired cache entries
fn clean_expired_cache_entries() {
    let now = Utc::now();
    let mut cache = CLAUDE_RESPONSE_CACHE.lock().unwrap();
    
    // Remove entries that have expired
    cache.retain(|_, entry| entry.expires_at > now);
}

// Periodically clean cache (call this from your main function)
pub fn start_cache_cleanup_task() {
    std::thread::spawn(|| {
        loop {
            // Sleep for 5 minutes
            std::thread::sleep(std::time::Duration::from_secs(300));
            clean_expired_cache_entries();
        }
    });
} 