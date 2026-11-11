use std::collections::BTreeSet;
use crate::StoreBackendFailureError;

pub fn evaluate_cfg_expression_truth(source: &str, values: &BTreeSet<String>) -> Result<bool, StoreBackendFailureError> {
    struct CfgExpressionParseState<'a> { source: &'a str, offset: usize, values: &'a BTreeSet<String> }
    impl CfgExpressionParseState<'_> {
        fn skip_cfg_source_whitespace(&mut self) {
            while self.source.as_bytes().get(self.offset).is_some_and(u8::is_ascii_whitespace) { self.offset += 1; }
        }
        fn read_cfg_boolean_expression(&mut self) -> Result<bool, ()> {
            self.skip_cfg_source_whitespace();
            let start = self.offset;
            while self.source.as_bytes().get(self.offset).is_some_and(|b| !b.is_ascii_whitespace() && !b"(),=".contains(b)) { self.offset += 1; }
            if start == self.offset { return Err(()); }
            let name = &self.source[start..self.offset];
            self.skip_cfg_source_whitespace();
            match self.source.as_bytes().get(self.offset) {
                Some(b'=') => {
                    self.offset += 1; self.skip_cfg_source_whitespace();
                    let start = self.offset;
                    if self.source.as_bytes().get(self.offset) != Some(&b'"') { return Err(()); }
                    self.offset += 1;
                    while let Some(byte) = self.source.as_bytes().get(self.offset) {
                        self.offset += 1;
                        if *byte == b'\\' { self.offset += 1; }
                        else if *byte == b'"' { return Ok(self.values.contains(&format!("{name}={}", &self.source[start..self.offset]))); }
                    }
                    Err(())
                }
                Some(b'(') => {
                    self.offset += 1;
                    let mut operands = Vec::new();
                    loop {
                        self.skip_cfg_source_whitespace();
                        if self.source.as_bytes().get(self.offset) == Some(&b')') { self.offset += 1; break; }
                        operands.push(self.read_cfg_boolean_expression()?);
                        self.skip_cfg_source_whitespace();
                        match self.source.as_bytes().get(self.offset) { Some(b',') => self.offset += 1, Some(b')') => {}, _ => return Err(()) }
                    }
                    match name { "all" => Ok(operands.iter().all(|b| *b)), "any" => Ok(operands.iter().any(|b| *b)), "not" if operands.len() == 1 => Ok(!operands[0]), _ => Err(()) }
                }
                _ => Ok(self.values.contains(name)),
            }
        }
    }
    let mut parser = CfgExpressionParseState {source,offset:0,values};
    let result = parser.read_cfg_boolean_expression();
    parser.skip_cfg_source_whitespace();
    if parser.offset != source.len() || result.is_err() {
        Err(StoreBackendFailureError{error_context_detail_text:format!("invalid cfg expression: {source}")})
    } else { result.map_err(|_| StoreBackendFailureError{error_context_detail_text:format!("invalid cfg expression: {source}")}) }
}
