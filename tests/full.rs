use gura::{
    errors::ParseError,
    object,
    parser::{dump, parse, GuraType},
};
use std::f64::{INFINITY, NAN, NEG_INFINITY};
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
        int7: 5349221,
        hex1: 3735928559,
        hex2: 3735928559,
        hex3: 3735928559,
        oct1: 342391,
        oct2: 493,
        bin1: 214,
        flt1: 1.0,
        flt2: 3.1415,
        flt3: -0.01,
        flt4: 5e+22,
        flt5: 1e06,
        flt6: -2E-2,
        flt7: 6.626e-34,
        flt8: 224617.445991228,
        sf1: INFINITY,
        sf2: INFINITY,
        sf3: NEG_INFINITY,
        null: null,
        bool1: true,
        bool2: false,
        1234: "1234",
        services: {
            nginx: {
                host: "127.0.0.1",
                port: 80
            },
            apache: {
                virtual_host: "10.10.10.4",
                port: 81
            }
        },
        integers: [1, 2, 3],
        colors: ["red", "yellow", "green"],
        nested_arrays_of_ints: [[1, 2], [3, 4, 5]],
        nested_mixed_array: [[1, 2], ["a", "b", "c"]],
        numbers: [0.1, 0.2, 0.5, 1, 2, 5],
        tango_singers: [
            {
                user1: {
                    name: "Carlos",
                    surname: "Gardel",
                    year_of_birth: 1890
                }
            }, {
                user2: {
                    name: "Aníbal",
                    surname: "Troilo",
                    year_of_birth: 1914
                }
            }
        ],
        integers2: [
            1, 2, 3
        ],
        integers3: [
            1,
            2
        ],
        my_server: {
            host: "127.0.0.1",
            port: 8080,
            native_auth: true
        },
        gura_is_cool: "Gura is cool"
    }
}

const PARENT_FOLDER: &str = "full";

#[test]
/// Tests all the common cases except NaNs
fn test_parse() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "full.ura").unwrap();
    assert_eq!(parsed_data, get_expected());
}

#[test]
/// Tests NaNs cases as they are an exceptional case
fn test_loads_nan() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "nan.ura").unwrap();
    for (_, value) in parsed_data.iter().unwrap() {
        assert_eq!(**value, NAN);
    }
}

#[test]
/// Tests dumps method
fn test_dumps() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "full.ura").unwrap();
    let string_data = dump(&parsed_data);
    let new_parsed_data = parse(&string_data).unwrap();
    assert_eq!(new_parsed_data, get_expected());
}

#[test]
/// Tests dumps method with NaNs values
fn test_dumps_nan() {
    let parsed_data = common::get_file_content_parsed(PARENT_FOLDER, "nan.ura").unwrap();
    let string_data_nan = dump(&parsed_data);
    let new_parsed_data = parse(&string_data_nan).unwrap();
    for (_, value) in new_parsed_data.iter().unwrap() {
        assert_eq!(**value, NAN);
    }
}

#[test]
/// Tests empty Gura documents
fn test_empty() {
    let parsed_data = parse(&"".to_string()).unwrap();
    assert_eq!(parsed_data, object! {});
}

#[test]
/// Tests empty Gura documents, even when some data is defined
fn test_empty_2() {
    let parsed_data = parse(&"$unused_var: 5".to_string()).unwrap();
    assert_eq!(parsed_data, object! {});
}

#[test]
/// Tests invalid key
fn test_invalid_key() {
    let parsed_data = parse(&"with.dot: 5".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid key
fn test_invalid_key_2() {
    let parsed_data = parse(&"\"with_quotes\": 5".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}

#[test]
/// Tests invalid key
fn test_invalid_key_3() {
    let parsed_data = parse(&"with-dashes: 5".to_string());
    assert!(parsed_data
        .unwrap_err()
        .downcast_ref::<ParseError>()
        .is_some());
}
