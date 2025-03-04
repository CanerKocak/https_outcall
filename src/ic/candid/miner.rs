use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum MinerType {
    Premium,
    Lite,
    Normal,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MinerInfo {
    pub speed_percentage: u8,
    pub current_token: Option<Principal>,
    pub chunks_per_refresh: u64,
    pub miner_type: MinerType,
    pub is_mining: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MiningStats {
    pub total_hashes: u64,
    pub blocks_mined: u64,
    pub chunks_since_refresh: u64,
    pub total_rewards: u64,
    pub last_hash_rate: f64,
    pub start_time: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Result {
    Ok(MinerInfo),
    Err(String),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum StatsResult {
    Ok(Option<MiningStats>),
    Err(String),
}

// Define the interface for the miner canister
pub fn miner_interface() -> candid::IDLValue {
    candid::IDLValue::Text(r#"
    type MinerInfo = record {
      speed_percentage : nat8;
      current_token : opt principal;
      chunks_per_refresh : nat64;
      miner_type : MinerType;
      is_mining : bool;
    };
    type MinerType = variant { Premium; Lite; Normal };
    type MiningStats = record {
      total_hashes : nat64;
      blocks_mined : nat64;
      chunks_since_refresh : nat64;
      total_rewards : nat64;
      last_hash_rate : float64;
      start_time : nat64;
    };
    type Result = variant { Ok : MinerInfo; Err : text };
    service : {
      get_info : () -> (Result) query;
      get_mining_stats : () -> (opt MiningStats) query;
    }
    "#.to_string())
} 