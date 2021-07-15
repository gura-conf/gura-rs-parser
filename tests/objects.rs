use gura::{
    errors::{InvalidIndentationError, ParseError},
    object,
    parser::GuraType,
};
mod common;

fn get_expected() -> GuraType {
    object! {
        user1: {
            name: "Carlos",
            surname: "Gardel",
            testing_nested: {
                nested_1: 1,
                nested_2: 2
            },
            year_of_birth: 1890
        },
        user2: {
            name: "Aníbal",
            surname: "Troilo",
            year_of_birth: 1914
        }
    }
}

const PARENT_FOLDER: &str = "objects";

#[test]
/// Tests all kind of objects
fn test_normal() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "normal.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests all kind of objects with comments between elements
fn test_with_comments() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "with_comments.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests parsing error in invalid objects
fn test_invalid() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "invalid.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests parsing error in invalid objects
fn test_invalid_2() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "invalid_2.ura");
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<InvalidIndentationError>()
        .is_some());
}
