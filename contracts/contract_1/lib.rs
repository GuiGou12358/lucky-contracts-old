#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract_1 {
    use ink_lang::codegen::{
        EmitEvent,
        Env,
    };
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, RoleType};

    use loto::impls::{
        game,
        game::*,
        manual_participant_management,
        manual_participant_management::*,
        raffle,
        raffle::*,
        reward::psp22_reward,
        reward::psp22_reward::*,
    };

    /// constants for managing access
    const CONTRACT_MANAGER: RoleType = ink_lang::selector_id!("CONTRACT_MANAGER");

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
        RewardError(RewardError),
        GameError(GameError),
        AccessControlError(AccessControlError),
    }

    /// convertor from RewardError to ContractError
    impl From<RewardError> for ContractError {
        fn from(error: RewardError) -> Self {
            ContractError::RewardError(error)
        }
    }

    /// convertor from GameError to ContractError
    impl From<GameError> for ContractError {
        fn from(error: GameError) -> Self {
            ContractError::GameError(error)
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
    impl Psp22Reward for Contract{}
    impl ParticipantManagement for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
                instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
                instance.grant_role(REWARD_VIEWER, caller).expect("Should grant the role REWARD_VIEWER");
                instance.grant_role(CONTRACT_MANAGER, caller).expect("Should grant the role CONTRACT_MANAGER");

            })
        }

        #[ink(message)]
        #[modifiers(only_role(CONTRACT_MANAGER))]
        pub fn set_config_distribution(&mut self, ratio: Vec<Balance>) -> Result<(), ContractError> {
            let max_winners_by_raffle = ratio.len();
            self._set_ratio_distribution(ratio)?;
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

    }

    impl psp22_reward::Internal for Contract {
        fn _emit_reward_claimed_event(&self, account: AccountId, amount: Balance){
            self.env().emit_event(RewardsClaimed { account, amount });
        }
    }

}
