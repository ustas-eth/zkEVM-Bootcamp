use ark_bls12_381::Fq as F; // Prime Field
use ark_ff::{Field, PrimeField};
use ark_std::UniformRand;

fn main() {
    let mut rng = rand::thread_rng();
    let modulus = <F as PrimeField>::MODULUS;

    // select a random value from the field
    let a = F::rand(&mut rng);

    // show that 1 + 1 = 2
    assert_eq!(F::from(1) + F::from(1), F::from(2));

    // show that the multiplicative inverse of a number multipled by itself equals one.
    assert_eq!(a.inverse().unwrap() * a, F::from(1));

    // show that a value raised to the power of the modulus is equal to itself
    // use the `pow` function to raise to a power
    assert_eq!(a.pow(modulus), a);
}
