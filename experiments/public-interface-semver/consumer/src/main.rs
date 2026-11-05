use semver::{Comparator, Error, Op, Version, VersionReq};
use std::str::FromStr;

fn main() {
    let requirement: VersionReq = match VersionReq::parse(">=1.2.3, <2.0.0") {
        Ok(requirement) => requirement,
        Err(error) => panic!("valid version requirement failed to parse: {error}"),
    };
    let comparators: &Vec<Comparator> = &requirement.comparators;
    assert_eq!(comparators.len(), 2);
    assert_eq!(comparators[0].op, Op::GreaterEq);
    assert_eq!(comparators[1].op, Op::Less);

    let version: Version = match Version::parse("1.5.0") {
        Ok(version) => version,
        Err(error) => panic!("valid version failed to parse: {error}"),
    };
    let matches_requirement: bool = requirement.matches(&version);
    assert!(matches_requirement);

    let parse_error: Error = match Version::parse("1.not-a-number.0") {
        Ok(_) => panic!("invalid version unexpectedly parsed"),
        Err(error) => error,
    };
    let from_str_error: <Version as FromStr>::Err = match Version::from_str("1.not-a-number.0") {
        Ok(_) => panic!("invalid version unexpectedly parsed through FromStr"),
        Err(error) => error,
    };
    let same_public_error_type: Error = from_str_error;
    assert_eq!(parse_error.to_string(), same_public_error_type.to_string());
}
