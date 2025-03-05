use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// WebSocket session data
pub struct WebSocketSession {
    // Unique session id
    pub id: String,
    // Last ping time
    pub last_heartbeat: Instant,
    // Server address
    pub server_addr: Addr<WebSocketServer>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat process
        self.heartbeat(ctx);

        // Register self with server
        let addr = ctx.address();
        self.server_addr.do_send(Connect {
            id: self.id.clone(),
            addr,
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        // Notify server about disconnect
        self.server_addr.do_send(Disconnect {
            id: self.id.clone(),
        });
        actix::Running::Stop
    }
}

// Handler for WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Handle text messages from client if needed
                log::debug!("Received message: {}", text);
                // For now, we just echo back
                ctx.text(text);
            }
            Ok(ws::Message::Binary(bin)) => {
                // Handle binary messages if needed
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// Add handler for BroadcastMessage to WebSocketSession
impl Handler<BroadcastMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl WebSocketSession {
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeat
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                log::warn!("WebSocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

// WebSocket server actor
pub struct WebSocketServer {
    // Map of session id to session address
    sessions: HashMap<String, Addr<WebSocketSession>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        WebSocketServer {
            sessions: HashMap::new(),
        }
    }

    // Send message to all sessions
    fn broadcast(&self, message: &str) {
        for (_, addr) in &self.sessions {
            addr.do_send(BroadcastMessage(message.to_owned()));
        }
    }
}

impl Actor for WebSocketServer {
    type Context = actix::Context<Self>;
}

// Message for WebSocket server communications
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub addr: Addr<WebSocketSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastNotification {
    pub event: String,
    pub data: Value,
}

// Handler for Connect message
impl Handler<Connect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut actix::Context<Self>) {
        log::info!("WebSocket client connected: {}", msg.id);
        self.sessions.insert(msg.id, msg.addr);
    }
}

// Handler for Disconnect message
impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut actix::Context<Self>) {
        log::info!("WebSocket client disconnected: {}", msg.id);
        self.sessions.remove(&msg.id);
    }
}

// Handler for Broadcast message
impl Handler<BroadcastMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut actix::Context<Self>) {
        self.broadcast(&msg.0);
    }
}

// Handler for BroadcastNotification message
impl Handler<BroadcastNotification> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastNotification, _: &mut actix::Context<Self>) {
        let notification = serde_json::json!({
            "event": msg.event,
            "data": msg.data,
            "timestamp": chrono::Utc::now().timestamp_millis()
        });
        
        if let Ok(json_str) = serde_json::to_string(&notification) {
            self.broadcast(&json_str);
        } else {
            log::error!("Failed to serialize notification");
        }
    }
}

// Global WebSocket server instance
lazy_static::lazy_static! {
    pub static ref WS_SERVER: Arc<Mutex<Option<Addr<WebSocketServer>>>> = Arc::new(Mutex::new(None));
}

// Initialize the WebSocket server
pub fn init_websocket_server() -> Addr<WebSocketServer> {
    let server = WebSocketServer::new().start();
    
    // Store server address in global variable
    let mut ws_server = WS_SERVER.lock().unwrap();
    *ws_server = Some(server.clone());
    
    server
}

// Broadcast a notification to all connected WebSocket clients
pub fn broadcast_notification(event: &str, data: Value) {
    if let Some(server) = WS_SERVER.lock().unwrap().as_ref() {
        server.do_send(BroadcastNotification {
            event: event.to_string(),
            data,
        });
    } else {
        log::warn!("WebSocket server not initialized, can't broadcast notification");
    }
} 