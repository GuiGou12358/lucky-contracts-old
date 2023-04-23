use openbrush::contracts::access_control::AccessControlError;

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ParticipantFilterError {
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantFilterError
impl From<AccessControlError> for ParticipantFilterError {
    fn from(error: AccessControlError) -> Self {
        ParticipantFilterError::AccessControlError(error)
    }
}

