use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;

use crate::state::{UserProfile, TipRecord};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String, // Initial admin address
}

#[cw_serde]
pub enum ExecuteMsg {
    // User profile management
    RegisterProfile {
        username: String,
        name: String,
        bio: Option<String>,
        profile_picture: Option<String>,
        banner_image: Option<String>,
        twitter: Option<String>,
        website: Option<String>,
    },
    
    UpdateProfile {
        username: String,
        name: Option<String>,
        bio: Option<String>,
        profile_picture: Option<String>,
        banner_image: Option<String>,
        twitter: Option<String>,
        website: Option<String>,
    },
    
    // Tip recording functionality
    RecordTip {
        to_username: String,
        amount: String,
        message: Option<String>,
    },
    
    // Admin management
    AddAdmin {
        admin: String,
    },
    
    RemoveAdmin {
        admin: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // User profile queries
    #[returns(ProfileResponse)]
    GetProfile { 
        username: String 
    },
    
    #[returns(ProfileResponse)]
    GetProfileByWallet { 
        wallet: String 
    },
    
    #[returns(ProfilesResponse)]
    ListProfiles { 
        limit: Option<u32>,
        start_after: Option<String>,
    },
    
    // Tip queries
    #[returns(TipsResponse)]
    GetTipsSent { 
        username: String,
        limit: Option<u32>,
        start_after: Option<String>,
    },
    
    #[returns(TipsResponse)]
    GetTipsReceived { 
        username: String,
        limit: Option<u32>,
        start_after: Option<String>,
    },
    
    #[returns(TipDetailResponse)]
    GetTipDetail { 
        from_username: String,
        to_username: String,
        timestamp: Timestamp,
    },
    
    // Statistics
    #[returns(StatsResponse)]
    GetUserStats {
        username: String,
    },
    
    // Admin check
    #[returns(AdminResponse)]
    IsAdmin { 
        address: String,
    },
    
    // Utility
    #[returns(UsernameAvailableResponse)]
    IsUsernameAvailable {
        username: String,
    },
}

// Response types
#[cw_serde]
pub struct ProfileResponse {
    pub profile: Option<UserProfile>,
}

#[cw_serde]
pub struct ProfilesResponse {
    pub profiles: Vec<UserProfile>,
}

#[cw_serde]
pub struct TipsResponse {
    pub tips: Vec<TipRecord>,
}

#[cw_serde]
pub struct TipDetailResponse {
    pub tip: Option<TipRecord>,
}

#[cw_serde]
pub struct StatsResponse {
    pub total_tips_sent: u64,
    pub total_tips_received: u64,
    pub total_amount_sent: String,
    pub total_amount_received: String,
}

#[cw_serde]
pub struct AdminResponse {
    pub is_admin: bool,
}

#[cw_serde]
pub struct UsernameAvailableResponse {
    pub is_available: bool,
}
