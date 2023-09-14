pub mod ciphertext;
pub mod encodedtext;
pub mod plaintext;
pub mod texts;
pub use ark_bls12_377::{Fr, FrParameters};
use ark_ff::FftParameters;
pub use ciphertext::Ciphertext;
pub use encodedtext::Encodedtext;
pub use texts::Texts;

pub use ark_ff::{FftField, Field, FpParameters};
pub use ark_mnt4_753::{Fq, FqParameters};
use ark_poly::{polynomial::univariate::DensePolynomial, univariate::DenseOrSparsePolynomial};
use ark_std::log2;
use num_bigint::BigUint;
pub use plaintext::{Plaintext, Plaintextish, Plaintexts};
use rand::Rng;
use rand_distr::{Distribution, Normal};

pub struct SHEParameters {
    // length of Plaintext
    s: usize,
    // degree of polynomial (length of Encodedtext) should be same with lenght of Plaintext
    n: usize,
    p: BigUint,
    q: BigUint,
    std_dev: f64,
}
#[derive(Clone)]
pub struct SecretKey {
    s: Encodedtext,
}

#[derive(Clone)]
pub struct PublicKey {
    a: Encodedtext,
    b: Encodedtext,
}

// impls SHEParameter {
impl SHEParameters {
    pub fn new(s: usize, n: usize, p: BigUint, q: BigUint, std_dev: f64) -> SHEParameters {
        SHEParameters {
            s,
            n,
            p,
            q,
            std_dev,
        }
    }
}

impl SecretKey {
    fn new(sk: Encodedtext) -> Self {
        Self { s: sk }
    }

    pub fn generate<T: Rng>(she_params: &SHEParameters, rng: &mut T) -> Self {
        let s = get_gaussian(she_params, she_params.n, rng);
        Self { s }
    }

    pub fn public_key_gen<T: Rng>(&self, she_params: &SHEParameters, rng: &mut T) -> PublicKey {
        let s = self.s.clone();
        let a = Encodedtext::rand(she_params, rng);

        let e = get_gaussian(she_params, she_params.n, rng);
        let b = a.clone() * s + e * she_params.p.clone();
        PublicKey { a, b }
    }
}

impl PublicKey {
    pub fn new(a: Encodedtext, b: Encodedtext) -> Self {
        Self { a, b }
    }
}

pub fn get_gaussian<T: Rng>(
    she_params: &SHEParameters,
    dimension: usize,
    rng: &mut T,
) -> Encodedtext {
    let gaussian = Normal::new(0.0, she_params.std_dev).unwrap(); // ?
    let val: Vec<Fq> = (0..dimension)
        .map(|_| Fq::from(gaussian.sample(rng).abs() as u128))
        .collect();
    Encodedtext::from_vec(val)
}

fn substitute(polynomial: &[Fr], variable: &Fr) -> Fr {
    let mut result = Fr::from(0);
    for (i, coefficient) in polynomial.iter().enumerate() {
        result += *coefficient * variable.pow([i as u64]);
    }
    result
}

fn cyclotomic_moduli(length: usize) -> Vec<Fr> {
    // moduli: lengthは本来N-1だが、sで切り捨て
    // N-1個の根は、円分多項式Phi_N(X) on Fpの根である

    // N=sである。N * 2=mである。mは2の冪である。m=2^kであるとき(ただし、1<=k<47)、moduliは、TWO_ADIC_ROOT_OF_UNITY^{2^(47-k)}のi乗である。

    let k = log2(length * 2);
    let m_root_of_unity = Fr::two_adic_root_of_unity().pow([2_u64.pow(47 - k)]);
    let mut moduli = Vec::new();
    for i in 0..length {
        moduli.push(m_root_of_unity.pow([(2 * i + 1) as u64]));
    }

    moduli
}

#[cfg(test)]
mod tests {
    use num_traits::One;
    use rand::thread_rng;

    use super::*;

    fn is_power_of_two(n: usize) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    #[test]
    fn test_cyclotomic_moduli() {
        // Set the parameters for this instantiation of BV11
        let std_dev = 3.2; // Standard deviation for generating the error
        let s = 64;
        let p: BigUint = FrParameters::MODULUS.into();
        let q: BigUint = FqParameters::MODULUS.into();
        let degree = s;

        let she_params = SHEParameters::new(s, degree, p.clone(), q.clone(), std_dev);

        let res = cyclotomic_moduli(she_params.n);

        for v in res {
            assert_eq!(Fr::one(), v.pow([(s * 2) as u64]));
        }
        println!("hello");
    }

    #[test]
    fn test() {
        let mut rng = thread_rng();

        // // Set the parameters for this instantiation of BV11
        let std_dev = 3.2; // Standard deviation for generating the error
        let s = 64; // must be 2^k for 0 <= k <= 46
        assert!(is_power_of_two(s));
        assert!(s as u64 <= 2_u64.pow(46));
        let p: BigUint = FrParameters::MODULUS.into();
        let q: BigUint = FqParameters::MODULUS.into();
        let degree = s; // degree = length = N     Degree of polynomials used for encoding and encrypting messages

        let she_params = SHEParameters::new(s, degree, p, q, std_dev);

        // // Generate secret, public keys using the given parameters
        let secret_key = SecretKey::generate(&she_params, &mut rng);
        let public_key = secret_key.public_key_gen(&she_params, &mut rng);

        let pt = Plaintexts::rand(&she_params, &mut rng);
        let pt_2 = Plaintexts::rand(&she_params, &mut rng);
        let pt_3 = Plaintexts::rand(&she_params, &mut rng);

        let et = pt.encode(&she_params);
        let et_2 = pt_2.encode(&she_params);
        let et_3 = pt_3.encode(&she_params);

        let r = get_gaussian(&she_params, degree * 3, &mut rng);
        let r_2 = get_gaussian(&she_params, degree * 3, &mut rng);
        let r_3 = get_gaussian(&she_params, degree * 3, &mut rng);

        let ct = Ciphertext::encrypt_from(&et, &public_key, &r, &she_params);
        let ct_2 = Ciphertext::encrypt_from(&et_2, &public_key, &r_2, &she_params);
        let ct_3 = Ciphertext::encrypt_from(&et_3, &public_key, &r_3, &she_params);

        // Plain -> Encoded  -> Plain, uni text
        let dect = et.decode(&she_params);

        assert_eq!(pt, dect);

        // Plain -> Encoded -> Cipher -> Encoded -> Plain, uni text
        let dect = ct.decrypt(&secret_key);
        let decrypted_decoded_ct = dect.decode(&she_params);

        assert_eq!(pt, decrypted_decoded_ct);

        // Plain -> Encoded -> Cipher -> Encoded -> Plain, sum of 2 texts
        let expr_ct = ct.clone() + ct_2.clone(); // + ct_3.clone();
        let dect = expr_ct.decrypt(&secret_key).decode(&she_params);
        let expected_pt = pt.clone() + pt_2.clone();

        assert_eq!(expected_pt, dect);

        // Plain -> Encoded -> Plain, multiplication of 2 texts
        let expr_et = et.clone() * et_2.clone(); // + ct_3.clone();
        let dect = expr_et.decode(&she_params);
        let expected_pt = pt.clone() * pt_2.clone();

        assert_eq!(expected_pt, dect);

        // Plain -> Encoded -> Cipher -> Encoded -> Plain, multiplication of 2 texts
        let expr_ct = ct.clone() * ct_2.clone(); // + ct_3.clone();
        let dect = expr_ct.decrypt(&secret_key).decode(&she_params);
        let expected_pt = pt.clone() * pt_2.clone();

        assert_eq!(expected_pt, dect);

        // Plain -> Encoded -> Cipher -> Encoded -> Plain, multiplication and addition of 3 texts
        let expr_ct = ct.clone() * ct_2.clone() + ct_3.clone();
        let dect = expr_ct.decrypt(&secret_key).decode(&she_params);
        let expected_pt = pt.clone() * pt_2.clone() + pt_3.clone();

        assert_eq!(expected_pt, dect);
    }
}
