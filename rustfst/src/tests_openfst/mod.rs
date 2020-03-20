#![cfg(test)]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;

use failure::{bail, Fail, Fallible};
use path_abs::PathAbs;
use path_abs::PathInfo;
use path_abs::PathMut;
use serde_derive::{Deserialize, Serialize};

use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::semirings::{LogWeight, ProductWeight, SerializableSemiring, TropicalWeight};
use crate::tests_openfst::algorithms::factor_weight_gallic::test_factor_weight_gallic;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicOperationResult;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityOperationResult;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::{
    test_factor_weight_identity, test_factor_weight_identity_dynamic,
};
use crate::tests_openfst::algorithms::fst_convert::test_fst_convert;
use crate::tests_openfst::algorithms::gallic_encode_decode::test_gallic_encode_decode;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicOperationResult;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicTestData;
use crate::tests_openfst::algorithms::matcher::test_sorted_matcher;
use crate::tests_openfst::io::const_fst_bin_deserializer::{
    test_const_fst_aligned_bin_deserializer, test_const_fst_bin_deserializer,
};
use crate::tests_openfst::io::const_fst_bin_serializer::{
    test_const_fst_bin_serializer, test_const_fst_bin_serializer_with_symt,
};
use crate::tests_openfst::io::const_fst_text_serialization::{
    test_const_fst_text_serialization, test_const_fst_text_serialization_with_symt,
};

use self::algorithms::{
    arc_map::{
        test_arc_map_identity, test_arc_map_input_epsilon, test_arc_map_invert,
        test_arc_map_output_epsilon, test_arc_map_plus, test_arc_map_quantize,
        test_arc_map_rmweight, test_arc_map_times, ArcMapWithWeightOperationResult,
        ArcMapWithWeightTestData,
    },
    arcsort::{test_arcsort_ilabel, test_arcsort_olabel},
    compose::{test_compose, test_compose_dynamic},
    connect::test_connect,
    determinize::{test_determinize, DeterminizeOperationResult, DeterminizeTestData},
    encode::{test_encode, test_encode_decode, EncodeOperationResult, EncodeTestData},
    inverse::test_invert,
    minimize::{test_minimize, MinimizeOperationResult, MinimizeTestData},
    project::{test_project_input, test_project_output},
    properties::{parse_fst_properties, test_fst_properties},
    push::{test_push, PushOperationResult, PushTestData},
    replace::{test_replace, test_replace_dynamic, ReplaceOperationResult, ReplaceTestData},
    reverse::test_reverse,
    rm_epsilon::{test_rmepsilon, test_rmepsilon_dynamic},
    shortest_distance::{
        test_shortest_distance, ShorestDistanceOperationResult, ShortestDistanceTestData,
    },
    shortest_path::{test_shortest_path, ShorestPathOperationResult, ShortestPathTestData},
    state_map::{test_state_map_arc_sum, test_state_map_arc_unique},
    topsort::test_topsort,
    union::{UnionOperationResult, UnionTestData},
    weight_pushing::{test_weight_pushing_final, test_weight_pushing_initial},
};
use self::fst_impls::const_fst::test_const_fst_convert_convert;
use self::fst_impls::test_fst_into_iterator::{
    test_fst_into_iterator_const, test_fst_into_iterator_vector,
};
use self::misc::test_del_all_states;
use crate::fst_traits::SerializableFst;
use crate::tests_openfst::algorithms::closure::{
    test_closure_plus, test_closure_plus_dynamic, test_closure_star, test_closure_star_dynamic,
    SimpleStaticDynamicOperationResult, SimpleStaticDynamicTestData,
};
use crate::tests_openfst::algorithms::compose::{ComposeOperationResult, ComposeTestData};
use crate::tests_openfst::algorithms::concat::{
    test_concat, test_concat_dynamic, ConcatOperationResult, ConcatTestData,
};
use crate::tests_openfst::algorithms::matcher::{MatcherOperationResult, MatcherTestData};
use crate::tests_openfst::algorithms::union::{test_union, test_union_dynamic};
use crate::tests_openfst::io::vector_fst_bin_deserializer::test_vector_fst_bin_deserializer;
use crate::tests_openfst::io::vector_fst_bin_deserializer::test_vector_fst_bin_with_symt_deserializer;
use crate::tests_openfst::io::vector_fst_bin_serializer::{
    test_vector_fst_bin_serializer, test_vector_fst_bin_serializer_with_symt,
};
use crate::tests_openfst::io::vector_fst_text_serialization::{
    test_vector_fst_text_serialization, test_vector_fst_text_serialization_with_symt,
};
use crate::tests_openfst::algorithms::compose::{ComposeOperationResult, ComposeTestData};

#[macro_use]
mod macros;

mod algorithms;
mod fst_impls;
mod io;
mod misc;
mod test_symt;
mod test_weights;

#[derive(Serialize, Deserialize, Debug)]
struct FstOperationResult {
    result: String,
}

impl FstOperationResult {
    fn parse<F: SerializableFst>(&self) -> F
    where
        F::W: SerializableSemiring,
    {
        F::from_text_string(self.result.as_str()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedFstTestData {
    rmepsilon: SimpleStaticDynamicOperationResult,
    name: String,
    invert: FstOperationResult,
    weight_type: String,
    raw: FstOperationResult,
    project_output: FstOperationResult,
    connect: FstOperationResult,
    weight_pushing_initial: FstOperationResult,
    weight_pushing_final: FstOperationResult,
    project_input: FstOperationResult,
    reverse: FstOperationResult,
    arc_map_identity: FstOperationResult,
    arc_map_rmweight: FstOperationResult,
    arc_map_invert: FstOperationResult,
    arc_map_input_epsilon: FstOperationResult,
    arc_map_output_epsilon: FstOperationResult,
    arc_map_plus: ArcMapWithWeightOperationResult,
    arc_map_times: ArcMapWithWeightOperationResult,
    arc_map_quantize: FstOperationResult,
    encode: Vec<EncodeOperationResult>,
    encode_decode: Vec<EncodeOperationResult>,
    state_map_arc_sum: FstOperationResult,
    state_map_arc_unique: FstOperationResult,
    determinize: Vec<DeterminizeOperationResult>,
    minimize: Vec<MinimizeOperationResult>,
    arcsort_ilabel: FstOperationResult,
    arcsort_olabel: FstOperationResult,
    topsort: FstOperationResult,
    fst_properties: HashMap<String, bool>,
    raw_vector_bin_path: String,
    raw_const_bin_path: String,
    raw_const_aligned_bin_path: String,
    shortest_distance: Vec<ShorestDistanceOperationResult>,
    shortest_path: Vec<ShorestPathOperationResult>,
    gallic_encode_decode: Vec<GallicOperationResult>,
    factor_weight_identity: Vec<FwIdentityOperationResult>,
    factor_weight_gallic: Vec<FwGallicOperationResult>,
    push: Vec<PushOperationResult>,
    replace: Vec<ReplaceOperationResult>,
    union: Vec<UnionOperationResult>,
    concat: Vec<ConcatOperationResult>,
    closure_plus: SimpleStaticDynamicOperationResult,
    closure_star: SimpleStaticDynamicOperationResult,
    raw_vector_with_symt_bin_path: String,
    matcher: Vec<MatcherOperationResult>,
    compose: Vec<ComposeOperationResult>,
}

pub struct FstTestData<F: SerializableFst>
where
    F::W: SerializableSemiring,
{
    pub rmepsilon: SimpleStaticDynamicTestData<F>,
    #[allow(unused)]
    pub name: String,
    pub invert: F,
    pub raw: F,
    pub project_output: F,
    pub connect: F,
    pub weight_pushing_initial: F,
    pub weight_pushing_final: F,
    pub project_input: F,
    pub reverse: F,
    pub arc_map_identity: F,
    pub arc_map_rmweight: F,
    pub arc_map_invert: F,
    pub arc_map_input_epsilon: F,
    pub arc_map_output_epsilon: F,
    pub arc_map_plus: ArcMapWithWeightTestData<F>,
    pub arc_map_times: ArcMapWithWeightTestData<F>,
    pub arc_map_quantize: F,
    pub encode: Vec<EncodeTestData<F>>,
    pub encode_decode: Vec<EncodeTestData<F>>,
    pub state_map_arc_sum: F,
    pub state_map_arc_unique: F,
    pub determinize: Vec<DeterminizeTestData<F>>,
    pub minimize: Vec<MinimizeTestData<F>>,
    pub arcsort_ilabel: F,
    pub arcsort_olabel: F,
    pub topsort: F,
    pub fst_properties: FstProperties,
    pub raw_vector_bin_path: PathBuf,
    pub raw_const_bin_path: PathBuf,
    pub raw_const_aligned_bin_path: PathBuf,
    pub shortest_distance: Vec<ShortestDistanceTestData<F::W>>,
    pub shortest_path: Vec<ShortestPathTestData<F>>,
    pub gallic_encode_decode: Vec<GallicTestData<F>>,
    pub factor_weight_identity: Vec<FwIdentityTestData<F>>,
    pub factor_weight_gallic: Vec<FwGallicTestData<F>>,
    pub push: Vec<PushTestData<F>>,
    pub replace: Vec<ReplaceTestData<F>>,
    pub union: Vec<UnionTestData<F>>,
    pub concat: Vec<ConcatTestData<F>>,
    pub closure_plus: SimpleStaticDynamicTestData<F>,
    pub closure_star: SimpleStaticDynamicTestData<F>,
    pub raw_vector_with_symt_bin_path: PathBuf,
    pub matcher: Vec<MatcherTestData<F>>,
    pub compose: Vec<ComposeTestData<F>>,
}

impl<F: SerializableFst> FstTestData<F>
where
    F::W: SerializableSemiring,
{
    pub fn new(data: &ParsedFstTestData, absolute_path_folder: &Path) -> Self {
        Self {
            rmepsilon: data.rmepsilon.parse(),
            name: data.name.clone(),
            invert: data.invert.parse(),
            raw: data.raw.parse(),
            project_output: data.project_output.parse(),
            connect: data.connect.parse(),
            weight_pushing_initial: data.weight_pushing_initial.parse(),
            weight_pushing_final: data.weight_pushing_final.parse(),
            project_input: data.project_input.parse(),
            reverse: data.reverse.parse(),
            arc_map_identity: data.arc_map_identity.parse(),
            arc_map_rmweight: data.arc_map_rmweight.parse(),
            arc_map_invert: data.arc_map_invert.parse(),
            arc_map_input_epsilon: data.arc_map_input_epsilon.parse(),
            arc_map_output_epsilon: data.arc_map_output_epsilon.parse(),
            arc_map_plus: data.arc_map_plus.parse(),
            arc_map_times: data.arc_map_times.parse(),
            arc_map_quantize: data.arc_map_quantize.parse(),
            encode: data.encode.iter().map(|v| v.parse()).collect(),
            encode_decode: data.encode_decode.iter().map(|v| v.parse()).collect(),
            state_map_arc_sum: data.state_map_arc_sum.parse(),
            state_map_arc_unique: data.state_map_arc_unique.parse(),
            determinize: data.determinize.iter().map(|v| v.parse()).collect(),
            minimize: data.minimize.iter().map(|v| v.parse()).collect(),
            arcsort_ilabel: data.arcsort_ilabel.parse(),
            arcsort_olabel: data.arcsort_olabel.parse(),
            topsort: data.topsort.parse(),
            fst_properties: parse_fst_properties(&data.fst_properties),
            raw_vector_bin_path: absolute_path_folder
                .join(&data.raw_vector_bin_path)
                .to_path_buf(),
            raw_const_bin_path: absolute_path_folder
                .join(&data.raw_const_bin_path)
                .to_path_buf(),
            raw_const_aligned_bin_path: absolute_path_folder
                .join(&data.raw_const_aligned_bin_path)
                .to_path_buf(),
            shortest_distance: data.shortest_distance.iter().map(|v| v.parse()).collect(),
            shortest_path: data.shortest_path.iter().map(|v| v.parse()).collect(),
            gallic_encode_decode: data
                .gallic_encode_decode
                .iter()
                .map(|v| v.parse())
                .collect(),
            factor_weight_identity: data
                .factor_weight_identity
                .iter()
                .map(|v| v.parse())
                .collect(),
            factor_weight_gallic: data
                .factor_weight_gallic
                .iter()
                .map(|v| v.parse())
                .collect(),
            push: data.push.iter().map(|v| v.parse()).collect(),
            replace: data.replace.iter().map(|v| v.parse()).collect(),
            union: data.union.iter().map(|v| v.parse()).collect(),
            concat: data.concat.iter().map(|v| v.parse()).collect(),
            closure_plus: data.closure_plus.parse(),
            closure_star: data.closure_star.parse(),
            raw_vector_with_symt_bin_path: absolute_path_folder
                .join(&data.raw_vector_with_symt_bin_path)
                .to_path_buf(),
            matcher: data.matcher.iter().map(|v| v.parse()).collect(),
            compose: data.compose.iter().map(|v| v.parse()).collect(),
        }
    }
}

pub(crate) fn get_path_folder(test_name: &str) -> Fallible<PathBuf> {
    let mut path_repo = PathAbs::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
    path_repo.append("rustfst-tests-data")?;
    path_repo.append(test_name)?;
    Ok(path_repo.as_path().to_path_buf())
}

pub(crate) struct ExitFailure(failure::Error);

/// Prints a list of causes for this Error, along with any backtrace
/// information collected by the Error (if RUST_BACKTRACE=1).
impl std::fmt::Debug for ExitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fail = self.0.as_fail();

        writeln!(f, "{}", &fail)?;

        let mut x: &dyn Fail = fail;
        while let Some(cause) = x.cause() {
            writeln!(f, " -> caused by: {}", &cause)?;
            x = cause;
        }
        if let Some(backtrace) = x.backtrace() {
            writeln!(f, "{:?}", backtrace)?;
        }

        Ok(())
    }
}

impl<T: Into<failure::Error>> From<T> for ExitFailure {
    fn from(t: T) -> Self {
        ExitFailure(t.into())
    }
}

macro_rules! do_run {
    ($f: ident, $fst_name: expr) => {
        let absolute_path_folder = get_path_folder($fst_name)?;
        let mut path_metadata = absolute_path_folder.clone();
        path_metadata.push("metadata.json");

        let string = read_to_string(&path_metadata)
            .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
        let parsed_test_data: ParsedFstTestData = serde_json::from_str(&string).unwrap();

        match parsed_test_data.weight_type.as_str() {
            "tropical" | "standard" => {
                let test_data: FstTestData<VectorFst<TropicalWeight>> =
                    FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            "log" => {
                let test_data: FstTestData<VectorFst<LogWeight>> =
                    FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            "tropical_X_log" => {
                let test_data: FstTestData<VectorFst<ProductWeight<TropicalWeight, LogWeight>>> =
                    FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            _ => bail!("Weight type unknown : {:?}", parsed_test_data.weight_type),
        };
    };
}

macro_rules! test_fst {
    ($namespace: tt, $fst_name: expr) => {
        mod $namespace {
            use super::*;

            #[test]
            fn test_union_openfst() -> Fallible<()> {
                do_run!(test_union, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_identity_openfst() -> Fallible<()> {
                do_run!(test_arc_map_identity, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_invert_openfst() -> Fallible<()> {
                do_run!(test_arc_map_invert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_input_epsilon_openfst() -> Fallible<()> {
                do_run!(test_arc_map_input_epsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_output_epsilon_openfst() -> Fallible<()> {
                do_run!(test_arc_map_output_epsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_plus_openfst() -> Fallible<()> {
                do_run!(test_arc_map_plus, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_times_openfst() -> Fallible<()> {
                do_run!(test_arc_map_times, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_quantize_openfst() -> Fallible<()> {
                do_run!(test_arc_map_quantize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arc_map_rmweight_openfst() -> Fallible<()> {
                do_run!(test_arc_map_rmweight, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arcsort_ilabel_openfst() -> Fallible<()> {
                do_run!(test_arcsort_ilabel, $fst_name);
                Ok(())
            }

            #[test]
            fn test_arcsort_olabel_openfst() -> Fallible<()> {
                do_run!(test_arcsort_olabel, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_plus_openfst() -> Fallible<()> {
                do_run!(test_closure_plus, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_star_openfst() -> Fallible<()> {
                do_run!(test_closure_star, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_plus_dynamic_openfst() -> Fallible<()> {
                do_run!(test_closure_plus_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_star_dynamic_openfst() -> Fallible<()> {
                do_run!(test_closure_star_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_concat_openfst() -> Fallible<()> {
                do_run!(test_concat, $fst_name);
                Ok(())
            }

            #[test]
            fn test_concat_dynamic_openfst() -> Fallible<()> {
                do_run!(test_concat_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_connect_openfst() -> Fallible<()> {
                do_run!(test_connect, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_identity_openfst() -> Fallible<()> {
                do_run!(test_factor_weight_identity, $fst_name);
                Ok(())
            }

            #[test]
            fn test_determinize_openfst() -> Fallible<()> {
                do_run!(test_determinize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_encode_decode_openfst() -> Fallible<()> {
                do_run!(test_encode_decode, $fst_name);
                Ok(())
            }

            #[test]
            fn test_encode_openfst() -> Fallible<()> {
                do_run!(test_encode, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_gallic_openfst() -> Fallible<()> {
                do_run!(test_factor_weight_gallic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_identity_dynamic_openfst() -> Fallible<()> {
                do_run!(test_factor_weight_identity_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_gallic_encode_decode_openfst() -> Fallible<()> {
                do_run!(test_gallic_encode_decode, $fst_name);
                Ok(())
            }

            #[test]
            fn test_invert_openfst() -> Fallible<()> {
                do_run!(test_invert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_minimize_openfst() -> Fallible<()> {
                do_run!(test_minimize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_project_output_openfst() -> Fallible<()> {
                do_run!(test_project_output, $fst_name);
                Ok(())
            }

            #[test]
            fn test_project_input_openfst() -> Fallible<()> {
                do_run!(test_project_input, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_properties_openfst() -> Fallible<()> {
                do_run!(test_fst_properties, $fst_name);
                Ok(())
            }

            #[test]
            fn test_push_openfst() -> Fallible<()> {
                do_run!(test_push, $fst_name);
                Ok(())
            }

            #[test]
            fn test_replace_openfst() -> Fallible<()> {
                do_run!(test_replace, $fst_name);
                Ok(())
            }

            #[test]
            fn test_replace_dynamic_openfst() -> Fallible<()> {
                do_run!(test_replace_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_reverse_openfst() -> Fallible<()> {
                do_run!(test_reverse, $fst_name);
                Ok(())
            }

            #[test]
            fn test_shortest_distance_openfst() -> Fallible<()> {
                do_run!(test_shortest_distance, $fst_name);
                Ok(())
            }

            #[test]
            fn test_state_map_arc_unique_openfst() -> Fallible<()> {
                do_run!(test_state_map_arc_unique, $fst_name);
                Ok(())
            }

            #[test]
            fn test_state_map_arc_sum_openfst() -> Fallible<()> {
                do_run!(test_state_map_arc_sum, $fst_name);
                Ok(())
            }

            #[test]
            fn test_shortest_path_openfst() -> Fallible<()> {
                do_run!(test_shortest_path, $fst_name);
                Ok(())
            }

            #[test]
            fn test_topsort_openfst() -> Fallible<()> {
                do_run!(test_topsort, $fst_name);
                Ok(())
            }

            #[test]
            fn test_union_dynamic_openfst() -> Fallible<()> {
                do_run!(test_union_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_weight_pushing_initial_openfst() -> Fallible<()> {
                do_run!(test_weight_pushing_initial, $fst_name);
                Ok(())
            }

            #[test]
            fn test_del_all_states_openfst() -> Fallible<()> {
                do_run!(test_del_all_states, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_text_serialization_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_text_serialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_text_serialization_with_symt_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_text_serialization_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_serializer_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_bin_serializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_serializer_with_symt_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_bin_serializer_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_with_symt_deserializer_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_bin_with_symt_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_deserializer_openfst() -> Fallible<()> {
                do_run!(test_vector_fst_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_weight_pushing_final_openfst() -> Fallible<()> {
                do_run!(test_weight_pushing_final, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_convert_convert_openfst() -> Fallible<()> {
                do_run!(test_const_fst_convert_convert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_deserializer_openfst() -> Fallible<()> {
                do_run!(test_const_fst_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_aligned_bin_deserializer_openfst() -> Fallible<()> {
                do_run!(test_const_fst_aligned_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_serializer_openfst() -> Fallible<()> {
                do_run!(test_const_fst_bin_serializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_serializer_with_symt_openfst() -> Fallible<()> {
                do_run!(test_const_fst_bin_serializer_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_text_serialization_openfst() -> Fallible<()> {
                do_run!(test_const_fst_text_serialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_text_serialization_with_symt_openfst() -> Fallible<()> {
                do_run!(test_const_fst_text_serialization_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_rmepsilon_openfst() -> Fallible<()> {
                do_run!(test_rmepsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_rmepsilon_dynamic_openfst() -> Fallible<()> {
                do_run!(test_rmepsilon_dynamic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_into_iterator_const_openfst() -> Fallible<()> {
                do_run!(test_fst_into_iterator_const, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_into_iterator_vector_openfst() -> Fallible<()> {
                do_run!(test_fst_into_iterator_vector, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_convert_openfst() -> Fallible<()> {
                do_run!(test_fst_convert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_sorted_matcher_openfst() -> Fallible<()> {
                do_run!(test_sorted_matcher, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_compose_openfst() -> Fallible<()> {
                do_run!(test_compose, $fst_name);
                Ok(())
            }

            // #[test]
            // fn test_fst_compose_dynamic_openfst() -> Fallible<()> {
            //     do_run!(test_compose_dynamic, $fst_name);
            //     Ok(())
            // }
        }
    };
}

test_fst!(test_openfst_fst_000, "fst_000");
test_fst!(test_openfst_fst_001, "fst_001");
test_fst!(test_openfst_fst_002, "fst_002");
test_fst!(test_openfst_fst_003, "fst_003");
test_fst!(test_openfst_fst_004, "fst_004");
test_fst!(test_openfst_fst_005, "fst_005");
test_fst!(test_openfst_fst_006, "fst_006");
test_fst!(test_openfst_fst_007, "fst_007");
test_fst!(test_openfst_fst_008, "fst_008");
test_fst!(test_openfst_fst_009, "fst_009");
test_fst!(test_openfst_fst_010, "fst_010");
test_fst!(test_openfst_fst_011, "fst_011");
