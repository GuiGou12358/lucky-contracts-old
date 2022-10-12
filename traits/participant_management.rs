use ink_prelude::vec::Vec;
use openbrush::traits::AccountId;
use openbrush::traits::Balance;

#[openbrush::trait_definition]
pub trait ParticipantManagement {

    /// add a participant in the rafle for a given era
    /// a participant with a weigth higher than another participant will have normally more chance to be selected in the rafle
    /// weigth can represent the number of rafle ticked for this participant.
    /// weigth can also represent the amount staked in dAppStaking
    fn _add_participant(&mut self, era: u128, participant: AccountId, weigth: Balance);

    /// list all participant for a given era
    fn _list_participants(&self, era: u128) -> Vec<(AccountId, Balance)>;

}
