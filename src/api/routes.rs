use actix_web::web;
use crate::api::handlers::{canister, token, miner, system};

/// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Canister routes
    cfg.service(
        web::scope("/canisters")
            .route("", web::get().to(canister::get_all_canisters))
            .route("", web::post().to(canister::register_canister))
            .route("/{canister_id}", web::get().to(canister::get_canister))
            .route("/{canister_id}", web::put().to(canister::update_canister))
            .route("/{canister_id}", web::delete().to(canister::delete_canister))
    );
    
    // Token routes
    cfg.service(
        web::scope("/tokens")
            .route("", web::get().to(token::get_all_tokens))
            .route("/{canister_id}", web::get().to(token::get_token))
            .route("/{canister_id}", web::delete().to(token::delete_token))
    );
    
    // Miner routes
    cfg.service(
        web::scope("/miners")
            .route("", web::get().to(miner::get_all_miners))
            .route("/{canister_id}", web::get().to(miner::get_miner))
            .route("/{canister_id}", web::delete().to(miner::delete_miner))
            .route("/{canister_id}/stats", web::get().to(miner::get_miner_stats))
            .route("/by-token/{token_canister_id}", web::get().to(miner::get_miners_by_token))
            .route("/stats", web::get().to(miner::get_all_mining_stats))
    );
    
    // Module hash routes
    cfg.service(
        web::scope("/module-hashes")
            .route("", web::get().to(canister::get_all_module_hashes))
            .route("", web::post().to(canister::set_module_hash))
    );
    
    // System routes
    cfg.service(
        web::scope("/system")
            .route("/status", web::get().to(system::get_system_status))
            .route("/refresh", web::post().to(system::trigger_refresh))
            .route("/interfaces", web::get().to(system::generate_interfaces))
    );
} 