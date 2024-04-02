use ark_ff::PrimeField;
use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
    ToBitsGadget,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use mpc_algebra::{
    malicious_majority::MpcField, MpcBoolean, MpcEqGadget, MpcFpVar, MpcToBitsGadget,
};

type Fr = ark_bls12_377::Fr;
type MFr = MpcField<Fr>;

pub struct BitDecompositionCircuit<F: PrimeField> {
    pub a: F,
}

impl ConstraintSynthesizer<MFr> for BitDecompositionCircuit<MFr> {
    fn generate_constraints(self, cs: ConstraintSystemRef<MFr>) -> Result<(), SynthesisError> {
        let a_var = MpcFpVar::new_witness(cs.clone(), || Ok(self.a))?;

        let bits = a_var.to_bits_le()?;

        // a_var.is_zero()?.enforce_equal(&MpcBoolean::TRUE)?;

        Ok(())
    }
}

impl ConstraintSynthesizer<Fr> for BitDecompositionCircuit<Fr> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let a_var = FpVar::new_witness(cs.clone(), || Ok(self.a))?;

        // a_var.is_zero()?.enforce_equal(&Boolean::TRUE)?;

        let bits = a_var.to_bits_le()?;

        Ok(())
    }
}
