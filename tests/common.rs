use gura::{parse, GuraType};
use std::fs;

/// Reads a file located in tests/{parent_folder}/tests-files/{file_path} and parses its
/// content to Gura format
pub fn get_file_content_parsed(
    parent_folder: &str,
    file_path: &str,
) -> Result<GuraType, Box<dyn std::error::Error>> {
    let content =
        fs::read_to_string(format!("tests/{}/tests-files/{}", parent_folder, file_path)).unwrap();
    parse(&content)
}
