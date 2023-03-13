use ink::env::hash::{Keccak256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::traits::Storage;
use openbrush::contracts::access_control::{access_control, RoleType};

pub use crate::traits::random_generator::{
    *,
    RandomGeneratorError::*,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
pub const RANDOM_GENERATOR_CONSUMER: RoleType = ink::selector_id!("RANDOM_GENERATOR_CONSUMER");
pub const RANDOM_GENERATOR_MANAGER: RoleType = ink::selector_id!("RANDOM_GENERATOR_MANAGER");

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    salt: u64,
}

impl<T> RandomGenerator for T
    where
        T: Storage<Data>,
        T: Storage<access_control::Data>,
{
    
    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_CONSUMER))]
    default fn get_random_number(&mut self, min: u128, max: u128) -> Result<u128, RandomGeneratorError> {
        let seed = Self::env().block_timestamp();
        let salt = self.data::<Data>().salt;
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&seed.to_be_bytes());
        input.extend_from_slice(&salt.to_be_bytes());
        let mut output = <Keccak256 as HashOutput>::Type::default();
        ink::env::hash_bytes::<Keccak256>(&input, &mut output);
        self.data::<Data>().salt = salt + 1;


        let a = output[0] as u128;

        //(a  as u32) * (max - min) / (u32::MAX) + min
        let b = max.checked_sub(min).ok_or(SubOverFlow)?;
        let c = a.checked_mul(b).ok_or(MulOverFlow)?;
        let d = c.checked_div(u8::MAX as u128).ok_or(DivByZero)?;
        let e = d.checked_add(min).ok_or(AddOverFlow)?;

        ink::env::debug_println!("random {}", e);

        Ok(e)
    }


    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_MANAGER))]
    default fn get_salt(&mut self) -> Result<u64, RandomGeneratorError>{
        Ok(self.data::<Data>().salt)
    }

    #[openbrush::modifiers(access_control::only_role(RANDOM_GENERATOR_MANAGER))]
    default fn set_salt(&mut self, salt: u64) -> Result<(), RandomGeneratorError>{
        self.data::<Data>().salt = salt;
        Ok(())
    }


}
