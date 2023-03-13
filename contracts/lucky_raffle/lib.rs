#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod rafle_contract {
    use ink::env::call::{ExecutionInput, Selector};
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, DEFAULT_ADMIN_ROLE};

    use lucky::impls::{
        raffle,
        raffle::*,
    };
    use lucky::traits::oracle::{OracleDataConsumerRef};
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
        nb_winners: u8,
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


    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        raffle: raffle::Data,
        #[storage_field]
        access: access_control::Data,
        dapps_staking_developer_address: Option<AccountId>,
        lucky_oracle_address: Option<AccountId>,
        random_generator_address: Option<AccountId>,
        reward_manager_address: Option<AccountId>,
    }

    impl Random for Contract {
        fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomError> {
            // get the random number
            let random_generator_address = self.random_generator_address.ok_or(RandomError::MissingAddress)?;
            let random = RandomGeneratorRef::get_random_number(&random_generator_address, min, max)?;
            Ok(random)
        }
    }

    impl Raffle for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            lucky_oracle_address: AccountId,
            random_generator_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(RAFFLE_MANAGER, caller).expect("Should grant the role RAFFLE_MANAGER");
            instance.dapps_staking_developer_address = Some(dapps_staking_developer_address);
            instance.lucky_oracle_address = Some(lucky_oracle_address);
            instance.random_generator_address = Some(random_generator_address);
            instance.reward_manager_address = Some(reward_manager_address);
            instance
        }


        #[ink(message)]
        #[modifiers(only_role(RAFFLE_MANAGER))]
        pub fn call_2(&mut self, rewards: Balance) -> Result<(), ContractError> {

            // withdraw the rewards from developer dAppsStaking
            let dapps_staking_developer_address = self.dapps_staking_developer_address.ok_or(ContractError::DappsStakingDeveloperAddressMissing)?;
            let r = ink::env::call::build_call::<Environment>()
                .call(dapps_staking_developer_address)
                .exec_input(
                    ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR))
                        .push_arg(rewards)
                )
                .returns::<()>()
                .invoke();
                //.try_invoke()
                //.map_err(|_| ContractError::CrossContractCallError2a)?
                //.map_err(|_| ContractError::CrossContractCallError2b)?;
            Ok(r)
        }
        



        #[ink(message)]
        #[modifiers(only_role(RAFFLE_MANAGER))]
        pub fn run_raffle(&mut self, era: u32) -> Result<(), ContractError> {

            // get the oracle data
            let lucky_oracle_address = self.lucky_oracle_address.ok_or(ContractError::LuckyOracleAddressMissing)?;
            let oracle_data = OracleDataConsumerRef::get_data(&lucky_oracle_address, era);

            let participants = oracle_data.participants;
            let rewards = oracle_data.rewards;

            // select the participants
            let winners = self._run_raffle(era, participants, rewards)?;
            let nb_winners = winners.len();

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

            // emit event RaffleDone
            self.env().emit_event(RaffleDone {
                contract: self.env().caller(),
                era,
                nb_winners: nb_winners as u8,
                pending_rewards: rewards,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_role_raffle_manager(&self) -> RoleType {
            RAFFLE_MANAGER
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
        pub fn set_lucky_oracle_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.lucky_oracle_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_lucky_oracle_address(&mut self) -> Option<AccountId> {
            self.lucky_oracle_address
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
