#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod rafle_contract {
    use ink_env::call::{Call, ExecutionInput, Selector};
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, DEFAULT_ADMIN_ROLE};

    use lucky::impls::{
        raffle,
        raffle::*,
    };
    use lucky::traits::oracle::OracleDataConsumerRef;

    // Selector of withdraw: "0x410fcc9d"
    const WITHDRAW_SELECTOR : [u8; 4] = [0x41, 0x0f, 0xcc, 0x9d];
    // Selector of Psp22Reward::fund_rewards_and_add_winners": ""0xc218e5ba
    const FUND_REWARDS_AND_WINNERS_SELECTOR : [u8; 4] = [0xc2, 0x18, 0xe5, 0xba];

    /// Event emitted when the Rafle is done
    #[ink(event)]
    pub struct RafleDone {
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
        TransferError,
        UpgradeError,
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
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        raffle: raffle::Data,
        #[storage_field]
        access: access_control::Data,
        dapps_staking_developer_address: AccountId,
        lucky_oracle_address: AccountId,
        reward_manager_address: AccountId,
    }

    impl Raffle for Contract{}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            lucky_oracle_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(RAFFLE_MANAGER, caller).expect("Should grant the role RAFFLE_MANAGER");
                instance.dapps_staking_developer_address = dapps_staking_developer_address;
                instance.lucky_oracle_address = lucky_oracle_address;
                instance.reward_manager_address = reward_manager_address;
            })
        }

        #[ink(message)]
        #[modifiers(only_role(RAFFLE_MANAGER))]
        pub fn run_raffle(&mut self, era: u32) -> Result<(), ContractError> {

            // get the oracle data
            let oracle_data = OracleDataConsumerRef::get_data(&mut self.lucky_oracle_address, era);

            let participants = oracle_data.participants;
            let rewards = oracle_data.rewards;

            // select the participants
            let winners = self._run_raffle(era, participants, rewards)?;
            let nb_winners = winners.len();

            // withdraw the rewards from developer dAppsStaking
            ink_env::call::build_call::<Environment>()
                .call_type(
                    Call::new()
                        .callee(self.dapps_staking_developer_address)
                )
                .exec_input(
                    ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR))
                        .push_arg(rewards)
                )
                .returns()
                .fire()
                .map_err(|_| ContractError::CrossContractCallError1)?;


            // set the list of winners and fund the rewards 
            ink_env::call::build_call::<Environment>()
                .call_type(
                    Call::new()
                        .callee(self.reward_manager_address)
                        .transferred_value(rewards)
                )
                .exec_input(
                    ExecutionInput::new(Selector::new(FUND_REWARDS_AND_WINNERS_SELECTOR))
                        .push_arg(era)
                        .push_arg(winners)
                )
                .returns()
                .fire()
                .map_err(|_| ContractError::CrossContractCallError2)?;

            // emit event RafleDone
            self.env().emit_event(RafleDone {
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
            self.dapps_staking_developer_address = address;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_lucky_oracle_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.lucky_oracle_address = address;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_reward_manager_address(&mut self, address: AccountId) -> Result<(), ContractError> {
            self.reward_manager_address = address;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink_env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
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
