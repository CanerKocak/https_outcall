use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::websocket;

// Structure for canister notifications
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationData {
    pub event: String,
    pub miner_id: String,
    pub timestamp: u64,
    pub data: Value,
}

// Modified cache entry with serializable response data instead of HttpResponse
#[derive(Clone)]
struct CacheEntry {
    response_data: Value,
    expires_at: DateTime<Utc>,
}

// Thread-safe cache for responses
lazy_static::lazy_static! {
    static ref RESPONSE_CACHE: Arc<Mutex<HashMap<String, CacheEntry>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

// Handle canister notifications with deduplication
pub async fn handle_canister_notification(
    req: HttpRequest, 
    data: web::Json<NotificationData>
) -> HttpResponse {
    // Extract API key from headers for authentication
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    // Validate API key (replace with your actual validation logic)
    if !validate_api_key(api_key) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid API key"
        }));
    }
    
    // Extract notification details
    let canister_id = data.miner_id.clone();
    let timestamp = data.timestamp;
    let event_type = data.event.clone();
    
    // Create a unique cache key for this notification
    let cache_key = format!("{}:{}:{}", canister_id, timestamp, event_type);
    
    // Check if we've seen this exact notification before
    let cached_response_data = {
        let cache = RESPONSE_CACHE.lock().unwrap();
        cache.get(&cache_key).map(|entry| entry.response_data.clone())
    };
    
    // If we have a cached response, return it
    if let Some(response_data) = cached_response_data {
        log::info!("Returning cached response for duplicate notification: {}", cache_key);
        return HttpResponse::Ok().json(response_data);
    }
    
    // First time seeing this notification - process it
    log::info!("Processing new notification: {} from {}", event_type, canister_id);
    
    // Create a deterministic response ID using a simple hash
    // Replaced MD5 with a simpler hash method
    let response_id = format!("{:x}", canister_id.len() + timestamp as usize + event_type.len());
    
    // Prepare the response
    let response_body = serde_json::json!({
        "status": "processed", 
        "id": response_id
    });
    
    // Cache the response (expires after 5 minutes)
    {
        let mut cache = RESPONSE_CACHE.lock().unwrap();
        cache.insert(
            cache_key, 
            CacheEntry {
                response_data: response_body.clone(),
                expires_at: Utc::now() + Duration::minutes(5),
            }
        );
    }
    
    // Process the notification based on event type
    match event_type.as_str() {
        "token_connected" => {
            // Handle token connection event
            log::info!("Miner {} connected to token", canister_id);
            websocket::broadcast_notification(&event_type, data.data.clone());
        },
        "mining_started" => {
            // Handle mining started event
            log::info!("Miner {} started mining", canister_id);
            websocket::broadcast_notification(&event_type, data.data.clone());
        },
        "solution_found" => {
            // Handle solution found event
            log::info!("Miner {} found solution", canister_id);
            websocket::broadcast_notification(&event_type, data.data.clone());
        },
        _ => {
            log::warn!("Unknown event type: {}", event_type);
            // Still broadcast unknown events to WebSocket clients
            websocket::broadcast_notification(&event_type, data.data.clone());
        }
    }
    
    // Clean up expired cache entries
    clean_expired_cache_entries();
    
    // Return the response
    HttpResponse::Ok().json(response_body)
}

// Validate API key
fn validate_api_key(_api_key: &str) -> bool {
    // For now, we'll accept any key as requested by the user
    true
}

// Clean up expired cache entries
fn clean_expired_cache_entries() {
    let now = Utc::now();
    let mut cache = RESPONSE_CACHE.lock().unwrap();
    
    // Remove entries that have expired
    cache.retain(|_, entry| entry.expires_at > now);
}

// Periodically clean cache (call this from your main function)
pub fn start_cache_cleanup_task() {
    std::thread::spawn(|| {
        loop {
            // Sleep for 1 minute
            std::thread::sleep(std::time::Duration::from_secs(60));
            clean_expired_cache_entries();
        }
    });
} 