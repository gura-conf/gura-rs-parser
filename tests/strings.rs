use std::env;

use gura::{
    errors::VariableNotDefinedError,
    object,
    parser::{parse, GuraType},
};
mod common;

const ESCAPED_VALUE: &str = "$name is cool";

fn get_expected_basic() -> GuraType {
    object! {
        str: "I'm a string. \"You can quote me\". Na\x08me\tJosé\nLocation\tSF.",
        str_2: "I'm a string. \"You can quote me\". Na\x08me\tJosé\nLocation\tSF.",
        with_var: "Gura is cool",
        escaped_var: ESCAPED_VALUE,
        with_env_var: "Gura is very cool"
    }
}

const MULTILINE_VALUE: &str = "Roses are red\nViolets are blue";
const MULTILINE_VALUE_WITHOUT_NEWLINE: &str = "The quick brown fox jumps over the lazy dog.";
fn get_expected_multiline_basic() -> GuraType {
    object! {
        str: MULTILINE_VALUE,
        str_2: MULTILINE_VALUE,
        str_3: MULTILINE_VALUE,
        with_var: MULTILINE_VALUE,
        with_env_var: MULTILINE_VALUE,
        str_with_backslash: MULTILINE_VALUE_WITHOUT_NEWLINE,
        str_with_backslash_2: MULTILINE_VALUE_WITHOUT_NEWLINE,
        str_4: "Here are two quotation marks: \"\". Simple enough.",
        str_5: "Here are three quotation marks: \"\"\".",
        str_6: "Here are fifteen quotation marks: \"\"\"\"\"\"\"\"\"\"\"\"\"\"\".",
        escaped_var: ESCAPED_VALUE,
    }
}

fn get_expected_literal() -> GuraType {
    object! {
        quoted: "John \"Dog lover\" Wick",
        regex: "<\\i\\c*\\s*>",
        winpath: "C:\\Users\\nodejs\\templates",
        winpath2: "\\\\ServerX\\admin$\\system32\\",
        with_var: "$no_parsed variable!",
        escaped_var: ESCAPED_VALUE
    }
}

fn get_expected_multiline_literal() -> GuraType {
    object! {
        lines: "The first newline is\ntrimmed in raw strings.\n   All other whitespace\n   is preserved.\n",
        regex2: "I [dw]on't need \\d{2} apples",
        with_var: "$no_parsed variable!",
        escaped_var: ESCAPED_VALUE
    }
}

const PARENT_FOLDER: &str = "strings";

#[test]
/// Tests basic strings
fn test_basic_strings() {
    let env_var_name = "env_var_value";
    env::set_var(env_var_name, "very");
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "basic.ura").unwrap();
    env::remove_var(env_var_name);
    assert_eq!(parsed_data, get_expected_basic());
}

#[test]
/// Tests multiline basic strings
fn test_multiline_basic_strings() {
    let env_var_name = "env_var_value_multiline";
    env::set_var(env_var_name, "Roses");
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "multiline_basic.ura").unwrap();
    env::remove_var(env_var_name);
    assert_eq!(parsed_data, get_expected_multiline_basic());
}

#[test]
/// Tests errors in basic strings
fn test_basic_strings_errors() {
    let parsed_data = parse(&"test: \"$false_var\"");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<VariableNotDefinedError>()
        .is_some());
}

#[test]
/// Tests literal strings
fn test_literal_strings() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "literal.ura").unwrap();
    assert_eq!(parsed_data, get_expected_literal());
}

#[test]
/// Tests multiline literal strings
fn test_multiline_literal_strings() {
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "multiline_literal.ura").unwrap();
    assert_eq!(parsed_data, get_expected_multiline_literal());
}

#[test]
/// Tests invalid escape sentences interpreted as literals
fn test_invalid_escape_sentence() {
    let parsed_data = parse(r##"foo: "\t\h\i\\i""##).unwrap();
    assert_eq!(
        parsed_data,
        object! {
            foo: "\t\\h\\i\\i"
        }
    );
}
