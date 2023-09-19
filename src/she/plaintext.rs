use std::ops::Mul;

use ark_std::UniformRand;
use num_bigint::BigUint;
use rand::Rng;

use super::{
    polynomial::{cyclotomic_moduli, interpolate},
    Encodedtext, Fq, Fr, SHEParameters, Texts,
};

pub type Plaintext = Fr;
pub type Plaintexts = Texts<Plaintext>;

pub trait Plaintextish {
    fn diagonalize(&self, length: usize) -> Plaintexts;
}

impl Plaintextish for Plaintext {
    fn diagonalize(&self, length: usize) -> Plaintexts {
        Plaintexts::from(&vec![*self; length])
    }
}

impl Plaintexts {
    pub fn rand<T: Rng>(params: &SHEParameters, rng: &mut T) -> Plaintexts {
        let res = (0..params.s).map(|_| Plaintext::rand(rng)).collect();
        Plaintexts { vals: res }
    }

    pub fn encode(&self, params: &SHEParameters) -> Encodedtext {
        let remainders = self.vals.clone();
        let moduli = cyclotomic_moduli(params.s);

        let result_vec = interpolate(&moduli, &remainders).unwrap();

        let result_vec_on_fq = result_vec
            .iter()
            .map(|&x| Fq::from(std::convert::Into::<BigUint>::into(x)))
            .collect::<Vec<Fq>>();

        Encodedtext {
            vals: result_vec_on_fq,
        }
    }
}

impl Mul<Plaintexts> for Plaintexts {
    type Output = Self;

    fn mul(self, other: Plaintexts) -> Self {
        let out_val = self
            .vals
            .iter()
            .zip(other.vals.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        Self { vals: out_val }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plaintext() {
        let pl_1: Plaintexts = Plaintexts::from_vec(vec![Fr::from(1), Fr::from(2)]);
        let pl_2: Plaintexts = Plaintexts::from_vec(vec![Fr::from(2), Fr::from(3)]);
        let pl_add = pl_1.clone() + pl_2.clone();
        let pl_add_expected = Plaintexts::from(&[Fr::from(1 + 2), Fr::from(2 + 3)]);
        assert_eq!(pl_add, pl_add_expected);

        let pl_vec: Vec<Plaintexts> = vec![pl_1.clone(), pl_2.clone()];
        let pl_sum: Plaintexts = pl_vec.iter().cloned().sum();
        assert_eq!(pl_sum, pl_add_expected);
    }
}
