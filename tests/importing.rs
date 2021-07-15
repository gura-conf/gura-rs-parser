use gura::{
    errors::{
        DuplicatedImportError, DuplicatedKeyError, DuplicatedVariableError, FileNotFoundError,
        ParseError,
    },
    object,
    parser::{parse, GuraType},
};
use tempfile::NamedTempFile;
mod common;
use std::io::Write;

fn get_expected() -> GuraType {
    object! {
        from_file_one: 1,
        from_file_two: {
            name: "Aníbal",
            surname: "Troilo",
            year_of_birth: 1914
        },
        from_original_1: [1, 2, 5],
        from_original_2: false,
        from_file_three: true,
    }
}

const PARENT_FOLDER: &str = "importing";

#[test]
/// Tests importing from several files
fn test_normal() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "normal.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests importing from several files with a variable in import sentences
fn test_with_variables() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "with_variable.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests errors importing a non existing file
fn test_not_found_error() {
    let parsed_data = parse(&"import \"invalid_file.ura\"".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<FileNotFoundError>()
        .is_some());
}

#[test]
/// Tests errors when redefines a key
fn test_duplicated_key_error() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "duplicated_key.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<DuplicatedKeyError>()
        .is_some());
}

#[test]
/// Tests errors when redefines a variable
fn test_duplicated_variable_error() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "duplicated_variable.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<DuplicatedVariableError>()
        .is_some());
}

#[test]
/// Tests errors when imports more than once a file
fn test_duplicated_imports() {
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "duplicated_imports_simple.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<DuplicatedImportError>()
        .is_some());
}

#[test]
/// Tests that absolute paths works as expected
fn test_with_absolute_paths() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "from_temp: true").unwrap();
    let parsed_data = parse(&format!(
        "import \"{}\"\nfrom_original: false",
        temp_file.path().to_str().unwrap()
    ))
    .unwrap();
    assert_eq!(
        parsed_data,
        object! {
            from_temp: true,
            from_original: false
        }
    );
    temp_file.close().unwrap();
}

#[test]
/// Tests errors invalid importing sentence (there are blanks before import)
fn test_parse_error_1() {
    let parsed_data = parse(&"  import \"another_file.ura\"".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests errors invalid importing sentence (there are more than one whitespace between import and file name)
fn test_parse_error_2() {
    let parsed_data = parse(&"import   \"another_file.ura\"".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}
