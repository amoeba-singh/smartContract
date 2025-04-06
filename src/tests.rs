use crate::contract::{execute, instantiate, query};
use crate::msg::{
    BuyCreditsMessage, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RegisterUserMessage,
    TransactionsResponse, UseCreditsMessage, UserResponse,
};

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Addr};
use crate::error::ContractError;

const ADMIN: &str = "admin";
const USER: &str = "user1";

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);

    // Test instantiation with no admin (sender becomes admin)
    let msg = InstantiateMsg { admin: None };
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(3, res.attributes.len());

    // Query the config to verify
    let config: ConfigResponse = from_json(
        &query(deps.as_ref(), env.clone(), QueryMsg::GetConfig {}).unwrap()
    ).unwrap();
    assert_eq!(config.config.admin, Addr::unchecked(ADMIN));
}

#[test]
fn register_user() {
    
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // First instantiate the contract
    let info = mock_info(ADMIN, &[]);
    let msg = InstantiateMsg { admin: None };
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

     // Register a user
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
     
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    
    let user: UserResponse = from_json(
        &query(deps.as_ref(), env.clone(), QueryMsg::GetUser{ address: Addr::unchecked(USER) }).unwrap()
    ).unwrap();
    
    assert_eq!(user.user.credit_balance, 10);
    
    
    // Try to register again (should fail)
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {  });
    
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::UserExists {}));
}

#[test]
fn buy_credits_basic_bundle() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup
    let info = mock_info(ADMIN, &[]);
    let msg = InstantiateMsg { admin: None };
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register a user
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
     
    // Buy basic bundle
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info=mock_info(USER, &[]);
    let msg = ExecuteMsg::BuyCredits(BuyCreditsMessage{
        bundle:"Basic".to_string(),
    });
    

    // let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InsufficientAmount {}));

    
    let info=mock_info(USER, &coins(10_000,"uxion"));
    let msg = ExecuteMsg::BuyCredits(BuyCreditsMessage{
        bundle:"Basic".to_string(),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    // Query the user's updated balance
    let user_res: UserResponse = from_json(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUser { address: Addr::unchecked(USER) },
        ).unwrap()
    ).unwrap();
    
    // Initial 10 + 10 from Basic bundle
    assert_eq!(user_res.user.credit_balance, 20);


    let transactions: TransactionsResponse = from_json(
        &query(deps.as_ref(), env.clone(), QueryMsg::GetTransactions { address: Addr::unchecked(USER) }).unwrap()
    ).unwrap();
    assert_eq!(transactions.transactions.len(), 1);
    assert_eq!(transactions.transactions[0].credits, 10);
    assert_eq!(transactions.transactions[0].label, "BOUGHT");

}


#[test]
fn buy_credits_premium_bundle() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Setup
    let info = mock_info(ADMIN, &[]);
    let msg = InstantiateMsg { admin: None };
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register a user
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
     
    // Buy basic bundle
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info=mock_info(USER, &[]);
    let msg = ExecuteMsg::BuyCredits(BuyCreditsMessage{
        bundle:"Premium".to_string(),
    });
    

    // let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InsufficientAmount {}));

    
    let info=mock_info(USER, &coins(40_000,"uxion"));
    let msg = ExecuteMsg::BuyCredits(BuyCreditsMessage{
        bundle:"Premium".to_string(),
    });


    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    // Query the user's updated balance
    let user_res: UserResponse = from_json(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUser { address: Addr::unchecked(USER) },
        ).unwrap()
    ).unwrap();
    
    // Initial 10 + 10 from Basic bundle
    assert_eq!(user_res.user.credit_balance, 110);


    let transactions: TransactionsResponse = from_json(
        &query(deps.as_ref(), env.clone(), QueryMsg::GetTransactions { address: Addr::unchecked(USER) }).unwrap()
    ).unwrap();
    assert_eq!(transactions.transactions.len(), 1);
    assert_eq!(transactions.transactions[0].credits, 100);
    assert_eq!(transactions.transactions[0].label, "BOUGHT");

}

    

#[test]
fn use_credits_success() {
    
    let mut deps = mock_dependencies();
    let _env = mock_env();
    
    
    // Register a user first
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    // Use 5 credits
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::UseCredits(UseCreditsMessage { 
        credits: 1
    });
    
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(3, res.attributes.len());
    
    // Query the user's updated balance
    let user_res: UserResponse = from_json(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUser { address: Addr::unchecked(USER) },
        ).unwrap()
    ).unwrap();
    
    // Initial 10 - 5 used
    assert_eq!(user_res.user.credit_balance, 9);
}

#[test]
fn use_credits_insufficient() {
    let mut deps = mock_dependencies();
    let _env = mock_env();
    
    // Register a user first
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    // Try to use more credits than available
    let info = mock_info(USER, &[]);
    let msg = ExecuteMsg::UseCredits(UseCreditsMessage { 
        credits: 15 // User only has 10 credits
    });
    
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InsufficientCredits {}));
}

// #[test]
// fn query_transactions() {
//     let (mut deps, _) = setup_contract();
    
//     // Register a user first
//     let info = mock_info(USER1, &[]);
//     let msg = ExecuteMsg::RegisterUser(RegisterUserMessage {});
//     let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
//     // Buy credits to create a transaction
//     let payment = coins(10_000, DENOM);
//     let info = mock_info(USER1, &payment);
//     let msg = ExecuteMsg::BuyCredits(BuyCreditsMessage { 
//         bundle: "Basic".to_string() 
//     });
//     let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
//     // Use credits to create another transaction
//     let info = mock_info(USER1, &[]);
//     let msg = ExecuteMsg::UseCredits(UseCreditsMessage { 
//         credits: 5 
//     });
//     let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
//     // Query transactions
//     let tx_res: TransactionsResponse = from_json(
//         &query(
//             deps.as_ref(),
//             mock_env(),
//             QueryMsg::GetTransactions { address: Addr::unchecked(USER1) },
//         ).unwrap()
//     ).unwrap();
    
//     // Should have 2 transactions
//     assert_eq!(tx_res.transactions.len(), 2);
//     assert_eq!(tx_res.transactions[0].label, "BOUGHT");
//     assert_eq!(tx_res.transactions[0].credits, 10);
//     assert_eq!(tx_res.transactions[1].label, "USED");
//     assert_eq!(tx_res.transactions[1].credits, 5);
// }

// #[test]
// fn query_nonexistent_user() {
//     let (deps, _) = setup_contract();
    
//     // Try to query a user that doesn't exist
//     let err = query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::GetUser { address: Addr::unchecked("nonexistent") },
//     ).unwrap_err();
    
//     assert!(matches!(err, StdError::NotFound { .. }));
// }
