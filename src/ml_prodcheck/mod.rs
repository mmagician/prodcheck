//! Grand product relations protocol for multilinear extension

use ark_ec::pairing::Pairing;
use ark_linear_sumcheck::{
    ml_sumcheck::{
        data_structures::ListOfProductsOfPolynomials,
        protocol::{prover::ProverMsg, verifier::SubClaim, IPForMLSumcheck, PolynomialInfo},
    },
    rng::{Blake2s512Rng, FeedableRNG},
    Error,
};
use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_std::{marker::PhantomData, rc::Rc, vec::Vec, One, UniformRand, Zero};

pub struct MLProdcheck<E: Pairing>(#[doc(hidden)] PhantomData<E>);

/// Proof generated by prover
pub struct Proof<E: Pairing> {
    pub sumcheck_proof: Vec<ProverMsg<E::ScalarField>>,
    pub comm_v: E::G1,
    pub comm_f: E::G1,
    pub claimed_value: E::ScalarField,
}

impl<E: Pairing> MLProdcheck<E> {
    /// Generate proof of the prod of polynomial v over {0,1}^`num_vars`
    pub fn prove(v: &DenseMultilinearExtension<E::ScalarField>) -> Result<Proof<E>, Error> {
        let mut fs_rng = Blake2s512Rng::setup();
        // fs_rng.feed(&polynomial.info())?;

        // 1. Commit to v
        let comm_v = E::G1::default();

        // 2. Compute f from v
        let (f, p) = compute_f::<E>(v);

        // 3. Commit to f
        let comm_f = E::G1::default();

        // 4. use the FS randomness to sample a random tau
        let tau: Vec<E::ScalarField> = (0..v.num_vars())
            .map(|_| E::ScalarField::rand(&mut fs_rng))
            .collect();

        // 5. Compute MLE g of f:
        //  G(x, t) = eq(t, x) * f(x,0)*f(x,1) - f(1,x)
        // g(t) = \sum_{x \in {0,1}^n} G(x, t)
        let g = compute_G::<E>(&f, &tau);
        let mut polynomial = ListOfProductsOfPolynomials::new(v.num_vars());
        polynomial.add_product(vec![Rc::new(g)], E::ScalarField::one());

        // 7. Provide openings and proofs of f(0, gamma) = g(gamma)

        // 8. Provide openings and proofs of f(1, ..., 1, 0) = P

        // 5. Run sum check protocol on g

        let mut prover_state = IPForMLSumcheck::prover_init(&polynomial);
        let mut verifier_msg = None;
        let mut prover_msgs = Vec::with_capacity(polynomial.num_variables);
        for _ in 0..polynomial.num_variables {
            let prover_msg = IPForMLSumcheck::prove_round(&mut prover_state, &verifier_msg);
            fs_rng.feed(&prover_msg)?;
            prover_msgs.push(prover_msg);
            verifier_msg = Some(IPForMLSumcheck::sample_round(&mut fs_rng));
        }

        Ok(Proof {
            sumcheck_proof: prover_msgs,
            comm_v,
            comm_f,
            claimed_value: p,
        })
    }

    /// Verify the claimed prod using the proof
    pub fn verify(
        polynomial_info: &PolynomialInfo,
        proof: &Proof<E>,
    ) -> Result<SubClaim<E::ScalarField>, Error> {
        let mut fs_rng = Blake2s512Rng::setup();
        fs_rng.feed(polynomial_info)?;
        let mut verifier_state = IPForMLSumcheck::verifier_init(polynomial_info);
        for i in 0..polynomial_info.num_variables {
            let prover_msg = proof.sumcheck_proof.get(i).expect("proof is incomplete");
            fs_rng.feed(prover_msg)?;
            let _verifier_msg = IPForMLSumcheck::verify_round(
                (*prover_msg).clone(),
                &mut verifier_state,
                &mut fs_rng,
            );
        }

        IPForMLSumcheck::check_and_generate_subclaim(verifier_state, proof.claimed_value)
    }
}

// Compute MLE f of v in v.num_vars() + 1 variables
pub fn compute_f<E: Pairing>(
    v: &DenseMultilinearExtension<E::ScalarField>,
) -> (DenseMultilinearExtension<E::ScalarField>, E::ScalarField) {
    let s = v.num_vars();
    let mut evals = vec![E::ScalarField::zero(); 1 << (s + 1)];

    // case where first element is 0:
    for x in 0usize..(1 << s) {
        let (le_x, _) = x.reverse_bits().overflowing_shr(usize::BITS - (s as u32));

        let f_index = le_x << 1;
        evals[f_index] = v.evaluations[le_x];
    }

    // case where first element is 1:
    for x in 0usize..(1 << s) {
        let (le_x, _) = x.reverse_bits().overflowing_shr(usize::BITS - (s as u32));
        let f_index = (le_x << 1) + 1;

        let f_index_l = le_x;
        let f_index_r = le_x + (1 << s);

        evals[f_index] = evals[f_index_l] * &evals[f_index_r];
    }

    // Extract the claim P. It's at index f(1,1,1,...0), i.e. in LE b0111..., or (1<<s) - 1
    let index = (1 << s) - 1;
    let p = evals[index];

    let f = DenseMultilinearExtension::from_evaluations_vec(s + 1, evals);
    (f, p)
}

// Compute G_t(x) = eq(x,t) * (f(1,x) - f(x,0)*f(x,1))
#[allow(non_snake_case)]
pub fn compute_G<E: Pairing>(
    f: &DenseMultilinearExtension<E::ScalarField>,
    t: &Vec<E::ScalarField>,
) -> DenseMultilinearExtension<E::ScalarField> {
    let s = f.num_vars() - 1;
    let mut fs_evals = vec![E::ScalarField::zero(); 1 << (s)];
    let eq = compute_mle_eq::<E>(t, s);

    for x in 0usize..(1 << s) {
        let (le_x, _) = x.reverse_bits().overflowing_shr(usize::BITS - (s as u32));

        let f_index = (le_x << 1) + 1;

        let f_index_l = le_x;
        let f_index_r = le_x + (1 << s);

        fs_evals[le_x] = eq[le_x]
            * (f.evaluations[f_index] - f.evaluations[f_index_l] * &f.evaluations[f_index_r]);
    }
    // fs = f(1,x)) - f(x,0)*f(x,1)
    DenseMultilinearExtension::from_evaluations_vec(s, fs_evals)
}

fn compute_mle_eq<E: Pairing>(
    t: &Vec<E::ScalarField>,
    s: usize,
) -> DenseMultilinearExtension<E::ScalarField> {
    let mut eq_evals = vec![E::ScalarField::one(); 1 << s];
    for x in 0usize..(1 << s) {
        let (le_x, _) = x.reverse_bits().overflowing_shr(usize::BITS - (s as u32));
        // turn x into a vector of field elements, where each element is F::zero() or F::one()
        let x_field: Vec<E::ScalarField> = (0..s)
            .rev()
            .map(|i| {
                if (x >> i) & 1 == 1 {
                    E::ScalarField::one()
                } else {
                    E::ScalarField::zero()
                }
            })
            .collect();

        // eq_i = \prod_{i=1}^s (t_i*x_i + (1-t_i)*(1-x_i))
        let eq_i: E::ScalarField = t
            .iter()
            .enumerate()
            .map(|(i, t_i)| {
                *t_i * &x_field[i]
                    + (E::ScalarField::one() - t_i) * (E::ScalarField::one() - x_field[i])
            })
            .product();
        eq_evals[le_x] = eq_i;
    }
    DenseMultilinearExtension::from_evaluations_vec(s, eq_evals)
}

#[cfg(test)]
mod tests {
    use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
    use ark_std::test_rng;
    use ark_test_curves::bls12_381::{Bls12_381, Fr};

    use crate::ml_prodcheck::{compute_f, compute_mle_eq};
    use ark_std::{One, UniformRand, Zero};

    use super::compute_G;

    #[test]
    fn f_computed_correctly() {
        let mut rng = test_rng();
        for s in 0..=10 {
            let v = DenseMultilinearExtension::<Fr>::rand(s, &mut rng);
            let (f, p) = compute_f::<Bls12_381>(&v);

            // assert f(1, 1, ..., 0) = P
            let mut f_point = vec![Fr::one(); s];
            f_point.push(Fr::zero());
            assert_eq!(p, f.evaluate(&f_point).unwrap());

            // assert v(r) = f(0, r)
            for _ in 0..50 {
                let mut f_point = vec![Fr::zero()];
                let v_point: Vec<_> = (0..s).map(|_| Fr::rand(&mut rng)).collect();
                f_point.extend(&v_point);
                assert_eq!(v.evaluate(&v_point).unwrap(), f.evaluate(&f_point).unwrap());
            }

            // assert f(1, x) = f(x, 0) * f(x, 1)
            for _ in 0..50 {
                // let x: Vec<_> = (0..s).map(|_| Fr::rand(&mut rng)).collect();
                let x: Vec<_> = (0..s)
                    .map(|_| {
                        let b = bool::rand(&mut rng);
                        Fr::from(b as u8)
                    })
                    .collect();

                // f(1, x)
                let mut f_point = vec![Fr::one()];
                f_point.extend(&x);

                // f(x, 0)
                let mut f_left = x.clone();
                f_left.push(Fr::zero());
                // f(x, 1)
                let mut f_right = x.clone();
                f_right.push(Fr::one());

                assert_eq!(
                    f.evaluate(&f_point).unwrap(),
                    (f.evaluate(&f_left).unwrap() * f.evaluate(&f_right).unwrap())
                );
            }
        }
    }

    #[test]
    fn mle_eq() {
        let mut rng = test_rng();
        for s in 0..=10 {
            println!("s: {}", s);

            for _ in 0..50 {
                let bool_point: Vec<_> = (0..s)
                    .map(|_| {
                        let b = bool::rand(&mut rng);
                        Fr::from(b as u8)
                    })
                    .collect();
                let other_point: Vec<_> = (0..s)
                    .map(|_| {
                        let b = bool::rand(&mut rng);
                        Fr::from(b as u8)
                    })
                    .collect();
                if bool_point == other_point {
                    println!("ups, points were equal");
                    continue;
                }
                let eq = compute_mle_eq::<Bls12_381>(&bool_point, s);
                assert_eq!(eq.evaluate(&bool_point).unwrap(), Fr::one());
                assert_eq!(eq.evaluate(&other_point).unwrap(), Fr::zero());
            }
        }
    }
    #[test]
    fn g_computed_correctly() {
        let mut rng = test_rng();
        for s in 4..=4 {
            println!("s: {}", s);
            let v = DenseMultilinearExtension::<Fr>::rand(s, &mut rng);
            let (f, _p) = compute_f::<Bls12_381>(&v);

            let bool_point: Vec<_> = (0..s)
                .map(|_| {
                    let b = bool::rand(&mut rng);
                    Fr::from(b as u8)
                })
                .collect();
            println!("bool_point: {:?}", bool_point);
            let g = compute_G::<Bls12_381>(&f, &bool_point);
            assert_eq!(g.evaluate(&bool_point).unwrap(), Fr::zero());

            let sum = (0..(1 << s)).fold(Fr::zero(), |acc, b| {
                let v_point: Vec<Fr> = (0..s)
                    .map(|j| {
                        if (b >> j) & 1 == 1 {
                            Fr::one()
                        } else {
                            Fr::zero()
                        }
                    })
                    .collect();
                println!("v_point: {:?}", v_point);
                acc + g.evaluate(&v_point).unwrap()
            });
            assert_eq!(sum, Fr::zero());
        }
    }
}
