use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::union::{union, UnionFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::lazy_fst::compare_fst_static_lazy;
use crate::tests_openfst::macros::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnionOperationResult {
    fst_2_path: String,
    result_static_path: String,
    result_lazy_path: String,
}

pub struct UnionTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_lazy: F,
    w: PhantomData<W>,
}

impl UnionOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> UnionTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        UnionTestData {
            fst_2: F::read(dir_path.as_ref().join(&self.fst_2_path)).unwrap(),
            result_static: F::read(dir_path.as_ref().join(&self.result_static_path)).unwrap(),
            result_lazy: F::read(dir_path.as_ref().join(&self.result_lazy_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_union<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for union_test_data in &test_data.union {
        let mut fst_res_static = test_data.raw.clone();
        union(&mut fst_res_static, &union_test_data.fst_2)?;

        test_eq_fst(
            &union_test_data.result_static,
            &fst_res_static,
            "Union failed",
        );
    }
    Ok(())
}

pub fn test_union_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for union_test_data in &test_data.union {
        let union_lazy_fst_openfst = &union_test_data.result_lazy;
        let union_lazy_fst = UnionFst::new(test_data.raw.clone(), union_test_data.fst_2.clone())?;

        compare_fst_static_lazy(union_lazy_fst_openfst, &union_lazy_fst)?;
    }
    Ok(())
}
