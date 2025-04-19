use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// User profile structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserProfile {
    pub username: String,           // Unique username (primary identifier)
    pub name: String,               // Display name
    pub bio: Option<String>,        // User biography
    pub profile_picture: Option<String>,  // URL to profile picture
    pub banner_image: Option<String>,     // URL to banner image
    pub twitter: Option<String>,    // Twitter handle
    pub website: Option<String>,    // Personal website
    pub wallet_address: Addr,       // User's wallet address
    pub created_at: Timestamp,      // When the profile was created
    pub updated_at: Timestamp,      // When the profile was last updated
}

// Tip record structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TipRecord {
    pub from_username: String,      // Username of sender
    pub to_username: String,        // Username of recipient
    pub amount: String,             // Amount as string (e.g., "5uxion")
    pub message: Option<String>,    // Optional message with the tip
    pub timestamp: Timestamp,       // When the tip was recorded
}

// Store user profiles by username
pub const USER_PROFILES: Map<&str, UserProfile> = Map::new("user_profiles");

// Store wallet addresses to usernames mapping (for quick lookup)
pub const WALLET_TO_USERNAME: Map<&Addr, String> = Map::new("wallet_to_username");

// Store tip records - using a composite string key instead of timestamp directly
// Key format: "from_username:to_username:timestamp.nanos()"
pub const TIP_RECORDS: Map<&str, TipRecord> = Map::new("tip_records");

// Store tips sent by a user
pub const TIPS_SENT: Map<&str, Vec<(String, String)>> = Map::new("tips_sent");

// Store tips received by a user
pub const TIPS_RECEIVED: Map<&str, Vec<(String, String)>> = Map::new("tips_received");

// Store global contract admins
pub const CONTRACT_ADMINS: Map<&Addr, bool> = Map::new("contract_admins");
