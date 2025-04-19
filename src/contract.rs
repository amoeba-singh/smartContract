#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr,
};
use cw2::set_contract_version;
use regex::Regex;

use crate::error::ContractError;
use crate::helpers::addr_validate;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, ProfileResponse, ProfilesResponse, TipsResponse,
    TipDetailResponse, StatsResponse, AdminResponse, UsernameAvailableResponse,
};
use crate::state::{
    UserProfile, TipRecord, USER_PROFILES, WALLET_TO_USERNAME, TIP_RECORDS, TIPS_SENT,
    TIPS_RECEIVED, CONTRACT_ADMINS,
};

// Contract name and version info for migration
const CONTRACT_NAME: &str = "crates.io:tipping-profiles";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    // Set the initial admin
    let admin_addr = addr_validate(deps.api, &msg.admin)?;
    CONTRACT_ADMINS.save(deps.storage, &admin_addr, &true)?;
    
    // Also set the contract creator as an admin
    if admin_addr != info.sender {
        CONTRACT_ADMINS.save(deps.storage, &info.sender, &true)?;
    }
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", msg.admin)
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterProfile {
            username,
            name,
            bio,
            profile_picture,
            banner_image,
            twitter,
            website,
        } => execute_register_profile(
            deps, 
            env, 
            info, 
            username, 
            name, 
            bio, 
            profile_picture, 
            banner_image, 
            twitter, 
            website,
        ),
        
        ExecuteMsg::UpdateProfile {
            username,
            name,
            bio,
            profile_picture,
            banner_image,
            twitter,
            website,
        } => execute_update_profile(
            deps, 
            env, 
            info, 
            username, 
            name, 
            bio, 
            profile_picture, 
            banner_image, 
            twitter, 
            website,
        ),
        
        ExecuteMsg::RecordTip {
            to_username,
            amount,
            message,
        } => execute_record_tip(deps, env, info, to_username, amount, message),
        
        ExecuteMsg::AddAdmin { admin } => execute_add_admin(deps, info, admin),
        
        ExecuteMsg::RemoveAdmin { admin } => execute_remove_admin(deps, info, admin),
    }
}

fn execute_register_profile(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    username: String,
    name: String,
    bio: Option<String>,
    profile_picture: Option<String>,
    banner_image: Option<String>,
    twitter: Option<String>,
    website: Option<String>,
) -> Result<Response, ContractError> {
    // Validate username format
    validate_username(&username)?;
    
    // Check if username is already taken
    if USER_PROFILES.has(deps.storage, &username) {
        return Err(ContractError::UsernameExists { username });
    }
    
    // Check if the wallet is already registered
    if WALLET_TO_USERNAME.has(deps.storage, &info.sender) {
        let existing_username = WALLET_TO_USERNAME.load(deps.storage, &info.sender)?;
        return Err(ContractError::WalletAlreadyRegistered { username: existing_username });
    }
    
    // Create and save the new profile
    let profile = UserProfile {
        username: username.clone(),
        name,
        bio,
        profile_picture,
        banner_image,
        twitter,
        website,
        wallet_address: info.sender.clone(),
        created_at: env.block.time,
        updated_at: env.block.time,
    };
    
    USER_PROFILES.save(deps.storage, &username, &profile)?;
    WALLET_TO_USERNAME.save(deps.storage, &info.sender, &username)?;
    
    Ok(Response::new()
        .add_attribute("method", "register_profile")
        .add_attribute("username", username)
        .add_attribute("wallet", info.sender))
}

fn execute_update_profile(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    username: String,
    name: Option<String>,
    bio: Option<String>,
    profile_picture: Option<String>,
    banner_image: Option<String>,
    twitter: Option<String>,
    website: Option<String>,
) -> Result<Response, ContractError> {
    // Check if profile exists
    if !USER_PROFILES.has(deps.storage, &username) {
        return Err(ContractError::UsernameNotFound { username });
    }
    
    // Load the profile
    let mut profile = USER_PROFILES.load(deps.storage, &username)?;
    
    // Check authorization - only the owner or an admin can update the profile
    if profile.wallet_address != info.sender && !is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }
    
    // Update fields if provided
    if let Some(new_name) = name {
        profile.name = new_name;
    }
    
    if let Some(new_bio) = bio {
        profile.bio = Some(new_bio);
    }
    
    if let Some(new_picture) = profile_picture {
        profile.profile_picture = Some(new_picture);
    }
    
    if let Some(new_banner) = banner_image {
        profile.banner_image = Some(new_banner);
    }
    
    if let Some(new_twitter) = twitter {
        profile.twitter = Some(new_twitter);
    }
    
    if let Some(new_website) = website {
        profile.website = Some(new_website);
    }
    
    profile.updated_at = env.block.time;
    
    // Save updated profile
    USER_PROFILES.save(deps.storage, &username, &profile)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_profile")
        .add_attribute("username", username))
}

fn execute_record_tip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_username: String,
    amount: String,
    message: Option<String>,
) -> Result<Response, ContractError> {
    // Get sender's username
    let from_username = match WALLET_TO_USERNAME.may_load(deps.storage, &info.sender)? {
        Some(username) => username,
        None => return Err(ContractError::CustomError {
            message: "Sender wallet address not registered with a profile".to_string(),
        }),
    };
    
    // Check that sender is not tipping themselves
    if from_username == to_username {
        return Err(ContractError::SelfTipping {});
    }
    
    // Check if recipient exists
    if !USER_PROFILES.has(deps.storage, &to_username) {
        return Err(ContractError::UsernameNotFound { username: to_username });
    }
    
    // Create tip record
    let timestamp = env.block.time;
    let tip = TipRecord {
        from_username: from_username.clone(),
        to_username: to_username.clone(),
        amount,
        message,
        timestamp,
    };
    
    // Create a composite key
    let tip_key = format!("{}:{}:{}", from_username, to_username, timestamp.nanos());
    
    // Save tip record
    TIP_RECORDS.save(deps.storage, &tip_key, &tip)?;
    
    // Update tips sent for sender
    let mut sender_tips = TIPS_SENT.may_load(deps.storage, &from_username)?.unwrap_or_default();
    sender_tips.push((to_username.clone(), tip_key.clone()));
    TIPS_SENT.save(deps.storage, &from_username, &sender_tips)?;
    
    // Update tips received for recipient
    let mut recipient_tips = TIPS_RECEIVED.may_load(deps.storage, &to_username)?.unwrap_or_default();
    recipient_tips.push((from_username.clone(), tip_key));
    TIPS_RECEIVED.save(deps.storage, &to_username, &recipient_tips)?;
    
    Ok(Response::new()
        .add_attribute("method", "record_tip")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username)
        .add_attribute("timestamp", timestamp.to_string()))
}

fn execute_add_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    if !is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }
    
    // Validate and add new admin
    let admin_addr = addr_validate(deps.api, &admin)?;
    CONTRACT_ADMINS.save(deps.storage, &admin_addr, &true)?;
    
    Ok(Response::new()
        .add_attribute("method", "add_admin")
        .add_attribute("admin", admin))
}

fn execute_remove_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    if !is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }
    
    // Validate admin address
    let admin_addr = addr_validate(deps.api, &admin)?;
    
    // Prevent removing the last admin
    let is_last_admin = CONTRACT_ADMINS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .count() <= 1;
    
    if is_last_admin {
        return Err(ContractError::CustomError {
            message: "Cannot remove the last admin".to_string(),
        });
    }
    
    CONTRACT_ADMINS.remove(deps.storage, &admin_addr);
    
    Ok(Response::new()
        .add_attribute("method", "remove_admin")
        .add_attribute("admin", admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProfile { username } => to_json_binary(&query_profile(deps, username)?),
        QueryMsg::GetProfileByWallet { wallet } => to_json_binary(&query_profile_by_wallet(deps, wallet)?),
        QueryMsg::ListProfiles { limit, start_after } => to_json_binary(&query_list_profiles(deps, limit, start_after)?),
        QueryMsg::GetTipsSent { username, limit, start_after } => to_json_binary(&query_tips_sent(deps, username, limit, start_after)?),
        QueryMsg::GetTipsReceived { username, limit, start_after } => to_json_binary(&query_tips_received(deps, username, limit, start_after)?),
        QueryMsg::GetTipDetail { from_username, to_username, timestamp } => to_json_binary(&query_tip_detail(deps, from_username, to_username, timestamp)?),
        QueryMsg::GetUserStats { username } => to_json_binary(&query_user_stats(deps, username)?),
        QueryMsg::IsAdmin { address } => to_json_binary(&query_is_admin(deps, address)?),
        QueryMsg::IsUsernameAvailable { username } => to_json_binary(&query_is_username_available(deps, username)?),
    }
}

fn query_profile(deps: Deps, username: String) -> StdResult<ProfileResponse> {
    let profile = USER_PROFILES.may_load(deps.storage, &username)?;
    Ok(ProfileResponse { profile })
}

fn query_profile_by_wallet(deps: Deps, wallet: String) -> StdResult<ProfileResponse> {
    let addr = deps.api.addr_validate(&wallet)?;
    
    // Get username associated with this wallet
    let username = match WALLET_TO_USERNAME.may_load(deps.storage, &addr)? {
        Some(username) => username,
        None => return Ok(ProfileResponse { profile: None }),
    };
    
    let profile = USER_PROFILES.may_load(deps.storage, &username)?;
    Ok(ProfileResponse { profile })
}

fn query_list_profiles(
    deps: Deps,
    limit: Option<u32>,
    start_after: Option<String>,
) -> StdResult<ProfilesResponse> {
    let limit = limit.unwrap_or(30) as usize;
    
    // Get all profiles
    let all_profiles: StdResult<Vec<UserProfile>> = USER_PROFILES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, profile) = item?;
            Ok(profile)
        })
        .collect();
    
    // Filter and limit profiles
    let profiles = all_profiles?
        .into_iter()
        .filter(|p| if let Some(ref start) = start_after {
            p.username > *start
        } else {
            true
        })
        .take(limit)
        .collect();
    
    Ok(ProfilesResponse { profiles })
}

fn query_tips_sent(
    deps: Deps,
    username: String,
    limit: Option<u32>,
    start_after: Option<String>,
) -> StdResult<TipsResponse> {
    // Check if user exists
    if !USER_PROFILES.has(deps.storage, &username) {
        return Ok(TipsResponse { tips: Vec::new() });
    }
    
    let limit = limit.unwrap_or(30) as usize;
    
    // Get all tip references for this sender
    let tip_refs = TIPS_SENT.may_load(deps.storage, &username)?.unwrap_or_default();
    
    // Filter by start_after if provided
    let filtered_refs = if let Some(after_key) = start_after {
        tip_refs
            .into_iter()
            .filter(|(_, key)| key > &after_key)
            .collect::<Vec<_>>()
    } else {
        tip_refs
    };
    
    // Sort by key (which contains timestamp) - newest first
    let mut sorted_refs = filtered_refs;
    sorted_refs.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take up to limit
    let limited_refs = sorted_refs.into_iter().take(limit).collect::<Vec<_>>();
    
    // Retrieve the actual tip records
    let mut tips = Vec::new();
    for (_, tip_key) in limited_refs {
        if let Ok(tip) = TIP_RECORDS.load(deps.storage, &tip_key) {
            tips.push(tip);
        }
    }
    
    Ok(TipsResponse { tips })
}

fn query_tips_received(
    deps: Deps,
    username: String,
    limit: Option<u32>,
    start_after: Option<String>,
) -> StdResult<TipsResponse> {
    // Check if user exists
    if !USER_PROFILES.has(deps.storage, &username) {
        return Ok(TipsResponse { tips: Vec::new() });
    }
    
    let limit = limit.unwrap_or(30) as usize;
    
    // Get all tip references for this recipient
    let tip_refs = TIPS_RECEIVED.may_load(deps.storage, &username)?.unwrap_or_default();
    
    // Filter by start_after if provided
    let filtered_refs = if let Some(after_key) = start_after {
        tip_refs
            .into_iter()
            .filter(|(_, key)| key > &after_key)
            .collect::<Vec<_>>()
    } else {
        tip_refs
    };
    
    // Sort by key (which contains timestamp) - newest first
    let mut sorted_refs = filtered_refs;
    sorted_refs.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take up to limit
    let limited_refs = sorted_refs.into_iter().take(limit).collect::<Vec<_>>();
    
    // Retrieve the actual tip records
    let mut tips = Vec::new();
    for (_, tip_key) in limited_refs {
        if let Ok(tip) = TIP_RECORDS.load(deps.storage, &tip_key) {
            tips.push(tip);
        }
    }
    
    Ok(TipsResponse { tips })
}

fn query_tip_detail(
    deps: Deps,
    from_username: String,
    to_username: String,
    timestamp: cosmwasm_std::Timestamp,
) -> StdResult<TipDetailResponse> {
    let key = format!("{}:{}:{}", from_username, to_username, timestamp.nanos());
    let tip = TIP_RECORDS.may_load(deps.storage, &key)?;
    Ok(TipDetailResponse { tip })
}

fn query_user_stats(deps: Deps, username: String) -> StdResult<StatsResponse> {
    // Check if user exists
    if !USER_PROFILES.has(deps.storage, &username) {
        return Ok(StatsResponse {
            total_tips_sent: 0,
            total_tips_received: 0,
            total_amount_sent: "0uxion".to_string(),
            total_amount_received: "0uxion".to_string(),
        });
    }
    
    // Get all tips sent
    let tips_sent = TIPS_SENT.may_load(deps.storage, &username)?.unwrap_or_default();
    let total_tips_sent = tips_sent.len() as u64;
    
    // Get all tips received
    let tips_received = TIPS_RECEIVED.may_load(deps.storage, &username)?.unwrap_or_default();
    let total_tips_received = tips_received.len() as u64;
    
    // Calculate total amount sent
    let mut total_amount_sent = 0u128;
    for (_, tip_key) in &tips_sent {
        if let Ok(tip) = TIP_RECORDS.load(deps.storage, tip_key) {
            if let Some(amount_str) = tip.amount.strip_suffix("uxion") {
                if let Ok(amount) = amount_str.parse::<u128>() {
                    total_amount_sent += amount;
                }
            }
        }
    }
    
    // Calculate total amount received
    let mut total_amount_received = 0u128;
    for (_, tip_key) in &tips_received {
        if let Ok(tip) = TIP_RECORDS.load(deps.storage, tip_key) {
            if let Some(amount_str) = tip.amount.strip_suffix("uxion") {
                if let Ok(amount) = amount_str.parse::<u128>() {
                    total_amount_received += amount;
                }
            }
        }
    }
    
    Ok(StatsResponse {
        total_tips_sent,
        total_tips_received,
        total_amount_sent: format!("{}uxion", total_amount_sent),
        total_amount_received: format!("{}uxion", total_amount_received),
    })
}

fn query_is_admin(deps: Deps, address: String) -> StdResult<AdminResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let is_admin = CONTRACT_ADMINS.may_load(deps.storage, &addr)?.unwrap_or(false);
    Ok(AdminResponse { is_admin })
}

fn query_is_username_available(deps: Deps, username: String) -> StdResult<UsernameAvailableResponse> {
    // Check if valid username format
    let is_valid = match validate_username(&username) {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // Check if already taken
    let is_available = is_valid && !USER_PROFILES.has(deps.storage, &username);
    
    Ok(UsernameAvailableResponse { is_available })
}

// Helper function to validate username format
fn validate_username(username: &str) -> Result<(), ContractError> {
    if username.len() < 3 || username.len() > 30 {
        return Err(ContractError::UsernameTooShortOrLong {});
    }
    
    // Username can only contain letters, numbers, underscores, and hyphens
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !re.is_match(username) {
        return Err(ContractError::InvalidUsername {
            username: username.to_string(),
        });
    }
    
    Ok(())
}

// Helper function to check if an address is an admin
fn is_admin(deps: Deps, addr: &Addr) -> StdResult<bool> {
    Ok(CONTRACT_ADMINS.may_load(deps.storage, addr)?.unwrap_or(false))
}
