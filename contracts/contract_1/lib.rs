#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract_1 {
    use ink_prelude::vec::Vec;
    use ink_lang::codegen::{
        EmitEvent,
        Env,
    };
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, RoleType};

    use loto::impls::{
        game,
        game::*,
        manual_participant_management,
        raffle,
        reward::psp22_reward,
        reward::psp22_reward::*,
    };
    use loto::impls::raffle::Raffle;

    /// constants for managing access
    const PARTICIPANT_MANAGER: RoleType = ink_lang::selector_id!("PARTICIPANT_MANAGER");
    const REWARD_MANAGER: RoleType = ink_lang::selector_id!("REWARD_MANAGER");
    const CONTRACT_MANAGER: RoleType = ink_lang::selector_id!("CONTRACT_MANAGER");
    const VIEWER: RoleType = ink_lang::selector_id!("VIEWER");

    /// Event emitted when a user claim rewards
    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
    }

    /// Event emitted when teh Rafle is done
    #[ink(event)]
    pub struct RafleDone {
        #[ink(topic)]
        contract: AccountId,
        era: u128,
        pending_rewards: Balance,
        nb_winners: u8,
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        RewardError(psp22_reward::RewardError),
        AccessControlError(AccessControlError),
    }

    /// convertor from RewardError to ContractError
    impl From<psp22_reward::RewardError> for ContractError {
        fn from(error: psp22_reward::RewardError) -> Self {
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
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        participants_manager: manual_participant_management::Data,
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        rafle: raffle::Data,
        #[storage_field]
        game: game::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contracts
    impl Raffle for Contract {}
    impl Game for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                //instance._init_with_owner(caller);
                instance._init_with_admin(caller);
                instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
                instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
                instance.grant_role(CONTRACT_MANAGER, caller).expect("Should grant the role CONTRACT_MANAGER");
                instance.grant_role(VIEWER, caller).expect("Should grant the role VIEWER");

            })
        }

/*
        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn add_participant(&mut self, era: u128, participant: AccountId, value: Balance) -> Result<(), ContractError> {
            self._add_participant(era, participant, value);
            Ok(())
        }
        */

        #[ink(message)]
        #[modifiers(only_role(CONTRACT_MANAGER))]
        pub fn set_config_distribution(&mut self, ratio: Vec<Balance>) -> Result<(), ContractError> {
            let max_winners_by_raffle = ratio.len();
            self._set_ratio_distribution(ratio);
            self._set_max_winners_by_raffle(max_winners_by_raffle as u8);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(CONTRACT_MANAGER))]
        pub fn run_raffle(&mut self, era: u128) -> Result<(), ContractError> {
            let pending_reward = self._play(era)?;

            self.env().emit_event( RafleDone{
                contract: self.env().caller(),
                era,
                nb_winners: pending_reward.nb_winners,
                pending_rewards: pending_reward.given_reward,
            } );

            Ok(())
        }
/*
        #[ink(message)]
        pub fn claim(&mut self) {
            let from = self.env().caller();
            let amount= self._claim_from(from).expect("Should claim");
            if amount > 0 {
                self.env().emit_event(RewardsClaimed { account: from, amount });
            }
        }

        #[ink(message)]
        pub fn has_pending_rewards(&self, era: Option<u128>) -> bool {
            let from = self.env().caller();
            self.has_pending_rewards_from(era, Some(from))
        }
*/
    }

    impl psp22_reward::Internal for Contract {
        fn _emit_reward_claimed_event(&self, account: AccountId, amount: Balance){
            self.env().emit_event(RewardsClaimed { account, amount });
        }
    }

}
