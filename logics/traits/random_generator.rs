use openbrush::traits::AccountId;

pub trait RandomGenerator {

    /// generate a random number between min and max values.
    /// The subject can be used to further randomize the number.
    fn get_random_number(&self, min: u128, max: u128, subject: AccountId) -> Result<u128, RandomGeneratorError> ;

}


#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomGeneratorError {
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    SubOverFlow,
}