use gura_rs::parser::{parse, GuraType};
use std::fs;

pub fn get_file_content_parsed(
    parent_folder: &str,
    file_path: &str,
) -> Result<GuraType, Box<dyn std::error::Error>> {
    println!(
        "abriendo {}",
        format!("tests/{}/tests-files/{}", parent_folder, file_path)
    );
    let content =
        fs::read_to_string(format!("tests/{}/tests-files/{}", parent_folder, file_path)).unwrap();
    parse(&content)
}
