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

/// The receiver's place steps as short text: `kind`, or `kind:detail`.
fn read_receiver_place_text(source: &str, call: &str) -> Option<Vec<String>> {
    let evidence = read_method_receiver_evidence(source, call);
    let steps = evidence["receiver_place_step_rows"].as_array()?;
    Some(
        steps
            .iter()
            .map(|step| {
                let kind = step["place_step_kind"].as_str().unwrap();
                match kind {
                    "parameter_identifier" => format!(
                        "{kind}:{}:{}",
                        step["receiver_declared_name_value"].as_str().unwrap(),
                        step["receiver_written_type_evidence"]["type_source_text"]
                            .as_str()
                            .unwrap()
                    ),
                    "value_path" => {
                        let start = step["span_byte_start_offset"].as_u64().unwrap() as usize;
                        let end = step["span_byte_end_offset"].as_u64().unwrap() as usize;
                        assert_eq!(Some(&source[start..end]), step["path_source_text"].as_str());
                        format!("{kind}:{}", &source[start..end])
                    }
                    "field_access" => format!(
                        "{kind}:{}",
                        step["field_declared_name_value"].as_str().unwrap()
                    ),
                    "index_access" => format!(
                        "{kind}:{}",
                        step["index_range_expression_flag"].as_bool().unwrap()
                    ),
                    _ => kind.to_owned(),
                }
            })
            .collect(),
    )
}

/// REQ-RUST-055.0 R2, R3, R4
#[test]
fn test_field_and_static_receivers() {
    let source = "struct Pairs([u8; 200]);\nstatic PAIRS: Pairs = Pairs([0; 200]);\n\
        struct Buffer { bytes: [u8; 4] }\n\
        impl Buffer { fn run(&mut self, buf: &mut [u8], n: usize) {\n\
        self.bytes.as_mut_ptr(); PAIRS.0.get_unchecked(n); buf[n + 1].write(1);\n\
        (buf[n..]).len(); buf[(1..2)].first(); m::PAIRS.0.len(); self.go();\n\
        make().f.len(); (*buf).len(); n.pow(2); } }\n\
        fn rebound(buf: &mut [u8], n: usize) { let buf = 1; buf[n].write(1); }\n\
        fn local(n: usize) { let x = [0u8; 4]; x[n].count_ones(); let PAIRS = 1; PAIRS.0.len(); }\n\
        fn closure() { let f = |b: &[u8]| b[0].count_ones(); }\n";
    let read = |call: &str| read_receiver_place_text(source, call);
    let steps = |texts: &[&str]| {
        Some(
            texts
                .iter()
                .map(|text| (*text).to_owned())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(
        steps(&["self_value", "field_access:bytes"]),
        read("self.bytes.as_mut_ptr()")
    );
    assert_eq!(
        steps(&["value_path:PAIRS", "field_access:0"]),
        read("PAIRS.0.get_unchecked(n)")
    );
    assert_eq!(
        steps(&["parameter_identifier:buf:&mut [u8]", "index_access:false"]),
        read("buf[n + 1].write(1)")
    );
    assert_eq!(
        steps(&["parameter_identifier:buf:&mut [u8]", "index_access:true"]),
        read("(buf[n..]).len()")
    );
    assert_eq!(
        steps(&["parameter_identifier:buf:&mut [u8]", "index_access:true"]),
        read("buf[(1..2)].first()")
    );
    assert_eq!(
        steps(&["value_path:m::PAIRS", "field_access:0"]),
        read("m::PAIRS.0.len()")
    );
    assert_eq!(steps(&["self_value"]), read("self.go()"));
    assert_eq!(steps(&["parameter_identifier:n:usize"]), read("n.pow(2)"));
    // R3: a call's result and a dereference are no place this slice reads.
    // R4: a base name the body binds again, a local, a closure parameter.
    for call in [
        "make().f.len()",
        "(*buf).len()",
        "buf[n].write(1)",
        "x[n].count_ones()",
        "PAIRS.0.len()",
        "b[0].count_ones()",
    ] {
        assert_eq!(None, read(call), "{call}");
    }
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
