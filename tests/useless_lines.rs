use gura::{object, parser::GuraType};
mod common;

fn get_expected() -> GuraType {
    object! {
        a_string: "test string",
        int1: 99,
        int2: 42,
        int3: 0,
        int4: -17,
        int5: 1000,
        int6: 5349221,
        int7: 5349221
    }
}

fn get_expected_object() -> GuraType {
    object! {
        testing: {
            test_2: 2,
            test: {
                name: "JWARE",
                surname: "Solutions"
            }
        }
    }
}

fn get_expected_object_complex() -> GuraType {
    object! {
        testing: {
            test: {
                name: "JWARE",
                surname: "Solutions",
                skills: {
                    good_testing: false,
                    good_programming: false,
                    good_english: false
                }
            },
            test_2: 2,
            test_3: {
                key_1: true,
                key_2: false,
                key_3: 55.99
            }
        }
    }
}

const PARENT_FOLDER: &str = "useless_lines";

/// Helper: tests it against the data retrieved from get_expected()
fn check_test_file(file_name: &str) {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, file_name).unwrap();
    assert_eq!(parsed_data, get_expected());
}

/// Helper: tests it against the data retrieved from get_expected_object()
fn check_test_file_object(file_name: &str) {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, file_name).unwrap();
    assert_eq!(parsed_data, get_expected_object());
}

#[test]
/// Tests without comments or blank lines
fn test_without() {
    check_test_file("without.ura");
}

#[test]
/// Tests with comments or blank lines on the top of the file
fn test_on_top() {
    check_test_file("on_top.ura");
}

#[test]
/// Tests with comments or blank lines on the bottom of the file
fn test_on_bottom() {
    check_test_file("on_bottom.ura");
}

#[test]
/// Tests with comments or blank lines on the top and bottom of the file
fn test_on_both() {
    check_test_file("on_both.ura");
}

#[test]
/// Tests with comments or blank lines in the middle of valid sentences
fn test_in_the_middle() {
    check_test_file("in_the_middle.ura");
}

#[test]
/// Tests without comments or blank lines in the middle of valid object
fn test_without_object() {
    check_test_file_object("without_object.ura");
}

#[test]
/// Tests with comments or blank lines in the middle of valid object
fn test_in_the_middle_object() {
    check_test_file_object("without_object.ura");
}

#[test]
/// Tests with comments or blank lines in the middle of valid complex object
fn test_in_the_middle_object_complex() {
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "in_the_middle_object_complex.ura").unwrap();
    assert_eq!(parsed_data, get_expected_object_complex());
}

#[test]
/// Tests issue https://github.com/gura-conf/gura-rs-parser/issues/13
fn test_issue_13() {
    check_test_file("issue_13.ura");
}
