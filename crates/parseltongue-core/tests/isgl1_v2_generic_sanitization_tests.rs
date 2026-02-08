use parseltongue_core::isgl1_v2::sanitize_entity_name_for_isgl1;

#[test]
fn test_sanitize_single_generic_type() {
    let input = "List<string>";
    let expected = "List__lt__string__gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_multiple_generic_params_with_space() {
    let input = "Dictionary<string, object>";
    let expected = "Dictionary__lt__string__c__object__gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_nested_generics() {
    let input = "List<List<Integer>>";
    let expected = "List__lt__List__lt__Integer__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_array_notation() {
    let input = "int[]";
    let expected = "int__lb____rb__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_multidimensional_array() {
    let input = "string[][]";
    let expected = "string__lb____rb____lb____rb__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_all_special_chars() {
    let input = "Func<int[], Map<K, V>>";
    let expected = "Func__lt__int__lb____rb____c__Map__lt__K__c__V__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_brace_notation() {
    let input = "Set{T}";
    let expected = "Set__lc__T__rc__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_no_special_chars() {
    let input = "SimpleClass";
    let expected = "SimpleClass";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_only_spaces() {
    let input = "My Class Name";
    let expected = "My_Class_Name";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_mixed_spaces_and_generics() {
    let input = "Dictionary<string, List<int>>";
    let expected = "Dictionary__lt__string__c__List__lt__int__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_complex_cpp_template() {
    let input = "std::vector<std::pair<int, string>>";
    let expected = "std::vector__lt__std::pair__lt__int__c__string__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_java_generic_wildcard() {
    let input = "List<? extends Number>";
    let expected = "List__lt__?_extends_Number__gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_empty_string() {
    let input = "";
    let expected = "";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}
