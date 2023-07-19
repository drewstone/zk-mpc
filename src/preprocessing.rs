mod ZKPoPK {

    use super::super::she::{Ciphertext, Encodedtext, Plaintext, PublicKey, SecretKey};
    use ark_bls12_377::Fr;
    use rand::{thread_rng, Rng};

    pub struct Parameters {
        V: i32,
        N: i32,
        tau: i32,
        sec: i32,
        d: i32,
        rho: i32,
        p: i128,
        q: i128, //she
    }

    impl Parameters {
        pub fn new(V: i32, N: i32, tau: i32, sec: i32, d: i32, rho: i32, p: i128, q: i128) -> Self {
            Self {
                V,
                N,
                tau,
                sec,
                d,
                rho,
                p,
                q,
            }
        }

        pub fn get_q(&self) -> i128 {
            self.q
        }
    }

    pub struct Instance {
        pk: PublicKey,
        c: Vec<Ciphertext>,
    }

    impl Instance {
        pub fn new(pk: PublicKey, c: Vec<Ciphertext>) -> Self {
            Self { pk, c }
        }
    }

    pub struct Witness {
        m: Vec<Plaintext>,
        x: Vec<Encodedtext>,
        r: Vec<Encodedtext>,
    }

    impl Witness {
        pub fn new(m: Vec<Plaintext>, x: &Vec<Encodedtext>, r: &Vec<Encodedtext>) -> Self {
            Self {
                m,
                x: x.clone(),
                r: r.clone(),
            }
        }
    }

    pub struct Proof {
        a: Vec<Ciphertext>,  //G^V
        z: Vec<Encodedtext>, //\mathbb{Z}^{N\times V}
        T: Vec<Encodedtext>, //\mathbb{Z}^{V\times d}
    }

    struct ZKPoPK {
        parameters: Parameters,
        instance: Instance,
        witness: Witness,
    }

    pub fn prove(parameters: &Parameters, witness: &Witness, instance: &Instance) -> Proof {
        // step 1
        let u: Vec<Encodedtext> = generate_u(parameters);
        let s: Vec<Encodedtext> = generate_s(parameters);

        let y: Vec<Encodedtext> = witness
            .m
            .iter()
            .zip(u.iter())
            .map(|(&ref m_i, &ref u_i)| m_i.encode() + u_i.clone())
            .collect();

        // step 2
        let a: Vec<Ciphertext> = y
            .iter()
            .zip(s.iter())
            .map(|(&ref y_i, s_i)| y_i.encrypt(&instance.pk, s_i))
            .collect();

        // step 3
        //let commit_a = commit(a);

        // step 4
        let e = hash(&a, &instance.c, parameters);

        // step 5
        let M_e: Vec<Vec<i128>> = generate_M_e(&e, parameters);

        let z: Vec<Encodedtext> = y
            .iter()
            .zip(M_e.iter())
            .map(|(&ref y_i, &ref row)| y_i.clone() + dot_product2(&row, &witness.x))
            .collect();

        let T: Vec<Encodedtext> = s
            .iter()
            .zip(M_e.iter())
            .map(|(&ref s_i, &ref row)| s_i.clone() + dot_product2(&row, &witness.r))
            .collect();

        Proof { a, z, T }
    }

    fn generate_u(parameters: &Parameters) -> Vec<Encodedtext> {
        //let mut rng = rand::thread_rng();
        //let u: Vec<i32> = (0..V).map(|_| rng.gen_range(0, upper_bound_y-m_i)).collect_vec();
        //u
        (0..parameters.V)
            .map(|_| Encodedtext::new(vec![0; parameters.N as usize], parameters.q))
            .collect()
    }

    fn generate_s(parameters: &Parameters) -> Vec<Encodedtext> {
        //let mut rng = rand::thread_rng();
        //let s: Vec<Vec<i32>> = (0..V).map(|_| (0..N).map(|_| rng.gen_range(0, upper_bound_s)).collect_vec()).collect_vec();
        //s
        (0..parameters.V)
            .map(|_| Encodedtext::new(vec![0; parameters.d as usize], parameters.q))
            .collect()
    }

    // outputがsec bitのハッシュ関数
    fn hash(a: &Vec<Ciphertext>, c: &Vec<Ciphertext>, parameters: &Parameters) -> Vec<bool> {
        //let rng = &mut thread_rng();
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(10);
        (0..parameters.sec).map(|_| rng.gen_bool(0.5)).collect()
    }

    fn generate_M_e(e: &Vec<bool>, parameters: &Parameters) -> Vec<Vec<i128>> {
        let Me: Vec<Vec<i128>> = (0..parameters.V)
            .map(|i| {
                (0..parameters.sec)
                    .map(|k| {
                        if i - k + 1 >= 1 && i - k < parameters.sec {
                            // e[(i - k + 1) as usize] as i32
                            e[(i - k) as usize] as i128
                        } else {
                            0
                        }
                    })
                    .collect()
            })
            .collect();
        Me
    }

    pub fn dummy_prove(parameters: &Parameters) -> Proof {
        let mut rng = thread_rng();
        let a = vec![Ciphertext::rand(2, parameters.q, &mut rng); 4];
        let z = vec![Encodedtext::new(vec![0, 0], parameters.q); 2];
        let T = vec![Encodedtext::new(vec![0, 0], parameters.q); 2];

        Proof { a, z, T }
    }

    pub fn verify(proof: &Proof, parameters: &Parameters, instance: &Instance) -> Result<(), ()> {
        // TODO: SHEを整えてから実装する
        // step 6
        let e = hash(&proof.a, &instance.c, parameters);
        let d: Vec<Ciphertext> = proof
            .z
            .iter()
            .zip(proof.T.iter())
            .map(|(&ref z_i, &ref t_i)| z_i.encrypt(&instance.pk, &t_i.clone()))
            .collect();

        // step 7
        let M_e: Vec<Vec<i128>> = generate_M_e(&e, parameters);

        let rhs: Vec<Ciphertext> = M_e
            .iter()
            .zip(proof.a.iter())
            .map(|(&ref row, &ref a_i)| a_i.clone() + dot_product3(&row, &instance.c, parameters))
            .collect();

        assert_eq!(d, rhs);

        let norm_z = proof.z.iter().map(|z_i| z_i.norm()).max().unwrap();

        assert!(norm_z < (128 * parameters.N * parameters.tau * parameters.sec.pow(2)) as i128);

        let norm_T = proof.T.iter().map(|t_i| t_i.norm()).max().unwrap();

        assert!(norm_T < (128 * parameters.d * parameters.rho * parameters.sec.pow(2)) as i128);

        Ok(())
    }

    fn dot_product(row: &Vec<i32>, x: &Vec<i32>) -> i32 {
        assert_eq!(row.len(), x.len(), "Vector dimensions must match!");

        let mut sum = 0;

        for i in 0..row.len() {
            sum += row[i] * x[i];
        }

        sum
    }

    fn dot_product2(row: &Vec<i128>, x: &Vec<Encodedtext>) -> Encodedtext {
        assert_eq!(row.len(), x.len(), "Vector dimensions must match!");

        let mut sum = Encodedtext::new(vec![0; x[0].get_degree()], x[0].get_q());

        for i in 0..row.len() {
            sum = sum + x[i].clone() * row[i];
        }

        sum
    }

    fn dot_product3(row: &Vec<i128>, c: &Vec<Ciphertext>, parameters: &Parameters) -> Ciphertext {
        assert_eq!(row.len(), c.len(), "Vector dimensions must match!");

        let rng = &mut thread_rng();

        let mut sum = Ciphertext::new(
            Encodedtext::new(vec![0; parameters.N as usize], parameters.q),
            Encodedtext::new(vec![0; parameters.N as usize], parameters.q),
            Encodedtext::new(vec![0; parameters.N as usize], parameters.q),
        );

        for i in 0..row.len() {
            sum = sum + c[i].clone() * row[i];
        }

        sum
    }

    #[test]
    fn test_proof() {
        let mut rng = thread_rng();
        // /let length = 10;
        let parameters = Parameters {
            V: 7,  // 2*sec-1
            N: 10, // degree
            tau: 2,
            sec: 4,
            d: 30, // 3*N
            rho: 2,
            p: 41,
            q: 83380292323641237751,
        };

        //let m = vec![Plaintext::new(vec![Fr::from(0); parameters.N as usize]); parameters.V as usize];
        let x: Vec<Encodedtext> = vec![
            Encodedtext::rand(parameters.N, parameters.q, &mut rng)
                .modulo_p(parameters.p);
            parameters.sec as usize
        ];
        let r: Vec<Encodedtext> =
            vec![
                Encodedtext::new(vec![0; parameters.d as usize], parameters.q);
                parameters.sec as usize
            ];

        let witness = Witness::new(
            vec![Plaintext::new(vec![Fr::from(0); parameters.N as usize]); parameters.V as usize],
            &x,
            &r,
        );

        let sk = SecretKey::generate(parameters.N, parameters.q, 3.2, &mut rng);

        let pk = sk.public_key_gen(parameters.N, parameters.p, parameters.q, 3.2, &mut rng);

        let c: Vec<Ciphertext> = x
            .iter()
            .zip(r.iter())
            .map(|(&ref x_i, &ref r_i)| x_i.encrypt(&pk, &r_i))
            .collect();
        let instance = Instance::new(pk, c);

        let proof = prove(&parameters, &witness, &instance);

        verify(&proof, &parameters, &instance).unwrap();
    }
}

use super::she;
use ark_bls12_377::Fr;
use ark_std::{test_rng, UniformRand};
use rand::{thread_rng, Rng};
type Ciphertext = i32;

enum CiphertextOpiton {
    NewCiphertext,
    NoNewCiphertext,
}

fn encode(f: Fr) -> Ciphertext {
    0
}

fn decode(c: Ciphertext) -> Fr {
    Fr::from(0)
}

fn reshare(e_m: Ciphertext, enc: CiphertextOpiton) -> (Vec<Fr>, Option<Ciphertext>) {
    let n = 3;

    // step 1
    let rng = &mut test_rng();

    let f: Vec<Fr> = (0..n).map(|_| Fr::rand(rng)).collect();

    // // step 2
    let e_f_vec: Vec<i32> = f.iter().map(|&f_i| encode(f_i)).collect();

    // step 3
    let parameters = ZKPoPK::Parameters::new(2, 2, 2, 2, 2, 2, 41, 10_i128.pow(9) + 7);

    let instance = ZKPoPK::Instance::new(
        she::PublicKey::new(
            she::Encodedtext::new(vec![0, 0], parameters.get_q()),
            she::Encodedtext::new(vec![0, 0], parameters.get_q()),
            41,
        ),
        vec![
            she::Ciphertext::new(
                she::Encodedtext::new(vec![0, 0], parameters.get_q()),
                she::Encodedtext::new(vec![0, 0], parameters.get_q()),
                she::Encodedtext::new(vec![0, 0], parameters.get_q()),
            );
            10
        ],
    );

    let witness = ZKPoPK::Witness::new(
        vec![she::Plaintext::new(vec![Fr::rand(rng); 2]); 2],
        &vec![she::Encodedtext::new(vec![0, 0], parameters.get_q()); 2],
        &vec![she::Encodedtext::new(vec![0, 0], parameters.get_q()); 2],
    );

    let dummy_proof = ZKPoPK::prove(&parameters, &witness, &instance);

    ZKPoPK::verify(&dummy_proof, &parameters, &instance).unwrap();

    // step4
    let e_f: Ciphertext = e_f_vec.iter().sum();
    let e_mf: Ciphertext = e_m + e_f;

    // step 5
    let mf = decode(e_mf);

    // step 6
    let mut m: Vec<Fr> = vec![Fr::from(0); n];
    m[0] = mf - f[0];

    for i in 1..n {
        m[i] = -f[i];
    }

    // step 7
    let e_m_new = encode(mf) - e_f;
    match enc {
        _NewCiphertext => (m, Some(e_m_new)),
        _NoNewCiphertext => (m, None),
    }
}

struct AngleShare {
    public_modifier: Fr,
    share: Vec<Fr>,
    MAC: Vec<Fr>,
}

fn angle(m_vec: Vec<Fr>, e_m: Ciphertext) -> AngleShare {
    let e_alpha = encode(Fr::from(0));
    let e_malpha: Ciphertext = e_m * e_alpha;
    let (gamma_vec, _) = reshare(e_malpha, CiphertextOpiton::NoNewCiphertext);

    AngleShare {
        public_modifier: Fr::from(0),
        share: m_vec,
        MAC: gamma_vec,
    }
}

type PrivateKey = Fr;
struct BracketShare {
    share: Vec<Fr>,
    MAC: Vec<(PrivateKey, Vec<Fr>)>,
}

fn bracket(m_vec: Vec<Fr>, e_m: Ciphertext) -> BracketShare {
    let n = 3;

    // step 1
    let beta_vec = vec![Fr::from(0); n];
    let e_beta_vec = vec![encode(Fr::from(0)); n];

    let e_gamma_vec: Vec<Ciphertext> = e_beta_vec.iter().map(|&e_beta_i| e_beta_i * e_m).collect();

    // step 2
    let gamma_vecvec: Vec<Vec<Fr>> = e_gamma_vec
        .iter()
        .map(|&e_gamma_i| {
            let (gamma_vec, _) = reshare(e_gamma_i, CiphertextOpiton::NoNewCiphertext);
            gamma_vec
        })
        .collect();

    // step 3
    // step 4
    let mac: Vec<(PrivateKey, Vec<Fr>)> = (1..n)
        .map(|i| (beta_vec[i], (1..n).map(|j| gamma_vecvec[j][i]).collect()))
        .collect();

    BracketShare {
        share: m_vec,
        MAC: mac,
    }
}

// // initialize
// fn initialize() {
//     let n = 3;
//     // step 1
//     pk = keygendec
//     // step 2
//     beta = (1..n).map(|_| Fr::rand(rng)).collect();

//     // step 3
//     alpha_vec = (0..n).map(|_| Fr::rand(rng)).collect();

//     // step 4
//     e_alpha_vec = alpha_vec.iter().map(|&alpha_i| encode(alpha_i)).collect();
//     e_beta_vec = beta.iter().map(|&beta_i| encode(beta_i)).collect();

//     // step 5
//     // ZKPoPK

//     // step 6
//     diag bracket
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_reshare() {
        let result = reshare(0, CiphertextOpiton::NewCiphertext);
    }

    // #[test]
    fn test_angle() {
        let m_vec = vec![Fr::from(0); 3];
        let e_m = 0;
        let result = angle(m_vec, e_m);
    }

    // #[test]
    fn test_bracket() {
        let m_vec = vec![Fr::from(0); 3];
        let e_m = 0;
        let result = bracket(m_vec, e_m);
    }
}
