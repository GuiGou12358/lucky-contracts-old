#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
#[openbrush::contract]
pub mod random_generator {
    use openbrush::contracts::access_control::{*, access_control};
    use openbrush::traits::Storage;

    use lucky::impls::{
        *,
        random_generator::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        random_generator: random_generator::Data,
        #[storage_field]
        access: access_control::Data,
    }

    impl RandomGenerator for Contract {}
    impl AccessControl for Contract{}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.random_generator = random_generator::Data::default();
            let caller = instance.env().caller();
            instance._init_with_admin(caller);
            instance.grant_role(RANDOM_GENERATOR_CONSUMER, caller).expect("Should grant the role RANDOM_GENERATOR_CONSUMER");
            instance.grant_role(RANDOM_GENERATOR_MANAGER, caller).expect("Should grant the role RANDOM_GENERATOR_MANAGER");
            instance
        }

    }

    mod tests {

        use super::*;

        #[ink::test]
        fn test_get_pseudo_random() {
            let mut contract = Contract::new();
            for max_value in 0..=100 {
                for min_value in 0..=max_value {
                    let result = contract.get_random_number(min_value, max_value).unwrap();
                    assert!(result >= min_value);
                    assert!(result <= max_value);
                }
            }
        }
    }
}