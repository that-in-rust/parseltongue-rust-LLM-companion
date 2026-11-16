#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::original_call_resolution_tests::read_method_receiver_evidence;

fn read_written_receiver_type(source: &str, call: &str) -> Option<String> {
    let evidence = read_method_receiver_evidence(source, call);
    let written = &evidence["receiver_written_type_evidence"];
    if written.is_null() {
        return None;
    }
    let start = written["span_byte_start_offset"].as_u64().unwrap() as usize;
    let end = written["span_byte_end_offset"].as_u64().unwrap() as usize;
    assert_eq!(
        Some(&source[start..end]),
        written["type_source_text"].as_str(),
        "{evidence}"
    );
    Some(source[start..end].to_owned())
}

/// REQ-RUST-055.0 R1, R3, R4
#[test]
fn test_parameter_written_type_rows() {
    let source = "trait Sealed { type Buffer; }\n\
        fn slice(buf: &[MaybeUninit<u8>], offset: usize) { buf.get_unchecked(offset..); }\n\
        fn array(arr: &mut [MaybeUninit<u8>; 4]) { arr.len(); }\n\
        fn bytes(b: [u8; 200], p: *mut u8, i: I) { b.len(); p.cast(); i.go(); }\n\
        trait T: Sealed { fn fmt(self, sb: &mut Self::Buffer) -> usize { sb.len() } }\n\
        fn tuple((a, _): (u8, u8)) { a.len(); }\n\
        fn rebound(rb: &[u8]) { let rb = 3; rb.len(); }\n\
        fn local() { let x = make(); x.len(); }\n";
    let read = |call: &str| read_written_receiver_type(source, call);
    let written = |text: &str| Some(text.to_owned());
    assert_eq!(
        written("&[MaybeUninit<u8>]"),
        read("buf.get_unchecked(offset..)")
    );
    assert_eq!(written("&mut [MaybeUninit<u8>; 4]"), read("arr.len()"));
    assert_eq!(written("[u8; 200]"), read("b.len()"));
    assert_eq!(written("*mut u8"), read("p.cast()"));
    assert_eq!(written("I"), read("i.go()"));
    assert_eq!(written("&mut Self::Buffer"), read("sb.len()"));
    for call in ["a.len()", "rb.len()", "x.len()"] {
        assert_eq!(None, read(call), "{call}");
    }
}
