#![cfg_attr(not(feature = "std"), no_std)]

#[openbrush::contract]
mod dapps_staking_proxy {
    use dapps_staking_extension::*;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::contracts::ownable;
    use openbrush::contracts::ownable::*;
    use openbrush::modifiers;
    use openbrush::traits::Storage;

    use rafle::traits::participant_management::{ParticipantManagementError, /*ParticipantManagementRef*/};
    use rafle::traits::reward::{psp22_reward, psp22_reward::*};

    use crate::dapps_staking_proxy::Error::{SubOverFlow, TransferError, UpgradeError};

    #[ink(storage)]
    #[derive(Default, Storage, SpreadAllocate)]
    pub struct Contract {
        #[storage_field]
        ownable: ownable::Data,
        rafle_contract: AccountId,
    }

    //impl Ownable for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(rafle_contract: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance.ownable._init_with_owner(caller);
                instance.rafle_contract = rafle_contract;
            })
        }


        #[ink(message)]
        pub fn bond_and_stake(&mut self, value: Balance) -> Result<(), Error> {
            let contract = self.env().account_id();
            DappsStaking::bond_and_stake(contract, value)?;

            /*
            let caller = self.env().caller();
            let era = DappsStaking::read_current_era();
            ParticipantManagementRef::add_participant(&mut self.rafle_contract, era, caller, value)?;
             */

            Ok(())
        }

        #[ink(message)]
        pub fn claim(&mut self, era: u32) -> Result<(), Error> {
            let contract = self.env().account_id();

            let balance_before = self.env().balance();
            DappsStaking::claim_dapp(contract, era)?;
            let balance_after = self.env().balance();
            let reward = balance_before.checked_sub(balance_after).ok_or(SubOverFlow)?;

            let reward2 = if reward > 0 { reward } else {  reward.checked_add(2222).ok_or(SubOverFlow)? }; // TODO remove it

            // transfer the amount

            // use only in test
            //ink_env::pay_with_call!(Psp22RewardRef::fund_rewards(&mut self.rafle_contract, era), reward);

            self.env().transfer(self.rafle_contract, reward2).map_err(|_| TransferError)?;
            // TODO better to use payable method but how call it!
            //Psp22RewardRef::fund_rewards(&mut self.rafle_contract, era)?;

            Psp22RewardRef::fund_rewards_after_transfer(&mut self.rafle_contract, era, reward2)?;

            Ok(())
        }

        #[ink(message)]
        pub fn unbond_and_unstake(&mut self, value: Balance) -> Result<(), Error> {
            let contract = self.env().account_id();
            DappsStaking::unbond_and_unstake(contract, value)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn upgrade_contract(&mut self, new_code_hash: [u8; 32]) -> Result<(), Error> {
            ink_env::set_code_hash(&new_code_hash).map_err(|_| UpgradeError)?;
            Ok(())
        }

    }

    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TransferError,
        SubOverFlow,
        DSError(dapps_staking_extension::DSError),
        RewardError(psp22_reward::RewardError),
        ParticipantManagementError(ParticipantManagementError),
        UpgradeError,
        OwnableError(ownable::OwnableError),
    }

    /// convertor from DSError to Error
    impl From<DSError> for Error {
        fn from(error: DSError) -> Self {
            Error::DSError(error)
        }
    }

    /// convertor from RewardError to Error
    impl From<RewardError> for Error {
        fn from(error: RewardError) -> Self {
            Error::RewardError(error)
        }
    }

    /// convertor from ParticipantManagementError to Error
    impl From<ParticipantManagementError> for Error {
        fn from(error: ParticipantManagementError) -> Self {
            Error::ParticipantManagementError(error)
        }
    }

    /// convertor from OwnableError to Error
    impl From<OwnableError> for Error {
        fn from(error: OwnableError) -> Self {
            Error::OwnableError(error)
        }
    }

}
