use gura::{object, parser::GuraType};
mod common;

fn get_expected() -> GuraType {
    object! {
        colors: ["red", "yellow", "green"],
        integers: [1, 2, 3],
        integers_with_new_line: [1, 2, 3],
        nested_arrays_of_ints: [[1, 2], [3, 4, 5]],
        nested_mixed_array: [[1, 2], ["a", "b", "c"]],
        numbers: [0.1, 0.2, 0.5, 1, 2, 5],
        tango_singers: [{
            user1: {
                name: "Carlos",
                surname: "Gardel",
                testing_nested: {
                    nested_1: 1,
                    nested_2: 2
                },
                year_of_birth: 1890
            }
        }, {
            user2: {
                name: "Aníbal",
                surname: "Troilo",
                year_of_birth: 1914
            }
        }],
        mixed_with_object: [
            1,
            {test: {genaro: "Camele"}},
            2,
            [4, 5, 6],
            3
        ],
        separator: [
            {a: 1, b: 2},
            {a: 1},
            {b: 2}
        ]
    }
}

fn get_expected_inside_object() -> GuraType {
    object! {
        model: {
            columns: [
                ["var1", "str"],
                ["var2", "str"]
            ]
        }
    }
}

fn get_expected_trailing_comma() -> GuraType {
    object! {
        foo: [{
            bar: {
                baz: [
                    { far: "faz" }
                ]
            }
        }],
        barbaz: "boo"
    }
}

const PARENT_FOLDER: &str = "arrays";

#[test]
/// Tests all kind of arrays
fn test_normal() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "normal.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests a bug that breaks arrays with a mandatory trailing comma. In this case the trailing comma is
/// missing and it should parse correctly
fn bug_trailing_comma() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "bug_trailing_comma.ura").unwrap();
    assert_eq!(parsed_data, get_expected_trailing_comma());
}

/// Tests all kind of arrays with comments between elements
#[test]
fn test_with_comments() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "with_comments.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

/// Tests issue https://github.com/gura-conf/gura/issues/1
#[test]
fn test_array_in_object() {
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "array_in_object.ura").unwrap();
    assert_eq!(parsed_data, get_expected_inside_object());
    let parsed_data =
        common::get_file_content_parsed(PARENT_FOLDER, "array_in_object_trailing_comma.ura")
            .unwrap();
    assert_eq!(parsed_data, get_expected_inside_object());
}
