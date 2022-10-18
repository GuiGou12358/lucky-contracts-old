use ink_prelude::vec::Vec;
use openbrush::modifiers;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::wrapper]
pub type ParticipantManagementRef = dyn ParticipantManagement;

#[openbrush::trait_definition]
pub trait ParticipantManagement {

    /// add a participant in the raffle for a given era
    /// a participant with a weight higher than another participant will have normally more chance to be selected in the raffle
    /// weight can represent the number of raffle tickets for this participant.
    /// weight can also represent the amount staked in dAppStaking, ...
    #[ink(message)]
    #[modifiers(only_role(PARTICIPANT_MANAGER))]
    fn add_participant(&mut self, era: u128, participant: AccountId, weight: Balance);

    /// list all participant for a given era
    fn _list_participants(&self, era: u128) -> Vec<(AccountId, Balance)>;

}
