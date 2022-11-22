use ink_env::debug_println;
use openbrush::traits::{AccountId, DefaultEnv};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::RngCore;
use rand_chacha::rand_core::SeedableRng;

pub use crate::traits::random_generator::{
    RandomGenerator,
    RandomGeneratorError,
    RandomGeneratorError::*,
};

impl<T: DefaultEnv> RandomGenerator for T {

    default fn get_random_number(
        &self, min: u128, max: u128, account: AccountId
    ) -> Result<u128, RandomGeneratorError> {
        
        let random_seed = Self::env().random(account.as_ref());
        let mut seed_converted: [u8; 32] = Default::default();
        seed_converted.copy_from_slice(random_seed.0.as_ref());
        let mut rng = ChaChaRng::from_seed(seed_converted);

        //(a  as u32) * (max - min) / (u32::MAX) + min
        let a = rng.next_u32() as u128;
        let b = max.checked_sub(min).ok_or(SubOverFlow)?;
        let c = a.checked_mul(b).ok_or(MulOverFlow)?;
        let d = c.checked_div(u32::MAX as u128).ok_or(DivByZero)?;
        let e = d.checked_add(min).ok_or(AddOverFlow)?;
        debug_println!("a: {} - b: {} - c: {} - d: {} - e: {}", a, b, c, d, e);

        Ok(e)
    }

}
