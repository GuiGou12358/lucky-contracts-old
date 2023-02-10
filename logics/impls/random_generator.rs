use openbrush::traits::{AccountId, DefaultEnv};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::RngCore;
use rand_chacha::rand_core::SeedableRng;
use ink::env::hash::{Sha2x256, HashOutput};

pub use crate::traits::random_generator::{
    RandomGenerator,
    RandomGeneratorError,
    RandomGeneratorError::*,
};

impl<T: DefaultEnv> RandomGenerator for T {

    default fn get_random_number(
        &self, min: u128, max: u128, account: AccountId
    ) -> Result<u128, RandomGeneratorError> {
        
        // ink_env::random_function() have been removed in ink!v4
        // waiting DIA implements a Wasm Oracle to generate randomness 
        // for the hackaton, use quick and dirty solution :(

        let mut seed_converted: [u8; 32] = Default::default();

        /*
        let random_seed = Self::env().random(account.as_ref());
        seed_converted.copy_from_slice(random_seed.0.as_ref());
        */

        seed_converted.copy_from_slice(account.as_ref());
        let mut quotient = Self::env().block_number();

        let mut i = 31;

        loop {
            let rest = quotient.checked_rem(10).ok_or(MulOverFlow)? as u8;
            seed_converted[i] = rest;
            quotient = quotient.checked_div(10).ok_or(DivByZero)?;
            if quotient == 0 {
                break;
            }
            i = i.checked_sub(1).ok_or(SubOverFlow)?;
        }

        let mut rng = ChaChaRng::from_seed(seed_converted);
        let a = rng.next_u32() as u128;

        //(a  as u32) * (max - min) / (u32::MAX) + min
        let b = max.checked_sub(min).ok_or(SubOverFlow)?;
        let c = a.checked_mul(b).ok_or(MulOverFlow)?;
        let d = c.checked_div(u32::MAX as u128).ok_or(DivByZero)?;
        let e = d.checked_add(min).ok_or(AddOverFlow)?;
        

        let input: &[u8] = &[1, 2, 3];
        let mut output = <Sha2x256 as HashOutput>::Type::default();
        let hash = ink::env::hash_bytes::<Sha2x256>(input, &mut output);
        
        ink::env::debug_println!("hash {:?}", hash);

        Ok(e)
    }

}
