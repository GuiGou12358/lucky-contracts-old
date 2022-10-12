use ink_env::Error;
use openbrush::traits::{AccountId, Balance, DefaultEnv};
//use ink_lang as ink;

use crate::impls::reward::psp22::psp22_reward::Internal;

/*
/// Event emitted when a user claim rewards
#[ink(event)]
pub struct ClaimedReward {
    #[ink(topic)]
    account: AccountId,
    amount: Balance,
}
*/

impl<T: DefaultEnv> Internal for T {

    default fn _transfer_to(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>{
        Self::env().transfer(to, amount)
    }

    /*
    default fn _emit_event(&mut self, _account: AccountId, _amount: Balance){
        // specialize inner the contract
    }
     */


}


