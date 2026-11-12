use crate::{
    SyntaxFileInputData, build_syntax_stage_failure, collect_effective_node_attributes,
    parse_compatible_rust_syntax,
};
use parseltongue_core_row_types::{StoreBackendFailureError, UnitPreludeConfigurationInput};
use ra_ap_syntax::Edition;

pub fn read_unit_prelude_configuration(
    input: &SyntaxFileInputData,
) -> Result<UnitPreludeConfigurationInput, StoreBackendFailureError> {
    let source = std::str::from_utf8(&input.file_source_content_bytes)
        .map_err(build_syntax_stage_failure)?;
    let edition = match input.unit_rust_edition_value.as_str() {
        "2015" => Edition::Edition2015,
        "2018" => Edition::Edition2018,
        "2021" => Edition::Edition2021,
        "2024" => Edition::Edition2024,
        _ => return Err(build_syntax_stage_failure("unsupported edition")),
    };
    let syntax = parse_compatible_rust_syntax(source, "items", edition)?.syntax_node();
    let attributes = collect_effective_node_attributes(&syntax, &input.unit_target_cfg_values)?;
    let has = |name: &str| attributes.iter().any(|value| value.trim() == name);
    Ok(UnitPreludeConfigurationInput {
        unit_no_std_flag: has("no_std") || has("no_core"),
        unit_no_core_flag: has("no_core"),
        unit_implicit_prelude_flag: !has("no_implicit_prelude"),
    })
}

pub fn check_sysroot_context_needed(source: &str) -> bool {
    let tokens = ra_ap_parser::LexedStr::new(Edition::Edition2024, source);
    (0..tokens.len()).any(|index| {
        tokens.kind(index) == ra_ap_syntax::SyntaxKind::IDENT
            && matches!(
                tokens.text(index).trim_start_matches("r#"),
                "std"
                    | "core"
                    | "alloc"
                    | "Clone"
                    | "Copy"
                    | "Send"
                    | "Sync"
                    | "Sized"
                    | "Unpin"
                    | "Drop"
                    | "Fn"
                    | "FnMut"
                    | "FnOnce"
                    | "Box"
                    | "String"
                    | "Vec"
                    | "Option"
                    | "Some"
                    | "None"
                    | "Result"
                    | "Ok"
                    | "Err"
                    | "Default"
                    | "Eq"
                    | "PartialEq"
                    | "Ord"
                    | "PartialOrd"
                    | "From"
                    | "AsRef"
                    | "AsMut"
                    | "drop"
                    | "size_of"
                    | "size_of_val"
                    | "align_of"
                    | "align_of_val"
                    | "Into"
                    | "TryFrom"
                    | "TryInto"
                    | "Iterator"
                    | "IntoIterator"
                    | "DoubleEndedIterator"
                    | "ExactSizeIterator"
                    | "Extend"
                    | "FromIterator"
                    | "ToOwned"
                    | "ToString"
                    | "AsyncFn"
                    | "AsyncFnMut"
                    | "AsyncFnOnce"
                    | "Future"
                    | "IntoFuture"
            )
    })
}

pub fn check_recognized_lang_item(name: &str) -> bool {
    const NAMES: &str = "sized meta_sized pointee_sized unsize structural_peq structural_teq copy use_cloned clone trivial_clone sync discriminant_kind discriminant_type pointee_trait metadata_type dyn_metadata freeze non_null fn_ptr_trait fn_ptr_as_ptr fn_ptr_from_ptr drop destruct bikeshed_guaranteed_no_drop coerce_unsized dispatch_from_dyn reborrow coerce_shared transmute_opts transmute_trait add sub mul div rem neg not bitxor bitand bitor shl shr add_assign sub_assign mul_assign div_assign rem_assign bitxor_assign bitand_assign bitor_assign shl_assign shr_assign index index_mut unsafe_cell unsafe_pinned va_list deref deref_mut deref_pure deref_target receiver receiver_target fn fn_mut fn_once async_fn async_fn_mut async_fn_once async_fn_kind_helper async_fn_kind_upvars call_ref_future call_once_future async_fn_once_output fn_once_output future_trait async_iterator coroutine_state coroutine coroutine_return coroutine_yield unpin pin AsyncGenReady eq partial_ord c_void panic panic_nounwind panic_fmt panic_display const_panic_fmt panic_bounds_check panic_misaligned_pointer_dereference panic_info panic_location panic_impl panic_cannot_unwind panic_null_pointer_dereference begin_panic format_alignment format_argument format_arguments format_count format_placeholder format_unsafe_arg exchange_malloc box_free drop_in_place alloc_layout start eh_personality eh_catch_typeinfo owned_box phantom_data manually_drop maybe_uninit align_offset termination Try tuple_trait slice_len_fn from_residual from_output branch from_yeet into_try_type pointer_like const_param_ty Poll Ready Pending ResumeTy get_context Context poll future_output Option Some None Ok Err Continue Break into_future into_iter next iterator fused_iterator new_unchecked RangeFrom RangeFull RangeInclusive range_inclusive_new Range RangeToInclusive RangeTo RangeFromCopy RangeInclusiveCopy RangeCopy RangeToInclusiveCopy String CStr Ordering field field_base field_type RangeMin RangeMax RangeSub";
    NAMES.split_whitespace().any(|candidate| candidate == name)
}
