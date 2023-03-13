use crate::traits::random_generator::RandomGeneratorError;

pub trait Random {

    /// generate a random number between min and max values.
    fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomError> ;

}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomError {
    MissingAddress,
    RandomGeneratorError(RandomGeneratorError),
}

/// convertor from RandomError to RandomGeneratorError
impl From<RandomGeneratorError> for RandomError {
    fn from(error: RandomGeneratorError) -> Self {
        RandomError::RandomGeneratorError(error)
    }
}