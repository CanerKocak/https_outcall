use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;
use uuid::Uuid;

use crate::websocket::{WebSocketSession, WebSocketServer};

// WebSocket connection handler
pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<actix::Addr<WebSocketServer>>,
) -> Result<HttpResponse, Error> {
    // Generate a unique session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Log connection attempt
    log::info!("WebSocket connection attempt from: {}", 
        req.connection_info().peer_addr().unwrap_or("unknown"));
    
    // Create a new WebSocket session
    let ws_session = WebSocketSession {
        id: session_id,
        last_heartbeat: Instant::now(),
        server_addr: srv.get_ref().clone(),
    };
    
    // Start the WebSocket session
    let resp = ws::start(ws_session, &req, stream)?;
    Ok(resp)
}

// Simple status endpoint to check if WebSocket server is running
pub async fn websocket_status() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
} 