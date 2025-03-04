use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ArchiveOptions {
    pub num_blocks_to_archive: u64,
    pub max_transactions_per_response: Option<u64>,
    pub trigger_threshold: u64,
    pub more_controller_ids: Option<Vec<Principal>>,
    pub max_message_size_bytes: Option<u64>,
    pub cycles_for_archive_creation: Option<u64>,
    pub node_max_memory_size_bytes: Option<u64>,
    pub controller_id: Principal,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SocialLink {
    pub url: String,
    pub platform: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenInfo {
    pub decimals: u8,
    pub ticker: String,
    pub transfer_fee: u64,
    pub logo: Option<String>,
    pub name: String,
    pub ledger_id: Option<Principal>,
    pub archive_options: Option<ArchiveOptions>,
    pub total_supply: u64,
    pub social_links: Option<Vec<SocialLink>>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Result {
    Ok(TokenInfo),
    Err(String),
}

// Define the interface for the token canister
pub fn token_interface() -> candid::IDLValue {
    candid::IDLValue::Text(r#"
    type ArchiveOptions = record {
      num_blocks_to_archive : nat64;
      max_transactions_per_response : opt nat64;
      trigger_threshold : nat64;
      more_controller_ids : opt vec principal;
      max_message_size_bytes : opt nat64;
      cycles_for_archive_creation : opt nat64;
      node_max_memory_size_bytes : opt nat64;
      controller_id : principal;
    };
    type Result = variant {
      Ok : TokenInfo;
      Err : text;
    };
    type SocialLink = record {
      url : text;
      platform : text;
    };
    type TokenInfo = record {
      decimals : nat8;
      ticker : text;
      transfer_fee : nat64;
      logo : opt text;
      name : text;
      ledger_id : opt principal;
      archive_options : opt ArchiveOptions;
      total_supply : nat64;
      social_links : opt vec SocialLink;
    };
    service : {
      get_info : () -> (Result) query;
    }
    "#.to_string())
} 