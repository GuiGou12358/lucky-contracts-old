#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod reward_manager {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use openbrush::{
        modifiers,
        traits::Storage
    };
    use openbrush::contracts::access_control::{
        *,
        AccessControlError,
        AccessControl,
        RoleType
    };
    use lucky::impls::{
        reward::psp22_reward,
        reward::psp22_reward::*,
    };


    const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Event emitted when a reward is pending
    #[ink(event)]
    pub struct PendingReward {
        #[ink(topic)]
        account: AccountId,
        era: u32,
        amount: Balance,
    }

    /// Event emitted when a user claim rewards
    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
    }



    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        RewardError(RewardError),
        AccessControlError(AccessControlError),
        UpgradeError,
        TransferError,
    }

    /// convertor from RewardError to ContractError
    impl From<RewardError> for ContractError {
        fn from(error: RewardError) -> Self {
            ContractError::RewardError(error)
        }
    }

    /// convertor from AccessControlError to ContractError
    impl From<access_control::AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contracts
    impl Psp22Reward for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
            instance.grant_role(REWARD_VIEWER, caller).expect("Should grant the role REWARD_VIEWER");
            instance
        }


        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink::env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_role_reward_manager(&self) -> RoleType {
            REWARD_MANAGER
        }

        #[ink(message)]
        pub fn get_role_reward_viewer(&self) -> RoleType {
            REWARD_VIEWER
        }

        #[ink(message)]
        #[openbrush::modifiers(only_role(WHITELISTED_ADDRESS))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError>{
            let caller = Self::env().caller();
            Self::env().transfer(caller, value).map_err(|_| ContractError::TransferError)?;
            Ok(())
        }

    }

    impl psp22_reward::Internal for Contract {

        fn _emit_pending_reward_event(&self, account: AccountId, era: u32, amount: Balance){
            self.env().emit_event(PendingReward { account, era, amount });
        }

        fn _emit_rewards_claimed_event(&self, account: AccountId, amount: Balance){
            self.env().emit_event(RewardsClaimed { account, amount });
        }
    }

}
