use gura_rs::{errors::InvalidIndentationError};
mod common;

const PARENT_FOLDER: &str = "indentation";

#[test]
/// Tests raising an error when both whitespace and tabs are used at the time for indentation
fn test_wrong_indentation_char() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "different_chars.ura");
    assert!(parsed_data.unwrap_err().downcast_ref::<InvalidIndentationError>().is_some());
}

#[test]
/// Tests raising an error when indentation is not divisible by 4
fn test_indentation_not_divisible_by_4() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "not_divisible_by_4.ura");
    assert!(parsed_data.unwrap_err().downcast_ref::<InvalidIndentationError>().is_some());
}

#[test]
/// Tests raising an error when two levels of an object are not separated by only 4 spaces of difference
fn test_indentation_non_consecutive_blocks() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "more_than_4_difference.ura");
    assert!(parsed_data.unwrap_err().downcast_ref::<InvalidIndentationError>().is_some());
}

#[test]
/// Tests raising an error when tab character is used as indentation
fn test_indentation_with_tabs() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "with_tabs.ura");
    assert!(parsed_data.unwrap_err().downcast_ref::<InvalidIndentationError>().is_some());
}
