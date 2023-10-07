use gura::errors::Error;
mod common;

const PARENT_FOLDER: &str = "exception_report";

fn test_fail(filename: &str, expected_err_kind: Error, pos: isize, line: usize) {
    let parsed_res = common::get_file_content_parsed(PARENT_FOLDER, filename);
    if let Err(some_err) = parsed_res {
        println!("{}", some_err);
        assert_eq!(some_err.kind, expected_err_kind);
        assert_eq!(some_err.pos, pos);
        assert_eq!(some_err.line, line);
    } else {
        panic!("Expected error!")
    }
}

#[test]
/// Tests error position and line at beginning
fn test_line_and_pos_1() {
    test_fail("parsing_error_1.ura", Error::ParseError, 0, 1);
}

#[test]
/// Tests error position and line at the end of file
fn test_line_and_pos_2() {
    test_fail("parsing_error_2.ura", Error::ParseError, 10, 1);
}

#[test]
/// Tests error position and line in some random line
fn test_line_and_pos_3() {
    test_fail("parsing_error_3.ura", Error::ParseError, 42, 2);
}

#[test]
/// Tests error position and line in some random line
fn test_line_and_pos_4() {
    test_fail("parsing_error_4.ura", Error::ParseError, 45, 6);
}

#[test]
/// Tests error position and line when user uses tabs to indent
fn test_line_and_pos_indentation_1() {
    test_fail(
        "indentation_error_1.ura",
        Error::InvalidIndentationError,
        20,
        3,
    );
}

#[test]
/// Tests error position and line when indentation is not divisible by 4
fn test_line_and_pos_indentation_2() {
    test_fail(
        "indentation_error_2.ura",
        Error::InvalidIndentationError,
        19,
        3,
    );
}

#[test]
/// Tests error position and line when pair indentation is the same as the the parent
fn test_line_and_pos_indentation_3() {
    test_fail(
        "indentation_error_3.ura",
        Error::InvalidIndentationError,
        18,
        3,
    );
}

#[test]
/// Tests error position and line when pair indentation is more than 4 spaces from parent indentation level
fn test_line_and_pos_indentation_4() {
    test_fail(
        "indentation_error_4.ura",
        Error::InvalidIndentationError,
        26,
        3,
    );
}

#[test]
/// Tests error position and line when user defines the same key twice
fn test_duplicated_key_1() {
    test_fail(
        "duplicated_key_error_1.ura",
        Error::DuplicatedKeyError,
        11,
        2,
    );
}

#[test]
/// Tests error position and line when user defines the same key twice but in other line than 0
fn test_duplicated_key_2() {
    test_fail(
        "duplicated_key_error_2.ura",
        Error::DuplicatedKeyError,
        21,
        3,
    );
}

#[test]
/// Tests error position and line when user defines the same key twice inside an object
fn test_duplicated_key_3() {
    test_fail(
        "duplicated_key_error_3.ura",
        Error::DuplicatedKeyError,
        37,
        4,
    );
}

#[test]
/// Tests error position and line when user defines the same variable twice inside an object
fn test_duplicated_variable_1() {
    test_fail(
        "duplicated_variable_error_1.ura",
        Error::DuplicatedVariableError,
        12,
        2,
    );
}

#[test]
/// Tests error position and line when user defines the same variable twice but in other line than 0
fn test_duplicated_variable_2() {
    test_fail(
        "duplicated_variable_error_2.ura",
        Error::DuplicatedVariableError,
        25,
        3,
    );
}

#[test]
/// Tests error position and line when user defines the same variable twice but in other line than 0
fn test_duplicated_variable_3() {
    test_fail(
        "duplicated_variable_error_3.ura",
        Error::DuplicatedVariableError,
        37,
        6,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable
fn test_missing_variable_1() {
    test_fail(
        "missing_variable_error_1.ura",
        Error::VariableNotDefinedError,
        5,
        1,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable but in other line than 0
fn test_missing_variable_2() {
    test_fail(
        "missing_variable_error_2.ura",
        Error::VariableNotDefinedError,
        19,
        2,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable but in other line than 0
fn test_missing_variable_3() {
    test_fail(
        "missing_variable_error_3.ura",
        Error::VariableNotDefinedError,
        33,
        7,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable inside a basic string
fn test_missing_variable_4() {
    test_fail(
        "missing_variable_error_4.ura",
        Error::VariableNotDefinedError,
        17,
        1,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable inside a multiline basic string
fn test_missing_variable_5() {
    test_fail(
        "missing_variable_error_5.ura",
        Error::VariableNotDefinedError,
        24,
        2,
    );
}

#[test]
/// Tests error position and line when user uses a non defined variable inside an import statement
fn test_missing_variable_6() {
    test_fail(
        "missing_variable_error_6.ura",
        Error::VariableNotDefinedError,
        21,
        1,
    );
}

#[test]
/// Tests error position and line when imported files are duplicated
fn test_duplicated_import_1() {
    test_fail("importing_error_1.ura", Error::DuplicatedImportError, 74, 2);
}

#[test]
/// Tests error position and line when imported files are duplicated but in other line than 0
fn test_duplicated_import_2() {
    test_fail("importing_error_2.ura", Error::DuplicatedImportError, 86, 5);
}

/// Tests issue https://github.com/gura-conf/gura/issues/12
#[test]
fn test_array_issue_12() {
    test_fail("issue_12.ura", Error::InvalidIndentationError, 0, 2);
}
