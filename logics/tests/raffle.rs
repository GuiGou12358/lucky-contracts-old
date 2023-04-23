#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod raffle {
    use lucky::impls::{
        *,
        participant_manager::*,
        participant_filter::filter_latest_winners::*,
        reward::psp22_reward,
        reward::psp22_reward::*,
        raffle::*,
        random_generator,
        random_generator::*,
    };
    use openbrush::contracts::access_control::{*, access_control};
    use openbrush::traits::Storage;
    use lucky::impls::participant_filter::filter_latest_winners;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        participant_manager: participant_manager::Data,
        #[storage_field]
        raffle: raffle::Data,
        #[storage_field]
        random_generator: random_generator::Data,
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        filter_latest_winners: filter_latest_winners::Data,
    }

    impl ParticipantManager for Contract{}
    impl Raffle for Contract{}
    impl FilterLatestWinners for Contract{}
    impl AccessControl for Contract{}


    impl Random for Contract {
        fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomError> {
            let random = RandomGenerator::get_random_number(self, min, max)?;
            Ok(random)
        }
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
            instance.grant_role(RAFFLE_MANAGER, caller).expect("Should grant the role RAFFLE_MANAGER");
            instance.grant_role(REWARD_MANAGER, caller).expect("Should grant the role REWARD_MANAGER");
            instance.grant_role(REWARD_VIEWER, caller).expect("Should grant the role REWARD_VIEWER");
            instance.grant_role(RANDOM_GENERATOR_CONSUMER, caller).expect("Should grant the role RANDOM_GENERATOR_CONSUMER");
            instance.grant_role(PARTICIPANT_FILTER_MANAGER, caller).expect("Should grant the role PARTICIPANT_FILTER_MANAGER");
            instance
        }

        #[ink(message)]
        pub fn add_participants_with_filters(&mut self, participants: Vec<(AccountId, Balance)>) -> Result<(), ContractError>{

            let mut parts = participants.clone();

            let mut i = 0;
            while i < parts.len() {
                let p_account_id = parts.get(i).unwrap().0;
                if self._is_in_last_winners(&p_account_id) {
                    parts.remove(i);
                } else {
                    i += 1;
                }
            }

            self.add_participants(parts)?;
            Ok(())
        }

        #[ink(message)]
        pub fn run_raffle(&mut self, era: u32, rewards: Balance) -> Result<(), ContractError> {

            // select the winners
            let winners = self._run_raffle(era, rewards)?;


            // save the winners
            for winner in &winners {
                self._add_winner(winner.0);
            }

            // transfer the rewards and the winners
            ink::env::pay_with_call!(self.fund_rewards_and_add_winners(era, winners), rewards)?;

            Ok(())
        }
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        RaffleError(RaffleError),
        RewardError(RewardError),
        ParticipantManagerError(ParticipantManagerError),
    }


    /// convertor from RaffleError to ContractError
    impl From<RaffleError> for ContractError {
        fn from(error: RaffleError) -> Self {
            ContractError::RaffleError(error)
        }
    }

    /// convertor from RewardError to ContractError
    impl From<RewardError> for ContractError {
        fn from(error: RewardError) -> Self {
            ContractError::RewardError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<ParticipantManagerError> for ContractError {
        fn from(error: ParticipantManagerError) -> Self {
            ContractError::ParticipantManagerError(error)
        }
    }


    impl psp22_reward::Internal for Contract {
        fn _emit_rewards_claimed_event(&self, _account: AccountId, _amount: Balance){
            // no event for the tests
        }
        fn _emit_pending_reward_event(&self, _account: AccountId, _era: u32, _amount: Balance){
            // no event for the tests
        }
    }

    mod tests {
        use ink::env::debug_println;
        use openbrush::test_utils::accounts;

        use super::*;


        #[ink::test]
        fn test_ratio_distribution() {
            let mut contract = super::Contract::new();

            // 50 + 30 + 20 > 80 => Error
            let result = contract.set_ratio_distribution(vec![50, 30, 20], 90);
            match result {
                Err(IncorrectRatio) => debug_println!("Incorrect Ratio as expected"),
                _ => panic!("Error 1"),
            };

            // 50 + 30 + 20 = 100 => Ok
            let result = contract.set_ratio_distribution(vec![50, 30, 20], 100);
            match result {
                Ok(_) => debug_println!("No Error as expected"),
                _ => panic!("Error 2"),
            };

            // 50 + 30 + 20 < 150 => Ok
            let result = contract.set_ratio_distribution(vec![50, 30, 20], 150);
            match result {
                Ok(_) => debug_println!("No Error as expected"),
                _ => panic!("Error 2"),
            };

        }

        #[ink::test]
        fn test_run_raffle_no_ratio_set() {
            let mut contract = super::Contract::new();

            //contract.set_ratio_distribution(vec![50, 30, 20], 100).unwrap();

            let accounts = accounts();
            let participants = vec![
                (accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)
                ];
            contract.add_participants_with_filters(participants).unwrap();

            let result = contract._run_raffle(1, 1000);
            match result {
                Err(NoRatioSet) => debug_println!("NoRatioSet as expected"),
                _ => panic!("Error 1"),
            };
        }

        #[ink::test]
        fn test_run_raffle_no_participant() {
            let mut contract = super::Contract::new();

            contract.set_ratio_distribution(vec![50, 30, 20], 100).unwrap();

            let participants = vec![];
            contract.add_participants_with_filters(participants).unwrap();

            let result = contract._run_raffle(1, 1000);
            match result {
                Err(NoParticipant) => debug_println!("NoParticipant as expected"),
                _ => panic!("Error 1"),
            };
        }


        #[ink::test]
        fn test_run_raffle_no_reward() {
            let mut contract = super::Contract::new();

            contract.set_ratio_distribution(vec![50, 30, 20], 100).unwrap();

            let accounts = accounts();
            let participants = vec![
                (accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)
                ];
            contract.add_participants_with_filters(participants).unwrap();

            let result = contract._run_raffle(1, 0);
            match result {
                Err(RaffleError::NoReward) => debug_println!("NoParticipant as expected"),
                _ => panic!("Error 1"),
            };
        }
  
        #[ink::test]
        fn test_run_raffle_with_zero_in_ratio() {
            let mut contract = super::Contract::new();
            let accounts = accounts();

            let participants = vec![
                (accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)
                ];
            contract.add_participants_with_filters(participants).unwrap();

            // second winner receive nada
            contract.set_ratio_distribution(vec![50, 0, 50], 100).unwrap();

            // select the participants
            let winners = contract._run_raffle(1, 1000).unwrap();

            // assert two differents winners
            assert_eq!(winners.len(), 2); 
            assert!(winners[0] != winners[1]); 

            let mut total_rewards = 0;
            for (_, r) in  winners {
                total_rewards += r;
            }
            // assert all rewards are given
            assert_eq!(total_rewards, 1000); 
        }

        #[ink::test]
        fn test_run_raffle_not_already_done() {

            let mut contract = super::Contract::new();
            
            contract.set_ratio_distribution(vec![100], 100).unwrap();

            let accounts = accounts();
            let rewards = 1000;

            // first raffle => success
            let participants = vec![(accounts.alice, 100000)];
            contract.add_participants_with_filters(participants).unwrap();
            contract._run_raffle(2, rewards).unwrap();

            // second raffle for the same era => failure
            let participants = vec![(accounts.alice, 100000)];
            contract.add_participants_with_filters(participants).unwrap();
            let result = contract._run_raffle(2, rewards);
            match result {
                Err(RaffleError::RaffleAlreadyDone) => debug_println!("RaffleAlreadyDone as expected"),
                _ => panic!("Error 1"),
            };

            // second raffle for previous era => failure
            let result = contract._run_raffle(1,  rewards);
            match result {
                Err(RaffleError::RaffleAlreadyDone) => debug_println!("RaffleAlreadyDone as expected"),
                _ => panic!("Error 2"),
            };

            // raffle for next era => success
            contract._run_raffle(3, rewards).unwrap();

        }


        #[ink::test]
        fn test_run_raffle_share_full_rewards() {
            let mut contract = super::Contract::new();
            let accounts = accounts();

            let rewards = 1000;

            let participants = vec![
                (accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)
                ];
            contract.add_participants_with_filters(participants).unwrap();

            contract.set_ratio_distribution(vec![50, 30, 20], 100).unwrap();

            // select the participants
            let winners = contract._run_raffle(1, rewards).unwrap();

            // assert three different winners
            assert_eq!(winners.len(), 3); 
            assert_ne!(winners[0], winners[1]);
            assert_ne!(winners[0], winners[2]);
            assert_ne!(winners[1], winners[2]);

            let mut total_rewards = 0;
            for (_, r) in  winners {
                total_rewards += r;
            }
            // assert all rewards are given
            assert_eq!(total_rewards, 1000); 
        }


        #[ink::test]
        fn test_run_raffle_share_partial_rewards() {
            let mut contract = super::Contract::new();
            let accounts = accounts();

            let participants = vec![
                (accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)
                ];
            contract.add_participants_with_filters(participants).unwrap();

            contract.set_ratio_distribution(vec![50, 30, 20], 200).unwrap();

            // select the participants
            let winners = contract._run_raffle(1, 1000).unwrap();

            // assert three different winners
            assert_eq!(winners.len(), 3);
            assert_ne!(winners[0], winners[1]);
            assert_ne!(winners[0], winners[2]);
            assert_ne!(winners[1], winners[2]);

            let mut total_rewards = 0;
            for (_, r) in  winners {
                total_rewards += r;
            }
            // expected rewards: (50 + 30 + 20) / 200 * 1000 = 500
            assert_eq!(total_rewards, 500); 
        }


        #[ink::test]
        fn test_raffle_contract()  {

            let mut contract = super::Contract::new();
            contract.set_ratio_distribution(vec![50, 30, 20], 100).unwrap();

            let era = 1;
            let accounts = accounts();

            contract.add_participants_with_filters(
                vec![(accounts.alice, 100000), (accounts.bob, 100000), (accounts.charlie, 100000), 
                (accounts.django, 100000), (accounts.eve, 100000), (accounts.frank, 100000)]
            ).unwrap();

            contract.run_raffle(era, 1000).unwrap();

            let mut nb_winners = 0;
            let mut total_rewards = 0;
            if let Some(r) = get_reward(&mut contract, accounts.bob) {
                nb_winners += 1;
                total_rewards += r;
            }
            if let Some(r) = get_reward(&mut contract, accounts.alice) {
                nb_winners += 1;
                total_rewards += r;
            }
            if let Some(r) = get_reward(&mut contract, accounts.charlie) {
                nb_winners += 1;
                total_rewards += r;
            }
            if let Some(r) = get_reward(&mut contract, accounts.django) {
                nb_winners += 1;
                total_rewards += r;
            }
            if let Some(r) = get_reward(&mut contract, accounts.eve) {
                nb_winners += 1;
                total_rewards += r;
            }
            if let Some(r) = get_reward(&mut contract, accounts.frank) {
                nb_winners += 1;
                total_rewards += r;
            }            
            // assert three differents winners
            assert_eq!(nb_winners, 3); 
            // assert three differents winners
            assert_eq!(total_rewards, 1000); 

        }


        #[ink::test]
        fn test_add_participants_with_filters()  {

            let mut contract = super::Contract::new();
            contract.set_ratio_distribution(vec![50], 100).unwrap();
            contract.set_nb_winners_filtered(2).unwrap();

            let accounts = accounts();

            contract.add_participants_with_filters(
                vec![(accounts.alice, 100000)]
            ).unwrap();

            contract.run_raffle(1, 1000).unwrap();

            match get_reward(&mut contract, accounts.alice) {
                Some(r) => assert_eq!(500, r),
                _ => panic!("alice should have rewards"),
            };

            // alice claims the rewards
            contract._claim_from(accounts.alice).unwrap();

            match get_reward(&mut contract, accounts.alice) {
                Some(_) => panic!("alice should not have reward anymore"),
                _ => debug_println!("It's ok, alice has no reward"),
            };

            // second era with only Alice
            // Alice already won so it should be removed from the participants
            contract.clear_data().unwrap();

            contract.add_participants_with_filters(
                vec![(accounts.alice, 100000)]
            ).unwrap();

            let result = contract._run_raffle(2, 1000);
            match result {
                Err(RaffleError::NoParticipant) => debug_println!("NoParticipant as expected"),
                _ => panic!("NoParticipant is expected"),
            };

            // third era with Alice and Bob
            // Alice already won so Bob must win
            contract.add_participants_with_filters(
                vec![(accounts.alice, 100000), (accounts.bob, 1)]
            ).unwrap();

            contract.run_raffle(3, 1000).unwrap();

            match get_reward(&mut contract, accounts.bob) {
                Some(r) => assert_eq!(500, r),
                _ => panic!("Bob should have rewards"),
            };

            // Bob claims the rewards
            contract._claim_from(accounts.bob).unwrap();

            match get_reward(&mut contract, accounts.bob) {
                Some(_) => panic!("Bob should not have reward anymore"),
                _ => debug_println!("It's ok, Bob has no reward"),
            };

            // 4th era with Alice and Bob
            // Both already won so it should be removed from the participants
            contract.clear_data().unwrap();

            contract.add_participants_with_filters(
                vec![(accounts.alice, 100000), (accounts.bob, 1)]
            ).unwrap();

            let result = contract._run_raffle(4, 1000);
            match result {
                Err(RaffleError::NoParticipant) => debug_println!("NoParticipant as expected"),
                _ => panic!("NoParticipant is expected"),
            };
        }

        pub fn get_reward(contract: &mut super::Contract, account: AccountId) -> Option<u128> {

            if contract._has_pending_rewards_from(account) {
                let pending_rewards = contract.get_pending_rewards_from(account).unwrap();
                debug_println!("Account {:?} has pending rewards: {:?} ", account, pending_rewards);
                return pending_rewards;
            }
            None

        }

    }
}