use actix_web::web;
use crate::api::handlers::{canister, token, miner, system, admin, claude};

/// Configure the API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Canister routes
    cfg.service(
        web::scope("/canisters")
            .route("", web::get().to(canister::get_all_canisters))
            .route("", web::post().to(canister::register_canister))
            .route("/type/{canister_type}", web::get().to(canister::get_canisters_by_type))
            .route("/{canister_id}", web::get().to(canister::get_canister))
            .route("/{canister_id}", web::put().to(canister::update_canister))
    );
    
    // Token routes
    cfg.service(
        web::scope("/tokens")
            .route("", web::get().to(token::get_all_tokens))
            .route("/{canister_id}", web::get().to(token::get_token))
    );
    
    // Miner routes
    cfg.service(
        web::scope("/miners")
            .route("", web::get().to(miner::get_all_miners))
            .route("/{canister_id}", web::get().to(miner::get_miner))
            .route("/{canister_id}/stats", web::get().to(miner::get_miner_stats))
            .route("/by-token/{token_canister_id}", web::get().to(miner::get_miners_by_token))
            .route("/stats", web::get().to(miner::get_all_mining_stats))
    );
    
    // Public module hash routes
    cfg.service(
        web::scope("/module-hashes")
            .route("", web::get().to(canister::get_all_verified_module_hashes))
    );
    
    // Admin module hash routes - using admin handlers with built-in authentication
    cfg.route("/admin/module-hashes", web::post().to(admin::add_verified_module_hash));
    cfg.route("/admin/module-hashes/{hash}", web::delete().to(admin::remove_verified_module_hash));
    
    // Admin canister management routes
    cfg.route("/admin/canisters/{canister_id}", web::delete().to(admin::delete_canister));
    cfg.route("/admin/tokens/{canister_id}", web::delete().to(admin::delete_token));
    cfg.route("/admin/miners/{canister_id}", web::delete().to(admin::delete_miner));
    
    // Admin module hash management routes
    cfg.route("/admin/canisters/module-hashes", web::get().to(admin::get_all_module_hashes));
    cfg.route("/admin/canisters/{canister_id}/module-hash", web::put().to(admin::set_module_hash));
    
    // Claude API route
    cfg.service(
        web::scope("/claude")
            .route("", web::post().to(claude::handle_claude_request))
    );
    
    // System routes
    cfg.service(
        web::scope("/system")
            .route("/status", web::get().to(system::get_system_status))
            .route("/refresh", web::post().to(system::trigger_refresh))
            .route("/interfaces", web::get().to(system::generate_interfaces))
            .route("/statistics", web::get().to(system::get_statistics))
    );
} 
