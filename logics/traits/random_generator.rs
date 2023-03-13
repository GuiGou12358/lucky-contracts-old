use openbrush::contracts::access_control::AccessControlError;

#[openbrush::wrapper]
pub type RandomGeneratorRef = dyn RandomGenerator;


#[openbrush::trait_definition]
pub trait RandomGenerator {

    /// generate a random number between min and max values.
    #[ink(message)]
    fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomGeneratorError> ;

    /// get the current salt used for randomness
    #[ink(message)]
    fn get_salt(&mut self) -> Result<u64, RandomGeneratorError> ;

    /// Set the current salt used for randomness
    #[ink(message)]
    fn set_salt(&mut self, salt: u64) -> Result<(), RandomGeneratorError>;

}


#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomGeneratorError {
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    SubOverFlow,
    MissingAddress,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RandomGeneratorError
impl From<AccessControlError> for RandomGeneratorError {
    fn from(error: AccessControlError) -> Self {
        RandomGeneratorError::AccessControlError(error)
    }
}