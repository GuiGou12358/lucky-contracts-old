#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod rafle_contract {
    use dapps_staking_extension::*;
    use ink_env::call::{Call, ExecutionInput, Selector};
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{modifiers, traits::Storage};
    use openbrush::contracts::access_control::{*, AccessControlError, DEFAULT_ADMIN_ROLE, RoleType};

    use lucky::helpers::{
        helper,
        helper::HelperError,
    };
    use lucky::impls::reward::psp22_reward::PendingReward;
    use lucky::traits::participant_management::ParticipantReaderRef;

    //use lucky::traits::reward::psp22_reward::Psp22RewardRef;
    //use dapps_staking_developer::dapps_staking_developer::ContractRef as DAppsStakingContractRef;

    // Selector of withdraw: "0x410fcc9d"
    const WITHDRAW_SELECTOR : [u8; 4] = [0x41, 0x0f, 0xcc, 0x9d];
    // Selector of Psp22Reward::fund_rewards_and_add_winners": ""0xc218e5ba
    const FUND_REWARDS_AND_WINNERS_SELECTOR : [u8; 4] = [0xc2, 0x18, 0xe5, 0xba];

    pub const CONTRACT_MANAGER: RoleType = ink_lang::selector_id!("CONTRACT_MANAGER");

    /// Event emitted when the Rafle is done
    #[ink(event)]
    pub struct RafleDone {
        #[ink(topic)]
        contract: AccountId,
        era: u32,
        pending_rewards: Balance,
        nb_winners: u8,
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        HelperError(HelperError),
        DSError,
        CrossContractCallError1,
        CrossContractCallError2,
        UpgradeError,
        SubOverFlow,
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// convertor from HelperError to ContractError
    impl From<HelperError> for ContractError {
        fn from(error: HelperError) -> Self {
            ContractError::HelperError(error)
        }
    }



    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        access: access_control::Data,
        dapps_staking_developer_address: AccountId,
        dapps_staking_contract_address: AccountId,
        participants_oracle_address: AccountId,
        reward_manager_address: AccountId,
    }

    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            dapps_staking_contract_address: AccountId,
            participants_oracle_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_admin(caller);
                instance.grant_role(CONTRACT_MANAGER, caller).expect("Should grant the role CONTRACT_MANAGER");
                instance.dapps_staking_developer_address = dapps_staking_developer_address;
                instance.dapps_staking_contract_address = dapps_staking_contract_address;
                instance.participants_oracle_address = participants_oracle_address;
                instance.reward_manager_address = reward_manager_address;
            })
        }

        #[ink(message)]
        #[modifiers(only_role(CONTRACT_MANAGER))]
        pub fn run_raffle(&mut self, era: u32) -> Result<(), ContractError> {

            // get the balance of dAppsStaking developer account before claiming the dApp
            let balance_developer_before = self.env().balance(); // TODO fixme
            // claim the rewards for the developer
            DappsStaking::claim_dapp(self.dapps_staking_contract_address, era).map_err(|_| ContractError::DSError)?;
            // get the balance of dAppsStaking developer account before claiming the dApp
            let balance_developer_after = self.env().balance(); // TODO fixme
            // compute the rewards
            let reward_developer = balance_developer_after.checked_sub(balance_developer_before).ok_or(ContractError::SubOverFlow)?;

            // TODO only for dev
            let reward_developer_2 = if reward_developer > 0 { reward_developer } else { reward_developer.checked_add(2222).ok_or(ContractError::SubOverFlow)? }; // TODO remove it

            // get the balance before withdrawing
            let balance_before = self.env().balance();

            // I have teh following issue with this piece of code
            // DAppsStakingContractRef::withdraw(&mut self.dapps_staking_developer_address, reward_developer_2)
            //  --------------------------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `dapps_staking_developer::ContractRef`, found struct `ink_env::AccountId`
            /*
            DAppsStakingContractRef::withdraw(&mut self.dapps_staking_developer_address, reward_developer_2)
                .map_err(|_| ContractError::CrossContractCallError1)?;
             */
            // withdraw the amount from developer dAppsStaking
            ink_env::call::build_call::<Environment>()
                .call_type(
                    Call::new()
                        .callee(self.dapps_staking_developer_address)
                )
                .exec_input(
                    ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR))
                        .push_arg(reward_developer_2)
                )
                .returns()
                .fire()
                .map_err(|_| ContractError::CrossContractCallError1)?;

            // get the balance after withdrawing
            let balance_after = self.env().balance();

            // compute the reward
            let reward = balance_after.checked_sub(balance_before).ok_or(ContractError::SubOverFlow)?;

            // get the participants
            let participants = ParticipantReaderRef::list_participants(&mut self.participants_oracle_address, era);
            // select the participants
            let winners = helper::select_winners(self, participants, 1)?; // TODO get the number of winners

            // transfer the reward and the winners
            // I cannot use the wrapper ref because I need to provide a payable value as well.
            /*
            let pending_reward = Psp22RewardRef::fund_rewards_and_add_winners(&self.reward_manager_address, era, winners)
                .map_err(|_| ContractError::CrossContractCallError2)?;
            */

            let pending_reward : PendingReward = ink_env::call::build_call::<Environment>()
                .call_type(
                    Call::new()
                        .callee(self.reward_manager_address)
                        .transferred_value(reward)
                )
                .exec_input(
                    ExecutionInput::new(Selector::new(FUND_REWARDS_AND_WINNERS_SELECTOR))
                        .push_arg(era)
                        .push_arg(winners)
                )
                .returns()
                .fire()
                .map_err(|_| ContractError::CrossContractCallError2)?;

            self.env().emit_event(RafleDone {
                contract: self.env().caller(),
                era,
                nb_winners: pending_reward.nb_winners,
                pending_rewards: pending_reward.given_reward,
            });
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), ContractError> {
            ink_env::set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        pub fn list_participants(&self, era: u32) -> Vec<(AccountId, Balance)> {
            ParticipantReaderRef::list_participants(&self.participants_oracle_address, era)
        }
    }

}
