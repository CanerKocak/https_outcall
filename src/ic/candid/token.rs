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
pub struct TokenAllInfo {
    pub name: String,
    pub ticker: String,
    pub total_supply: u64,
    pub ledger_id: Option<Principal>,
    pub logo: Option<String>,
    pub decimals: u8,
    pub transfer_fee: u64,
    pub social_links: Option<Vec<SocialLink>>,
    pub average_block_time: Option<f64>,
    pub formatted_block_time: Option<String>,
    pub block_time_rating: Option<String>,
    pub circulating_supply: u64,
    pub mining_progress_percentage: String,
    pub current_block_reward: u64,
    pub formatted_block_reward: String,
    pub principal: Principal,
    pub current_block_height: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Result {
    Ok(TokenInfo),
    Err(String),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum AllInfoResult {
    Ok(TokenAllInfo),
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
    type TokenAllInfo = record {
      name : text;
      ticker : text;
      total_supply : nat64;
      ledger_id : opt principal;
      logo : opt text;
      decimals : nat8;
      transfer_fee : nat64;
      social_links : opt vec SocialLink;
      average_block_time : opt float64;
      formatted_block_time : opt text;
      block_time_rating : opt text;
      circulating_supply : nat64;
      mining_progress_percentage : text;
      current_block_reward : nat64;
      formatted_block_reward : text;
      principal : principal;
      current_block_height : nat64;
    };
    type AllInfoResult = variant {
      Ok : TokenAllInfo;
      Err : text;
    };
    service : {
      get_all_info : () -> (AllInfoResult) query;
    }
    "#.to_string())
}
