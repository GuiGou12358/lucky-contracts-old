use ink::prelude::vec::Vec;
use ink::storage::Lazy;
use openbrush::contracts::access_control::{access_control, RoleType};
use openbrush::traits::AccountId;
use openbrush::traits::Balance;
use openbrush::traits::Storage;

pub use crate::traits::participant_manager::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const PARTICIPANT_MANAGER: RoleType = ink::selector_id!("PARTICIPANT_MANAGER");
pub const MAX_PART_BY_VEC: usize = 300;
pub const MAX_PART: usize = MAX_PART_BY_VEC * 6;

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    nb_participants: u16,
    /// participants
    /// to not reach max capacity size, we will split the participants in many vectors (max 300 participants by vector)
    participants_1: Lazy<Vec<Participant>>,
    total_value_1: Balance,
    participants_2: Lazy<Vec<Participant>>,
    total_value_2: Balance,
    participants_3: Lazy<Vec<Participant>>,
    total_value_3: Balance,
    participants_4: Lazy<Vec<Participant>>,
    total_value_4: Balance,
    participants_5: Lazy<Vec<Participant>>,
    total_value_5: Balance,
    participants_6: Lazy<Vec<Participant>>,
    total_value_6: Balance,
}

fn push_participants(index: usize, src: &Vec<(AccountId, Balance)>, dest: &mut Vec<Participant>) -> (usize, Balance)  {

    let mut total_value = Balance::default();

    let remaining_len = MAX_PART_BY_VEC - dest.len();
    let mut end_index = index + remaining_len;
    if end_index >= src.len(){
        end_index = src.len();
    }

    for (account, value) in  src[index..end_index].iter() {
        dest.push(Participant{account: *account, value: *value});
        total_value += *value;
    }

    (end_index - index, total_value)
}

/// Iterate on the participants, sum the values,
/// and return the participant if the sum is superior to the given weight
fn select_winner_matching_value(
    participants: &Vec<Participant>,
    selected_value : Balance
) -> Option<AccountId> {
    let mut total_value = 0;
    for participant in participants {
        total_value += participant.value;
        if total_value >= selected_value {
            return Some(participant.account);
        }
    }
    None
}


impl<T> ParticipantManager for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{

    default fn get_nb_participants(&self) -> u16 {
        self.data::<Data>().nb_participants
    }

    default fn get_total_value(&self) -> Balance {
        self.data::<Data>().total_value_1 + self.data::<Data>().total_value_2
            + self.data::<Data>().total_value_3 + self.data::<Data>().total_value_4
            + self.data::<Data>().total_value_5 + self.data::<Data>().total_value_6
    }

    default fn get_participant(&self, value: Balance) -> Option<AccountId> {

        let mut to_value= self.data::<Data>().total_value_1;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_1.get_or_default(),
                value
            );
        }
        let mut from_value = to_value;
        to_value += self.data::<Data>().total_value_2;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_2.get_or_default(),
                value - from_value
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_3;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_3.get_or_default(),
                value - from_value
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_4;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_4.get_or_default(),
                value - from_value
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_5;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_5.get_or_default(),
                value - from_value
            );
        }
        from_value = to_value;
        to_value += self.data::<Data>().total_value_6;
        if value <= to_value {
            return select_winner_matching_value(
                &self.data::<Data>().participants_6.get_or_default(),
                value - from_value
            );
        }
        None
    }

    default fn get_participants(&self, page: u8) -> Result<Vec<Participant>, ParticipantManagerError> {

        let participants;
        if page == 0 {
            participants = Vec::new();
        } else if page == 1 {
            participants = self.data::<Data>().participants_1.get_or_default();
        } else if page == 2 {
            participants = self.data::<Data>().participants_2.get_or_default();
        } else if page == 3 {
            participants = self.data::<Data>().participants_3.get_or_default();
        } else if page == 4 {
            participants = self.data::<Data>().participants_4.get_or_default();
        } else if page == 5 {
            participants = self.data::<Data>().participants_5.get_or_default();
        } else if page == 6 {
            participants = self.data::<Data>().participants_6.get_or_default();
        } else {
            return Err(ParticipantManagerError::PageNotFound);
        }

        Ok(participants)
    }

    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_MANAGER))]
    default fn add_participants(&mut self, participants: Vec<(AccountId, Balance)>) -> Result<(), ParticipantManagerError> {

        let mut nb_participants = self.data::<Data>().nb_participants as usize;
        let mut index = 0;

        while index < participants.len() {

            let inserted_participants;

            if nb_participants < MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_1.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_1 +=  total_value;
                self.data::<Data>().participants_1.set(&p);
            } else if nb_participants < 2 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_2.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_2 +=  total_value;
                self.data::<Data>().participants_2.set(&p);
            } else if nb_participants < 3 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_3.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_3 +=  total_value;
                self.data::<Data>().participants_3.set(&p);
            } else if nb_participants < 4 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_4.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_4 +=  total_value;
                self.data::<Data>().participants_4.set(&p);
            } else if nb_participants < 5 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_5.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_5 +=  total_value;
                self.data::<Data>().participants_5.set(&p);
            } else if nb_participants < 6 * MAX_PART_BY_VEC {
                let mut p = self.data::<Data>().participants_6.get_or_default();
                let (nb_pushed, total_value) = push_participants(index, &participants, &mut p);
                inserted_participants = nb_pushed;
                self.data::<Data>().total_value_6 +=  total_value;
                self.data::<Data>().participants_6.set(&p);
            } else {
                return Err(ParticipantManagerError::MaxSizeExceeded);
            }
            nb_participants = nb_participants + inserted_participants;
            index = index + inserted_participants;

        }
        self.data::<Data>().nb_participants = nb_participants as u16;
        Ok(())
    }

    #[openbrush::modifiers(access_control::only_role(PARTICIPANT_MANAGER))]
    default fn clear_data(&mut self) -> Result<(), ParticipantManagerError> {

        let nb_participants = self.data::<Data>().nb_participants as usize;
        
        if nb_participants > 0 {
            let mut p = self.data::<Data>().participants_1.get_or_default();
            p.clear();
            self.data::<Data>().participants_1.set(&p);
        } 
        if nb_participants > MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_2.get_or_default();
            p.clear();
            self.data::<Data>().participants_2.set(&p);
        } 
        if nb_participants > 2 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_3.get_or_default();
            p.clear();
            self.data::<Data>().participants_3.set(&p);
        } 
        if nb_participants > 3 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_4.get_or_default();
            p.clear();
            self.data::<Data>().participants_4.set(&p);
        } 
        if nb_participants > 4 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_5.get_or_default();
            p.clear();
            self.data::<Data>().participants_5.set(&p);
        } 
        if nb_participants > 5 * MAX_PART_BY_VEC {
            let mut p = self.data::<Data>().participants_6.get_or_default();
            p.clear();
            self.data::<Data>().participants_6.set(&p);
        } 

        self.data::<Data>().nb_participants = 0;
        self.data::<Data>().total_value_1 = 0;
        self.data::<Data>().total_value_2 = 0;
        self.data::<Data>().total_value_3 = 0;
        self.data::<Data>().total_value_4 = 0;
        self.data::<Data>().total_value_5 = 0;
        self.data::<Data>().total_value_6 = 0;

        Ok(())
    }

}
