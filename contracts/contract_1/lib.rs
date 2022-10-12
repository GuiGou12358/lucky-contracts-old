#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use ink_lang as ink;

#[ink::contract]
pub mod contract_1 {
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, RoleType};

    use loto::impls::{
        *,
        game::*,
        manual_participant_management::*,
        rafle::*,
        reward::psp22::*,
        reward::psp22::psp22_reward::*,
    };

    /// constants for managing access
    const MANAGER: RoleType = ink_lang::selector_id!("MANAGER");
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

    /// Errors occuried in the contract
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
        rafle: rafle::Data,
        #[storage_field]
        game: game::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contractss
    impl ParticipantManagement for Contract {}
    impl Reward for Contract {}
    impl Rafle for Contract {}
    impl Game for Contract {}
    impl Psp22Reward for Contract {}
    //impl native_psp22_reward::Internal for Contract {}
    //impl Ownable for Contract {}
    impl AccessControl for Contract {}


    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                //instance._init_with_owner(caller);
                instance._init_with_admin(caller);
                instance.grant_role(MANAGER, caller).expect("Should grant the role MANAGER");
                instance.grant_role(VIEWER, caller).expect("Should grant the role VIEWER");

            })
        }
/*
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = manual_participant_management::Data::default();
                instance.rafle = rafle::Data::default();
                instance.game = game::Data::default();
                instance.reward = psp22_reward::Data::default();
            })
        }
*/

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn add_participant(&mut self, era: u128, participant: AccountId, value: Balance) -> Result<(), ContractError> {
            self._add_participant(era, participant, value);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn set_ratio_distribution(&mut self, ratio: Vec<Balance>) -> Result<(), ContractError> {
            let nb_winners_by_rafle = ratio.len();
            self._set_ratio_distribution(ratio);
            self.rafle.set_nb_winners_by_rafle(nb_winners_by_rafle as u8);
            Ok(())
        }

        /// Set the total rewards shared by all wiiners for a given era
        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn set_total_rewards(&mut self, era: u128, amount: Balance) -> Result<(), ContractError> {
            self._set_total_rewards(era, amount);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(MANAGER))]
        pub fn run_rafle(&mut self, era: u128) -> Result<(), ContractError> {
            let pending_reward = self._play(era)?;

            self.env().emit_event( RafleDone{
                contract: self.env().caller(),
                era,
                nb_winners: pending_reward.nb_winners,
                pending_rewards: pending_reward.given_reward,
            } );

            Ok(())
        }

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
            self._has_pending_rewards_from(era, Some(from))
        }

        #[ink(message)]
        // TODO manage access (easier to remove it for testing)
        //#[modifiers(only_role(VIEWER))]
        pub fn list_pending_rewards_from(&self, era: Option<u128>, account: Option<AccountId>)
                                         //-> Result<Vec<(AccountId, u128, Balance)>, AccessControlError>
            -> Vec<(AccountId, u128, Balance)>
        {
            let rewards = self._list_pending_rewards_from(era, account);
            rewards
            //Ok(rewards)
        }

    }

}
