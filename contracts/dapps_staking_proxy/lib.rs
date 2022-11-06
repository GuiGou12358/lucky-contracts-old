#![cfg_attr(not(feature = "std"), no_std)]

#[openbrush::contract]
mod dapps_staking_proxy {
    use dapps_staking_extension::*;

    use rafle_lib::traits::participant_management::{ParticipantManagementError, ParticipantManagementRef};
    use rafle_lib::traits::reward::{psp22_reward, psp22_reward::*};

    use crate::dapps_staking_proxy::Error::{SubOverFlow, TransferError};

    #[ink(storage)]
    pub struct Contract {
        rafle_contract: AccountId,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(rafle_contract: AccountId) -> Self { Self {rafle_contract} }


        #[ink(message)]
        pub fn bond_and_stake(&mut self, value: Balance) -> Result<(), Error> {
            let contract = self.env().account_id();
            DappsStaking::bond_and_stake(contract, value)?;

            let caller = self.env().caller();
            let era = DappsStaking::read_current_era();
            ParticipantManagementRef::add_participant(&mut self.rafle_contract, era, caller, value)?;

            Ok(())
        }

        #[ink(message)]
        pub fn claim(&mut self, era: u32) -> Result<(), Error> {
            let contract = self.env().account_id();

            let balance_before = self.env().balance();
            DappsStaking::claim_dapp(contract, era)?;
            let balance_after = self.env().balance();
            let reward = balance_before.checked_sub(balance_after).ok_or(SubOverFlow)?;

            // transfer the amount
            if self.env().transfer(self.rafle_contract, reward).is_err(){
                return Err(TransferError);
            }
            Psp22RewardRef::fund_rewards(&mut self.rafle_contract, era)?;

            Ok(())
        }

        #[ink(message)]
        pub fn unbond_and_unstake(&mut self, value: Balance) -> Result<(), Error> {
            let contract = self.env().account_id();
            DappsStaking::unbond_and_unstake(contract, value)?;
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
        //AccessControlError(AccessControlError)
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


}
