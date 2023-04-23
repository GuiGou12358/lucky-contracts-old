#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod raffle_contract {
    use ink::env::call::{ExecutionInput, Selector};
    use ink::prelude::vec::Vec;
    use openbrush::{modifiers, traits::{Storage}};
    use openbrush::contracts::access_control::{*, AccessControlError, DEFAULT_ADMIN_ROLE};

    use lucky::impls::{
        raffle, raffle::*,
        participant_manager, participant_manager::*,
        participant_filter::filter_latest_winners, participant_filter::filter_latest_winners::*
    };
    use lucky::traits::random_generator::{RandomGeneratorRef};

    // Selector of withdraw: "0x410fcc9d"
    const WITHDRAW_SELECTOR : [u8; 4] = [0x41, 0x0f, 0xcc, 0x9d];
    // Selector of Psp22Reward::fund_rewards_and_add_winners": ""0xc218e5ba
    const FUND_REWARDS_AND_WINNERS_SELECTOR : [u8; 4] = [0xc2, 0x18, 0xe5, 0xba];

    /// Event emitted when the Rafle is done
    #[ink(event)]
    pub struct RaffleDone {
        #[ink(topic)]
        contract: AccountId,
        #[ink(topic)]
        era: u32,
        pending_rewards: Balance,
        nb_winners: u16,
        nb_participants: u16,
        total_value: Balance,
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        RaffleError(RaffleError),
        RaffleAlreadyDone,
        CrossContractCallError1,
        CrossContractCallError2,
        CrossContractCallError2a,
        CrossContractCallError2b,
        TransferError,
        UpgradeError,
        LuckyOracleAddressMissing,
        RandomGeneratorAddressMissing,
        DappsStakingDeveloperAddressMissing,
        RewardManagerAddressMissing,
        ParticipantManagerError(ParticipantManagerError),
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<RaffleError> for ContractError {
        fn from(error: RaffleError) -> Self {
            ContractError::RaffleError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<ParticipantManagerError> for ContractError {
        fn from(error: ParticipantManagerError) -> Self {
            ContractError::ParticipantManagerError(error)
        }
    }


    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        participant_manager: participant_manager::Data,
        #[storage_field]
        raffle: raffle::Data,
        #[storage_field]
        access: access_control::Data,
        dapps_staking_developer_address: Option<AccountId>,
        random_generator_address: Option<AccountId>,
        reward_manager_address: Option<AccountId>,
        #[storage_field]
        filter_latest_winners: filter_latest_winners::Data,
    }

    impl Random for Contract {
        fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomError> {
            // get the random number
            let random_generator_address = self.random_generator_address.ok_or(RandomError::MissingAddress)?;
            let random = RandomGeneratorRef::get_random_number(&random_generator_address, min, max)?;
            Ok(random)
        }
    }

    impl ParticipantManager for Contract{}
    impl Raffle for Contract{}
    impl FilterLatestWinners for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            random_generator_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(RAFFLE_MANAGER, caller).expect("Should grant the role RAFFLE_MANAGER");
            instance.grant_role(PARTICIPANT_MANAGER, caller).expect("Should grant the role PARTICIPANT_MANAGER");
            instance.grant_role(PARTICIPANT_FILTER_MANAGER, caller).expect("Should grant the role PARTICIPANT_FILTER_MANAGER");
            instance.dapps_staking_developer_address = Some(dapps_staking_developer_address);
            instance.random_generator_address = Some(random_generator_address);
            instance.reward_manager_address = Some(reward_manager_address);
            instance
        }

        /// add participants in the raffle and applied the filters
        /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
        /// weight can represent the number of raffle tickets for this participant.
        /// weight can also represent the amount staked in dAppStaking, ...
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
        #[modifiers(only_role(RAFFLE_MANAGER))]
        pub fn run_raffle(&mut self, era: u32, rewards: Balance) -> Result<(), ContractError> {


            // select the winners
            // initialize the empty list of randomly selected values
            let winners = self._run_raffle(era, rewards)?;
            let nb_winners = winners.len();

            // save the winners
            for winner in &winners {
                self._add_winner(winner.0);
            }

            // withdraw the rewards from developer dAppsStaking
            let dapps_staking_developer_address = self.dapps_staking_developer_address.ok_or(ContractError::DappsStakingDeveloperAddressMissing)?;
            ink::env::call::build_call::<Environment>()
                .call(dapps_staking_developer_address)
                .exec_input(
                    ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR))
                        .push_arg(rewards)
                )
                .returns::<()>()
                .invoke();
                //.map_err(|_| ContractError::CrossContractCallError1)?;


            // set the list of winners and fund the rewards 
            let reward_manager_address = self.reward_manager_address.ok_or(ContractError::RewardManagerAddressMissing)?;
            ink::env::call::build_call::<Environment>()
                .call(reward_manager_address)
                .transferred_value(rewards)
                .exec_input(
                    ExecutionInput::new(Selector::new(FUND_REWARDS_AND_WINNERS_SELECTOR))
                        .push_arg(era)
                        .push_arg(winners)
                )
                .returns::<()>()
                .invoke();
                //.map_err(|_| ContractError::CrossContractCallError2)?;

            // TODO split rewards given and total rewards
            let total_value = self.get_total_value();

            // emit event RaffleDone
            self.env().emit_event(RaffleDone {
                contract: self.env().caller(),
                era,
                nb_winners: nb_winners as u16,
                nb_participants: self.get_nb_participants(),
                total_value,
                pending_rewards: rewards,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_role_raffle_manager(&self) -> RoleType {
            RAFFLE_MANAGER
        }

        #[ink(message)]
        pub fn get_role_participant_manager(&self) -> RoleType {
            PARTICIPANT_MANAGER
        }

        #[ink(message)]
        pub fn get_role_participant_filter_manager(&self) -> RoleType {
            PARTICIPANT_FILTER_MANAGER
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_dapps_staking_developer_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.dapps_staking_developer_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_dapps_staking_developer_address(&mut self) -> Option<AccountId> {
            self.dapps_staking_developer_address
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_random_generator_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.random_generator_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_random_generator_address(&mut self) -> Option<AccountId> {
            self.random_generator_address
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_reward_manager_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.reward_manager_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_reward_manager_address(&mut self) -> Option<AccountId> {
            self.reward_manager_address
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink::env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError>{
            let caller = Self::env().caller();
            Self::env().transfer(caller, value).map_err(|_| ContractError::TransferError)?;
            Ok(())
        }

    }

}
