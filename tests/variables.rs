use gura::{
    errors::{DuplicatedVariableError, ParseError, VariableNotDefinedError},
    object,
    parser::{parse, GuraType},
};
use std::env;
mod common;

fn get_expected() -> GuraType {
    object! {
        plain: 5,
        in_array_middle: [1, 5, 3],
        in_array_last: [1, 2, 5],
        in_object: {
            name: "Aníbal",
            surname: "Troilo",
            year_of_birth: 1914
        }
    }
}

const PARENT_FOLDER: &str = "variables";

#[test]
/// Tests variables definition
fn test_normal() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "normal.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests errors in variables definition
fn test_with_error() {
    let parsed_data = parse(&"test: $false_var".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<VariableNotDefinedError>()
        .is_some());
}

#[test]
/// Tests errors in variables definition
fn test_with_duplicated() {
    let parsed_data = parse(&"$a_var: 14\n$a_var: 15".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<DuplicatedVariableError>()
        .is_some());
}

#[test]
/// Tests using environment variables
fn test_env_var() {
    // Sets a new environment variable to check the correct value retrieval from Gura
    let env_var_name = "env_var_value";
    let env_value = "using_env_var";
    env::set_var(env_var_name, env_value);
    let parsed_data = parse(&format!("test: ${}", env_var_name)).unwrap();
    assert_eq!(parsed_data, object! {test: env_value});
    env::remove_var(env_var_name);
}

#[test]
/// Tests invalid variable value type
fn test_invalid_variable() {
    let parsed_data = parse(&"$invalid: true".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid variable value type
fn test_invalid_variable_2() {
    let parsed_data = parse(&"$invalid: false".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid variable value type
fn test_invalid_variable_3() {
    let parsed_data = parse(&"$invalid: null".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid variable value type
fn test_invalid_variable_4() {
    let parsed_data = parse(&"$invalid: [ 1, 2, 3]".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid variable value type
fn test_invalid_variable_5() {
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "invalid_variable_with_object.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}
