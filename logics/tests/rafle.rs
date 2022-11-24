#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod helper {
    use ink_storage::traits::SpreadAllocate;
    use lucky::helpers::{helper, helper::*};
    use lucky::impls::{
        *,
        participant_management::*,
        reward::*,
        reward::psp22_reward::*,
    };
    use openbrush::contracts::access_control::{*, access_control, AccessControl};
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        participants_manager: participant_management::Data,
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl Psp22Reward for Contract{}
    impl ParticipantManager for Contract{}
    impl ParticipantReader for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.participants_manager = participant_management::Data::default();
                instance.reward = psp22_reward::Data::default();
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
                instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
                instance.grant_role(REWARD_VIEWER, caller).expect("Should grant the role REWARD_VIEWER");
            })
        }

        #[ink(message)]
        pub fn run_raffle(&mut self, era: u32, rewards: Balance, nb_winners: u32) -> Result<(), ContractError> {

            // get the participants
            let participants = self.list_participants(era);
            // select the participants
            let winners = helper::select_winners(self, participants, nb_winners as usize)?;

            // transfer the reward and the winners
            ink_env::pay_with_call!(self.fund_rewards_and_add_winners(era, winners), rewards)?;

            //self.fund_rewards_and_add_winners(era, winners)?;

            Ok(())
        }
            }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        HelperError(HelperError),
        RewardError(RewardError),
    }

    /// convertor from HelperError to ContractError
    impl From<HelperError> for ContractError {
        fn from(error: HelperError) -> Self {
            ContractError::HelperError(error)
        }
    }
    /// convertor from RewardError to ContractError
    impl From<RewardError> for ContractError {
        fn from(error: RewardError) -> Self {
            ContractError::RewardError(error)
        }
    }


    impl psp22_reward::Internal for Contract {
        fn _emit_rewards_claimed_event(&self, _account: AccountId, _amount: Balance){
            // no event for the tests
        }
        fn _emit_pending_reward_event(&self, _account: AccountId, _amount: Balance){
            // no event for the tests
        }
    }

    mod tests {
        use ink_env::debug_println;
        use ink_lang as ink;
        use openbrush::test_utils::accounts;

        use super::*;

        pub fn test(contract: &mut super::Contract, era: u32, nb_winners: u32){

            //use ink_env::codegen::Env;

            let accounts = accounts();

            //ink_env::pay_with_call!(contract.fund_rewards(era), 1000).unwrap();

            //contract.fund_rewards(era, 110).unwrap();
            contract.add_participant(era, accounts.alice, 100000).unwrap();
            contract.add_participant(era, accounts.bob, 100000).unwrap();
            contract.add_participant(era, accounts.charlie, 100000).unwrap();
            contract.add_participant(era, accounts.django, 100000).unwrap();
            contract.add_participant(era, accounts.eve, 100000).unwrap();
            contract.add_participant(era, accounts.frank, 100000).unwrap();
            contract.run_raffle(era, 1000, nb_winners).unwrap();

            debug_println!("winner era {} : {:?} ", era, contract.list_pending_rewards_from(Some(era), None));
        }


        #[ink::test]
        fn test_multi_users() {
            let mut contract = super::Contract::default();
            contract.set_ratio_distribution(vec![50, 30, 20]).unwrap();
            test(&mut contract, 1, 3);
        }

    }
}