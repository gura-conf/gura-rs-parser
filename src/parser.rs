use crate::errors::{
    DuplicatedImportError, DuplicatedKeyError, DuplicatedVariableError, FileNotFoundError,
    InvalidIndentationError, ParseError, ValueError, VariableNotDefinedError,
};
use crate::pretty_print_float::PrettyPrintFloatWithFallback;
use indexmap::IndexMap;
use itertools::Itertools;
use std::fs;
use std::path::Path;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    error::Error,
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt,
    ops::Index,
    usize, vec,
};
use unicode_segmentation::UnicodeSegmentation;

/// Number chars
const BASIC_NUMBERS_CHARS: &str = "0-9";
const HEX_OCT_BIN: &str = "A-Fa-fxob";
const INF_AND_NAN: &str = "in"; // The rest of the chars are defined in hex_oct_bin

// IMPORTANT: '-' char must be last, otherwise it will be interpreted as a range
// const ACCEPTABLE_NUMBER_CHARS: Option<String> = Some(BASIC_NUMBERS_CHARS + &HEX_OCT_BIN + &INF_AND_NAN + "Ee+._-");

/// Acceptable chars for keys
const KEY_ACCEPTABLE_CHARS: &str = "0-9A-Za-z_";

/// New line chars
const NEW_LINE_CHARS: &str = "\n\x0c\x0b\x08";

/// Returns a HashMap with special characters to be escaped
fn escape_sequences<'a>() -> HashMap<&'a str, String> {
    [
        ("b", "\x08".to_string()),
        ("f", "\x0c".to_string()),
        ("n", "\n".to_string()),
        ("r", "\r".to_string()),
        ("t", "\t".to_string()),
        ("\"", "\"".to_string()),
        ("\\", "\\".to_string()),
        ("$", "$".to_string()),
    ]
    .iter()
    .cloned()
    .collect()
}

// Sequences that need escaped when dumping string values
fn sequences_to_scape<'a>() -> HashMap<&'a str, String> {
    [
        ("\x08", "\\b".to_string()),
        ("\x0c", "\\f".to_string()),
        ("\n", "\\n".to_string()),
        ("\r", "\\r".to_string()),
        ("\t", "\\t".to_string()),
        ("\"", "\\\"".to_string()),
        ("\\", "\\\\".to_string()),
        ("$", "\\$".to_string()),
    ]
    .iter()
    .cloned()
    .collect()
}

// Indentation of 4 spaces
const INDENT: &str = "    ";

/// Useful for number parsing
#[derive(Debug, PartialEq, Eq)]
enum NumberType {
    Integer,
    Float,
}

type RuleResult = Result<GuraType, Box<dyn Error>>;
type Rules = Vec<Box<dyn Fn(&mut Input) -> RuleResult>>;

impl Eq for VariableValueType {}

impl PartialEq for VariableValueType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VariableValueType::String(value1), VariableValueType::String(value2)) => {
                value1 == value2
            }
            (VariableValueType::Integer(value1), VariableValueType::Integer(value2)) => {
                value1 == value2
            }
            (VariableValueType::Float(value1), VariableValueType::Float(value2)) => {
                value1.partial_cmp(value2) == Some(Ordering::Equal)
            }
            _ => false,
        }
    }
}

/// Defines all the possible types for a variable: numbers or strings
#[derive(Debug, Clone)]
enum VariableValueType {
    String(String),
    Integer(isize),
    Float(f64),
}

/// Data types to be returned by match expression methods.
#[derive(Debug, Clone, PartialEq)]
pub enum GuraType {
    /// Null values.
    Null,
    /// Indentation (intended to be used internally).
    Indentation(usize),
    /// An empty line (intended to be used internally).
    UselessLine,
    /// Pair of key/value. (intended to be used internally. Users normally uses Object to map key/values).
    Pair(String, Box<GuraType>, usize),
    /// Comment (intended to be used internally).
    Comment,
    /// Importing sentence (intended to be used internally).
    Import(String),
    /// Indicates matching with a variable definition (intended to be used internally).
    Variable,
    // Uses IndexMap as it preserves the order of insertion
    /// Object with information about indentation (intended to be used internally).
    ObjectWithWs(IndexMap<String, Box<GuraType>>, usize),
    /// Object with its key/value pairs.
    Object(IndexMap<String, Box<GuraType>>),
    /// Boolean values.
    Bool(bool),
    /// String values.
    String(String),
    /// Integer values.
    Integer(isize),
    /// Big integer values.
    BigInteger(i128),
    /// Float values.
    Float(f64),
    /// List of Gura values.
    Array(Vec<GuraType>),
    /// Spaces or new line characters (intended to be used internally).
    WsOrNewLine,
    /// Indicates the ending of an object (intended to be used internally).
    BreakParent,
}

impl fmt::Display for GuraType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&dump(self))
    }
}

/// Implements indexing by `&str` to easily access object members:
impl<T> Index<T> for GuraType
where
    T: AsRef<str>,
{
    type Output = GuraType;

    fn index(&self, index: T) -> &GuraType {
        match *self {
            GuraType::Object(ref object) => &object[index.as_ref()],
            _ => &GuraType::Null,
        }
    }
}

/// Implements Eq with primitive types
// TODO: refactor with macros
impl PartialEq<bool> for GuraType {
    fn eq(&self, other: &bool) -> bool {
        match self {
            GuraType::Bool(value) => value == other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for bool {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<isize> for GuraType {
    fn eq(&self, other: &isize) -> bool {
        match self {
            GuraType::Integer(value) => value == other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for isize {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<i32> for GuraType {
    fn eq(&self, other: &i32) -> bool {
        match self {
            GuraType::Integer(value) => (*value as i32) == *other,
            GuraType::BigInteger(value) => (*value as i32) == *other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for i32 {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<i64> for GuraType {
    fn eq(&self, other: &i64) -> bool {
        match self {
            GuraType::Integer(value) => (*value as i64) == *other,
            GuraType::BigInteger(value) => (*value as i64) == *other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for i64 {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<i128> for GuraType {
    fn eq(&self, other: &i128) -> bool {
        match self {
            GuraType::Integer(value) => (*value as i128) == *other,
            GuraType::BigInteger(value) => value == other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for i128 {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<f32> for GuraType {
    fn eq(&self, other: &f32) -> bool {
        match self {
            GuraType::Float(value) => *value == *other as f64,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for f32 {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<f64> for GuraType {
    fn eq(&self, other: &f64) -> bool {
        match self {
            GuraType::Float(value) => (value.is_nan() && other.is_nan()) || value == other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for f64 {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<&str> for GuraType {
    fn eq(&self, other: &&str) -> bool {
        match self {
            GuraType::String(value) => value == *other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for &str {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<String> for GuraType {
    fn eq(&self, other: &String) -> bool {
        match self {
            GuraType::String(value) => *value == *other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for String {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl GuraType {
    pub fn iter(&self) -> Result<indexmap::map::Iter<'_, String, Box<GuraType>>, &str> {
        match self {
            GuraType::Object(hash_map) => Ok(hash_map.iter()),
            _ => Err("This struct is not an object"),
        }
    }

    pub fn iter_mut(&mut self) -> Result<indexmap::map::IterMut<'_, String, Box<GuraType>>, &str> {
        match self {
            GuraType::Object(hash_map) => Ok(hash_map.iter_mut()),
            _ => Err("This struct is not an object"),
        }
    }
}

/// Struct to handle user Input internally
struct Input {
    /// Text as a Vec of Unicode chars (grapheme clusters)
    text: Vec<String>,
    pos: usize,
    line: usize,
    len: usize,
    /// Vec of Grapheme clusters vecs
    cache: HashMap<String, Vec<Vec<String>>>,
    variables: HashMap<String, VariableValueType>,
    indentation_levels: Vec<usize>,
    imported_files: HashSet<String>,
}

impl Input {
    // TODO: replace this with the same logic as restart_params
    fn new() -> Self {
        Input {
            cache: HashMap::new(),
            pos: 0,
            line: 0,
            len: 0,
            text: Vec::new(),
            variables: HashMap::new(),
            indentation_levels: Vec::new(),
            imported_files: HashSet::new(),
        }
    }

    /// Sets the params to start parsing from a specific text.
    ///
    /// # Arguments
    ///
    /// * text - Text to set as the internal text to be parsed.
    fn restart_params(&mut self, text: &str) {
        let graph = get_graphemes_cluster(&text.to_string());
        self.text = graph;
        self.pos = 0;
        self.line = 0;
        self.len = self.text.len();
    }

    /// Removes, if exists, the last indentation level.
    fn remove_last_indentation_level(&mut self) {
        if !self.indentation_levels.is_empty() {
            self.indentation_levels.pop();
        }
    }
}

/// Generates a Vec with every Grapheme cluster from an String
fn get_graphemes_cluster(text: &str) -> Vec<String> {
    UnicodeSegmentation::graphemes(text, true)
        .map(String::from)
        .collect()
}

/// Computes imports and matches the first expression of the file.Finally consumes all the useless lines.
fn start(text: &mut Input) -> RuleResult {
    compute_imports(text, None)?;
    let result = matches(text, vec![Box::new(object)])?;
    eat_ws_and_new_lines(text);
    Ok(result)
}

/// Matches with any primitive or complex type.
fn any_type(text: &mut Input) -> RuleResult {
    let result = maybe_match(text, vec![Box::new(primitive_type)])?;

    if let Some(result) = result {
        Ok(result)
    } else {
        matches(text, vec![Box::new(complex_type)])
    }
}

/// Matches with a primitive value: null, bool, strings(all of the four kind of string), number or variables values.
fn primitive_type(text: &mut Input) -> RuleResult {
    maybe_match(text, vec![Box::new(ws)])?;
    matches(
        text,
        vec![
            Box::new(null),
            Box::new(boolean),
            Box::new(basic_string),
            Box::new(literal_string),
            Box::new(number),
            Box::new(variable_value),
            Box::new(empty_object),
        ],
    )
}

/// Matches with a useless line. A line is useless when it contains only whitespaces
/// and/or a comment finishing in a new line.
fn useless_line(text: &mut Input) -> RuleResult {
    matches(text, vec![Box::new(ws)])?;
    let comment = maybe_match(text, vec![Box::new(comment)])?;
    let initial_line = text.line;
    maybe_match(text, vec![Box::new(new_line)])?;
    let is_new_line = (text.line - initial_line) == 1;

    if comment.is_none() && !is_new_line {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            String::from("It is a valid line"),
        )));
    }

    Ok(GuraType::UselessLine)
}

/// Matches with a list or an object.
fn complex_type(text: &mut Input) -> RuleResult {
    matches(text, vec![Box::new(list), Box::new(object)])
}

/// Consumes `null` keyword and returns null.
fn null(text: &mut Input) -> RuleResult {
    keyword(text, &["null"])?;
    Ok(GuraType::Null)
}

/// Consumes `empty` keyword and returns an empty object.
fn empty_object(text: &mut Input) -> RuleResult {
    keyword(text, &["empty"])?;
    Ok(GuraType::Object(IndexMap::new()))
}

/// Matches boolean values.
fn boolean(text: &mut Input) -> RuleResult {
    let value = keyword(text, &["true", "false"])? == "true";
    Ok(GuraType::Bool(value))
}

/// Matches with a simple / multiline basic string.
fn basic_string(text: &mut Input) -> RuleResult {
    let quote = keyword(text, &["\"\"\"", "\""])?;

    let is_multiline = quote == "\"\"\"";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")))?;
    }

    let mut final_string: String = String::new();

    let escape_sequences_map = escape_sequences();

    loop {
        let closing_quote = maybe_keyword(text, &[&quote])?;
        if closing_quote.is_some() {
            break;
        }

        let current_char = char(text, &None)?;
        if current_char == "\\" {
            let escape = char(text, &None)?;

            // Checks backslash followed by a newline to trim all whitespaces
            if is_multiline && escape == "\n" {
                eat_ws_and_new_lines(text)
            } else {
                // Supports Unicode of 16 and 32 bits representation
                if escape == "u" || escape == "U" {
                    let num_chars_code_point = if escape == "u" { 4 } else { 8 };
                    let mut code_point: String = String::with_capacity(num_chars_code_point);
                    for _ in 0..num_chars_code_point {
                        let code_point_char = char(text, &Some(String::from("0-9a-fA-F")))?;
                        code_point.push_str(&code_point_char);
                    }

                    // Gets hex value and gets the corresponding char
                    let hex_value = u32::from_str_radix(&code_point, 16);
                    match hex_value {
                        Err(_) => {
                            return Err(Box::new(ParseError::new(
                                text.pos,
                                text.line,
                                String::from("Bad hex value"),
                            )));
                        }
                        Ok(hex_value) => {
                            let char_value = char::from_u32(hex_value).unwrap(); // Converts from UNICODE to string
                            final_string.push(char_value)
                        }
                    };
                } else {
                    // Gets escaped char
                    let escaped_char = match escape_sequences_map.get(escape.as_str()) {
                        Some(good_escape_char) => good_escape_char,
                        None => &current_char,
                    };
                    final_string.push_str(escaped_char);
                }
            }
        } else {
            // Computes variables values in string
            if current_char == "$" {
                let var_name = get_var_name(text)?;
                let var_value_str: String = match get_variable_value(text, &var_name)? {
                    GuraType::Integer(number) => number.to_string(),
                    GuraType::Float(number) => number.to_string(),
                    GuraType::String(value) => value,
                    _ => "".to_string(),
                };

                final_string.push_str(&var_value_str);
            } else {
                final_string.push_str(&current_char);
            }
        }
    }

    Ok(GuraType::String(final_string))
}

/// Gets a variable name char by char.
fn get_var_name(text: &mut Input) -> Result<String, Box<dyn Error>> {
    let key_acceptable_chars = Some(String::from(KEY_ACCEPTABLE_CHARS));
    let mut var_name = String::new();
    while let Some(var_name_char) = maybe_char(text, &key_acceptable_chars)? {
        var_name.push_str(&var_name_char);
    }

    Ok(var_name)
}

/// Computes all the import sentences in Gura file taking into consideration relative paths to imported files.
///
/// # Arguments
///
/// * parentDirPath - Current parent directory path to join with imported files.
/// * importedFiles - Set with already imported files to raise an error in case of importing the same file more than once.
///
/// Returns a set with imported files after all the imports to reuse in the importation process of the imported Gura files.
fn compute_imports(
    text: &mut Input,
    parent_dir_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut files_to_import: Vec<(String, Option<String>)> = Vec::new();

    // First, consumes all the import sentences to replace all of them
    while text.pos < text.len {
        let match_result = maybe_match(
            text,
            vec![
                Box::new(gura_import),
                Box::new(variable),
                Box::new(useless_line),
            ],
        )?;
        if match_result.is_none() {
            break;
        }

        // Checks, it could be a comment
        if let Some(GuraType::Import(file_to_import)) = match_result {
            files_to_import.push((file_to_import, parent_dir_path.clone()));
        }
    }

    let mut final_content = String::new();

    if !files_to_import.is_empty() {
        for (mut file_to_import, origin_file_path) in files_to_import {
            // Gets the final file path considering parent directory
            if let Some(origin_path) = origin_file_path {
                file_to_import = Path::new(&origin_path)
                    .join(&file_to_import)
                    .to_string_lossy()
                    .to_string();
            }

            // Files can be imported only once. This prevents circular reference
            if text.imported_files.contains(&file_to_import) {
                return Err(Box::new(DuplicatedImportError::new(format!(
                    "The file {} has been already imported",
                    file_to_import
                ))));
            }

            // Checks if file exists
            let file_exists = fs::metadata(&file_to_import).is_ok();
            if !file_exists {
                return Err(Box::new(FileNotFoundError::new(format!(
                    "The file {} does not exist",
                    file_to_import
                ))));
            }

            // Gets content considering imports
            let content = fs::read_to_string(&file_to_import).unwrap();
            let parent_dir_path = Path::new(&file_to_import).parent().unwrap();
            let mut empty_input = Input::new();
            let content_with_import = get_text_with_imports(
                &mut empty_input,
                &content,
                parent_dir_path.to_str().unwrap().to_owned(),
            )?;

            final_content.push_str(&(content_with_import.iter().cloned().collect::<String>()));
            final_content.push('\n');

            text.imported_files.insert(file_to_import);
        }

        // Sets as new text
        let rest_of_content = get_string_from_slice(&text.text[text.pos..]);

        text.restart_params(&(final_content + &rest_of_content));
    }

    Ok(())
}

/// Matches with an already defined variable and gets its value.
fn variable_value(text: &mut Input) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    keyword(text, &["$"])?;

    // TODO: refactor with if let
    match matches(text, vec![Box::new(unquoted_string)])? {
        GuraType::String(key_name) => {
            let var_value = get_variable_value(text, &key_name)?;
            Ok(var_value)
        }
        _ => Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Invalid variable name"),
        ))),
    }
}

/// Checks that the parser has reached the end of file, otherwise it will raise a `ParseError`.
///
/// # Errors
///
/// * ParseError - If EOL has not been reached.
fn assert_end(text: &mut Input) -> Result<(), ParseError> {
    if text.pos < text.len {
        Err(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!("Expected end of string but got '{}'", text.text[text.pos]),
        ))
    } else {
        Ok(())
    }
}

/// Generates a String from a slice of Strings (Grapheme clusters)
fn get_string_from_slice(slice: &[String]) -> String {
    slice.iter().cloned().collect()
}

/// Generates a list of char from a list of char which could container char ranges (i.e. a-z or 0-9).
///
/// Returns a Vec of Grapheme clusters vectors.
fn split_char_ranges(text: &mut Input, chars: &str) -> Result<Vec<Vec<String>>, ValueError> {
    if text.cache.contains_key(chars) {
        return Ok(text.cache.get(chars).unwrap().to_vec());
    }

    let chars_graph = get_graphemes_cluster(chars);
    let length = chars_graph.len();
    let mut result: Vec<Vec<String>> = Vec::new();
    let mut index = 0;

    while index < length {
        if index + 2 < length && chars_graph[index + 1] == "-" {
            if chars_graph[index] >= chars_graph[index + 2] {
                return Err(ValueError {});
            }

            let some_chars = &chars_graph[index..index + 3];
            result.push(some_chars.to_vec());
            index += 3;
        } else {
            // Array of one char
            result.push(vec![chars_graph[index].clone()]);
            index += 1;
        }
    }

    text.cache.insert(chars.to_string(), result.clone());
    Ok(result)
}

/// Matches a list of specific chars and returns the first that matched. If any matched, it will raise a `ParseError`.
///
/// `chars` argument can be a range like "a-zA-Z" and they will be properly handled.
fn char(text: &mut Input, chars: &Option<String>) -> Result<String, Box<dyn Error>> {
    if text.pos >= text.len {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected {} but got end of string",
                match chars {
                    None => String::from("next character"),
                    Some(chars) => format!("'{}'", chars),
                }
            ),
        )));
    }

    match chars {
        None => {
            let next_char = &text.text[text.pos];
            text.pos += 1;
            Ok(next_char.to_string())
        }
        Some(chars_value) => {
            for char_range in split_char_ranges(text, chars_value)? {
                if char_range.len() == 1 {
                    let next_char = &text.text[text.pos];
                    if *next_char == char_range[0] {
                        text.pos += 1;
                        return Ok(next_char.to_string());
                    }
                } else if char_range.len() == 3 {
                    let next_char = &text.text[text.pos];
                    let bottom = &char_range[0];
                    let top = &char_range[2];
                    if bottom <= next_char && next_char <= top {
                        text.pos += 1;
                        return Ok(next_char.to_string());
                    }
                }
            }

            return Err(Box::new(ParseError::new(
                // text.pos + 1,
                text.pos,
                text.line,
                format!(
                    "Expected '{}' but got '{}'",
                    format!("[{}]", chars_value),
                    text.text[text.pos]
                ),
            )));
        }
    }
}

/// Matches specific keywords. If any matched, it will raise a `ParseError`.
fn keyword(text: &mut Input, keywords: &[&str]) -> Result<String, Box<dyn Error>> {
    if text.pos >= text.len {
        return Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            format!(
                "Expected '{}' but got end of string",
                keywords.iter().join(",")
            ),
        )));
    }

    for keyword in keywords {
        // let low = text.pos + 1;
        let low = text.pos;
        let high = low + keyword.len();
        // This checking prevents index out of range
        if high <= text.len {
            let substring = get_string_from_slice(&text.text[low..high]);
            if substring == *keyword {
                text.pos += keyword.len();
                return Ok(keyword.to_string());
            }
        }
    }

    Err(Box::new(ParseError::new(
        // text.pos + 1,
        text.pos,
        text.line,
        format!(
            "Expected '{}' but got '{}'",
            keywords.iter().join(", "),
            text.text[text.pos]
        ),
    )))
}

/// Matches specific rules. A rule does not match if its method raises `ParseError`.
///
/// Returns the first matched rule method's result.
fn matches(text: &mut Input, rules: Rules) -> RuleResult {
    let mut last_error_pos: Option<usize> = None;
    let mut last_exception: Option<Box<dyn Error>> = None;

    for rule in rules {
        let initial_pos = text.pos;
        match rule(text) {
            Err(an_error) => {
                // Only considers ParseError instances
                if let Some(parse_error) = an_error.downcast_ref::<ParseError>() {
                    text.pos = initial_pos;

                    if last_error_pos.is_none() || parse_error.pos > last_error_pos.unwrap() {
                        last_error_pos = Some(parse_error.pos);
                        last_exception = Some(an_error);
                    }
                } else {
                    // Any other kind of exception must be raised
                    return Err(an_error);
                }
            }
            result => return result,
        }
    }

    // Unwrap is safe as if this line is reached no rule matched
    Err(last_exception.unwrap())
}

// TODO: consider changing chars: &Option<&str>
/// Like char() but returns None instead of raising ParseError
fn maybe_char(text: &mut Input, chars: &Option<String>) -> Result<Option<String>, Box<dyn Error>> {
    match char(text, chars) {
        Err(e) => {
            if e.downcast_ref::<ParseError>().is_some() {
                Ok(None)
            } else {
                Err(e)
            }
        }
        result => Ok(result.ok()),
    }
}

/// Like match() but returns None instead of raising ParseError
fn maybe_match(text: &mut Input, rules: Rules) -> Result<Option<GuraType>, Box<dyn Error>> {
    match matches(text, rules) {
        Err(e) => {
            if e.downcast_ref::<ParseError>().is_some() {
                Ok(None)
            } else {
                Err(e)
            }
        }
        result => Ok(result.ok()),
    }
}

/// Like keyword() but returns None instead of raising ParseError
fn maybe_keyword(text: &mut Input, keywords: &[&str]) -> Result<Option<String>, Box<dyn Error>> {
    match keyword(text, keywords) {
        Err(e) => {
            if e.downcast_ref::<ParseError>().is_some() {
                Ok(None)
            } else {
                Err(e)
            }
        }
        result => Ok(result.ok()),
    }
}

/// Converts a GuraType::ObjectWithWs in GuraType::Object.
/// Any other types are returned as they are
fn object_ws_to_simple_object(object: GuraType) -> GuraType {
    if let GuraType::ObjectWithWs(values, _) = object {
        GuraType::Object(values)
    } else {
        object
    }
}

/// Parses a text in Gura format.
///
/// # Examples
///
/// ```
/// use gura::parse;
///
/// let gura_string = "
/// title: \"Gura Example\"
/// number: 13.4
/// an_object:
///     name: \"John\"
///     surname: \"Wick\"
///     has_pet: false
/// ".to_string();
///
/// let parsed = parse(&gura_string).unwrap();
///
/// assert_eq!("Gura Example", parsed["title"]);
/// assert_eq!(13.4, parsed["number"]);
///
/// let obj = &parsed["an_object"];
/// assert_eq!("John", obj["name"]);
/// assert_eq!("Wick", obj["surname"]);
/// assert_eq!(false, obj["has_pet"]);
/// ```
///
/// # Errors
///
/// This function could throw any kind of error listed
/// in [Gura specs](https://gura.netlify.app/docs/gura#standard-errors).
pub fn parse(text: &str) -> RuleResult {
    let text_parser: &mut Input = &mut Input::new();
    text_parser.restart_params(text);
    let result = start(text_parser)?;
    assert_end(text_parser)?;

    // Only objects are valid as final result
    match result {
        GuraType::ObjectWithWs(values, _) => Ok(GuraType::Object(values)),
        _ => Ok(GuraType::Object(IndexMap::new())),
    }
}

/// Matches with a new line. I.e any of the following chars:
/// * \n - U+000A
/// * \f - U+000C
/// * \v - U+000B
/// * \r - U+0008
fn new_line(text: &mut Input) -> RuleResult {
    let new_line_chars = Some(String::from(NEW_LINE_CHARS));
    let res = char(text, &new_line_chars);
    if res.is_ok() {
        text.line += 1;
    }

    Ok(GuraType::WsOrNewLine)
}

/// Matches with a comment.
fn comment(text: &mut Input) -> RuleResult {
    keyword(text, &["#"])?;
    while text.pos < text.len {
        let char = &text.text[text.pos];
        text.pos += 1;
        if String::from("\x0c\x0b\r\n").contains(char) {
            text.line += 1;
            break;
        }
    }

    Ok(GuraType::Comment)
}

/// Matches with white spaces taking into consideration indentation levels.
fn ws_with_indentation(text: &mut Input) -> RuleResult {
    let mut current_indentation_level = 0;

    while text.pos < text.len {
        match maybe_keyword(text, &[" ", "\t"])? {
            // If it is not a blank or new line, returns from the method
            None => break,
            Some(blank) => {
                // Tabs are not allowed
                if blank == "\t" {
                    return Err(Box::new(InvalidIndentationError::new(String::from(
                        "Tabs are not allowed to define indentation blocks",
                    ))));
                }

                current_indentation_level += 1
            }
        }
    }

    Ok(GuraType::Indentation(current_indentation_level))
}

/// Matches white spaces (blanks and tabs).
fn ws(text: &mut Input) -> RuleResult {
    while maybe_keyword(text, &[" ", "\t"])?.is_some() {
        continue;
    }

    Ok(GuraType::WsOrNewLine)
}

/// Matches with a quoted string(with a single quotation mark) taking into consideration a variable inside it.
/// There is no special character escaping here.
fn quoted_string_with_var(text: &mut Input) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    let quote = keyword(text, &["\""])?;
    let mut final_string = String::new();

    loop {
        let current_char = char(text, &None)?;

        if current_char == quote {
            break;
        }

        // Computes variables values in string
        if current_char == "$" {
            let var_name = get_var_name(text)?;
            match get_variable_value(text, &var_name) {
                Ok(some_var) => {
                    let var_value: String = match some_var {
                        GuraType::String(var_value_str) => var_value_str.to_string(),
                        GuraType::Integer(var_value_number) => var_value_number.to_string(),
                        GuraType::Float(var_value_number) => var_value_number.to_string(),
                        _ => "".to_string(),
                    };
                    final_string.push_str(&var_value);
                }
                _ => {
                    return Err(Box::new(VariableNotDefinedError::new(format!(
                        "{} is not defined",
                        var_name
                    ))));
                }
            }
        } else {
            final_string.push_str(&current_char);
        }
    }

    Ok(GuraType::String(final_string))
}

/// Consumes all the whitespaces and new lines.
fn eat_ws_and_new_lines(text: &mut Input) {
    let ws_and_new_lines_chars = Some(" ".to_owned() + NEW_LINE_CHARS);
    while let Ok(Some(_)) = maybe_char(text, &ws_and_new_lines_chars) {
        continue;
    }
}

/// Gets a variable value for a specific key from defined variables in file or as environment variable.
///
/// # Arguments
///
/// * key - Key to retrieve.
///
/// # Errors
///
/// * VariableNotDefinedError - If the variable is not defined in file nor environment.
fn get_variable_value(text: &mut Input, key: &str) -> RuleResult {
    match text.variables.get(key) {
        Some(ref value) => match value {
            VariableValueType::Integer(number_value) => Ok(GuraType::Integer(*number_value)),
            VariableValueType::Float(number_value) => Ok(GuraType::Float(*number_value)),
            VariableValueType::String(str_value) => Ok(GuraType::String(str_value.clone())),
        },
        _ => match env::var(key.to_string()) {
            Ok(value) => Ok(GuraType::String(value)),
            Err(_) => Err(Box::new(VariableNotDefinedError::new(format!(
                "Variable '{}' is not defined in Gura nor as environment variable",
                key
            )))),
        },
    }
}

/// Gets final text taking in consideration imports in original text.
/// Returns Final text with imported files' text on it and a HashSet with imported files.
///
/// # Arguments
///
/// * originalText - Text to be parsed.
/// * parentDirPath - Parent directory to keep relative paths reference.
/// * importedFiles - Set with imported files to check if any was imported more than once.
fn get_text_with_imports(
    text: &mut Input,
    original_text: &str,
    parent_dir_path: String,
) -> Result<Vec<String>, Box<dyn Error>> {
    text.restart_params(original_text);
    compute_imports(text, Some(parent_dir_path))?;
    Ok(text.text.clone())
}

/// Matches import sentence.
fn gura_import(text: &mut Input) -> RuleResult {
    keyword(text, &["import"])?;
    char(text, &Some(String::from(" ")))?;
    let string_match = matches(text, vec![Box::new(quoted_string_with_var)])?;

    if let GuraType::String(file_to_import) = string_match {
        matches(text, vec![Box::new(ws)])?;
        maybe_match(text, vec![Box::new(new_line)])?;
        Ok(GuraType::Import(file_to_import))
    } else {
        Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Gura import invalid"),
        )))
    }
}

/// Matches with a variable definition. Returns a Match result indicating that a variable has been added.
///
/// # Errors
///
/// * DuplicatedVariableError - If the current variable has been already defined.
fn variable(text: &mut Input) -> RuleResult {
    keyword(text, &["$"])?;
    let matched_key = matches(text, vec![Box::new(key)])?;

    if let GuraType::String(key_value) = matched_key {
        maybe_match(text, vec![Box::new(ws)])?;

        let match_result = matches(
            text,
            vec![
                Box::new(basic_string),
                Box::new(literal_string),
                Box::new(number),
                Box::new(variable_value),
            ],
        )?;

        // Checks duplicated
        if text.variables.contains_key(&key_value) {
            return Err(Box::new(DuplicatedVariableError::new(format!(
                "Variable '{}' has been already declared",
                key_value
            ))));
        }

        let final_var_value: VariableValueType = match match_result {
            GuraType::String(var_value) => VariableValueType::String(var_value),
            GuraType::Integer(var_value) => VariableValueType::Integer(var_value),
            GuraType::Float(var_value) => VariableValueType::Float(var_value),
            _ => {
                return Err(Box::new(ParseError::new(
                    text.pos,
                    text.line,
                    String::from("Invalid variable value"),
                )));
            }
        };

        // Store as variable
        text.variables.insert(key_value, final_var_value);
        Ok(GuraType::Variable)
    } else {
        Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Key not found"),
        )))
    }
}

/// Matches with a key.A key is an unquoted string followed by a colon (:).
///
/// # Errors
///
/// * ParseError - If key is not a valid string.
fn key(text: &mut Input) -> RuleResult {
    let matched_key = matches(text, vec![Box::new(unquoted_string)]);

    if matched_key.is_ok() {
        // TODO: try char
        keyword(text, &[":"])?;
        matched_key
    } else {
        let min_position = text.pos.min(text.len - 1);
        Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!("Expected string but got '{}'", text.text[min_position]),
        )))
    }
}

/// Gets the last indentation level or null in case it does not exist.
fn get_last_indentation_level(text: &mut Input) -> Option<usize> {
    if text.indentation_levels.is_empty() {
        None
    } else {
        Some(text.indentation_levels[text.indentation_levels.len() - 1])
    }
}

/// Parses an unquoted string.Useful for keys.
fn unquoted_string(text: &mut Input) -> RuleResult {
    let key_acceptable_chars = Some(String::from(KEY_ACCEPTABLE_CHARS));
    let mut chars = vec![char(text, &key_acceptable_chars)?];

    loop {
        let matched_char = maybe_char(text, &key_acceptable_chars)?;
        match matched_char {
            Some(a_char) => chars.push(a_char),
            None => break,
        };
    }

    let trimmed_str = chars
        .iter()
        .cloned()
        .collect::<String>()
        .trim_end()
        .to_string();

    Ok(GuraType::String(trimmed_str))
}

/// Parses a string checking if it is a number and get its correct value.
///
/// # Errors
///
/// * ParseError - If the extracted string is not a valid number.
fn number(text: &mut Input) -> RuleResult {
    let acceptable_number_chars: Option<String> =
        Some(BASIC_NUMBERS_CHARS.to_string() + HEX_OCT_BIN + INF_AND_NAN + "Ee+._-");

    let mut number_type = NumberType::Integer;

    let mut chars = char(text, &acceptable_number_chars)?;

    loop {
        let matched_char = maybe_char(text, &acceptable_number_chars)?;
        match matched_char {
            Some(a_char) => {
                if String::from("Ee.").contains(&a_char) {
                    number_type = NumberType::Float
                }

                chars.push_str(&a_char);
            }
            None => break,
        };
    }

    // Replaces underscores as Rust does not support them in the same way Gura does
    let result = chars.trim_end().replace("_", "");

    // Checks hexadecimal, octal and binary format
    let prefix = result.get(0..2).unwrap_or("");
    if vec!["0x", "0o", "0b"].contains(&prefix) {
        let base: u32;
        let without_prefix = result[2..].to_string();
        match prefix {
            "0x" => base = 16,
            "0o" => base = 8,
            _ => base = 2,
        };

        let int_value = isize::from_str_radix(&without_prefix, base).unwrap();
        return Ok(GuraType::Integer(int_value));
    }

    // Checks inf or NaN
    // Checks for length to prevent 'attempt to subtract with overflow' error
    let result_len = result.len();
    let last_three_chars = if result_len >= 3 {
        &result[result_len - 3..result_len]
    } else {
        ""
    };

    match last_three_chars {
        "inf" => Ok(GuraType::Float(if result.starts_with('-') {
            NEG_INFINITY
        } else {
            INFINITY
        })),
        "nan" => Ok(GuraType::Float(NAN)),
        _ => {
            // It's a normal number
            if number_type == NumberType::Integer {
                if let Ok(value) = result.parse::<isize>() {
                    return Ok(GuraType::Integer(value));
                } else {
                    // Tries 128 bit integer
                    if let Ok(value) = result.parse::<i128>() {
                        return Ok(GuraType::BigInteger(value));
                    }
                }
            } else if number_type == NumberType::Float {
                if let Ok(value) = result.parse::<f64>() {
                    return Ok(GuraType::Float(value));
                }
            }

            Err(Box::new(ParseError::new(
                // text.pos + 1,
                text.pos,
                text.line,
                format!("'{}' is not a valid number", result),
            )))
        }
    }
}

/// Matches with a list.
fn list(text: &mut Input) -> RuleResult {
    let mut result: Vec<GuraType> = Vec::new();

    maybe_match(text, vec![Box::new(ws)])?;
    // TODO: try char
    keyword(text, &["["])?;
    loop {
        // Discards useless lines between elements of array
        match maybe_match(text, vec![Box::new(useless_line)])? {
            Some(_) => continue,
            _ => {
                match maybe_match(text, vec![Box::new(any_type)])? {
                    None => break,
                    Some(GuraType::BreakParent) => (),
                    Some(value) => {
                        let item = object_ws_to_simple_object(value);
                        result.push(item);
                    }
                }

                maybe_match(text, vec![Box::new(ws)])?;
                maybe_match(text, vec![Box::new(new_line)])?;
                // TODO: try char()
                if maybe_keyword(text, &[","])?.is_none() {
                    break;
                }
            }
        }
    }

    maybe_match(text, vec![Box::new(ws)])?;
    maybe_match(text, vec![Box::new(new_line)])?;
    // TODO: try char()
    keyword(text, &["]"])?;
    Ok(GuraType::Array(result))
}

/// Matches with a simple/multiline literal string.
fn literal_string(text: &mut Input) -> RuleResult {
    let quote = keyword(text, &["'''", "'"])?;

    let is_multiline = quote == "'''";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")))?;
    }

    let mut final_string = String::new();

    loop {
        match maybe_keyword(text, &[&quote])? {
            Some(_) => break,
            _ => {
                let matched_char = char(text, &None)?;
                final_string.push_str(&matched_char);
            }
        }
    }

    Ok(GuraType::String(final_string))
}

/// Matches with a Gura object.
///
/// # Errors
///
/// * DuplicatedKeyError - If any of the defined key was declared more than once.
fn object(text: &mut Input) -> RuleResult {
    let mut result: IndexMap<String, Box<GuraType>> = IndexMap::new();
    let mut indentation_level = 0;
    while text.pos < text.len {
        match maybe_match(
            text,
            vec![Box::new(variable), Box::new(pair), Box::new(useless_line)],
        )? {
            None | Some(GuraType::BreakParent) => break,
            Some(GuraType::Pair(key, value, indentation)) => {
                if result.contains_key(&key) {
                    return Err(Box::new(DuplicatedKeyError::new(format!(
                        "The key '{}' has been already defined",
                        key
                    ))));
                }

                result.insert(key, value);
                indentation_level = indentation
            }
            _ => (), // If it's not a pair does nothing!
        }

        if maybe_keyword(text, &["]", ","])?.is_some() {
            // Breaks if it is the end of a list
            text.remove_last_indentation_level();
            text.pos -= 1;
            break;
        }
    }

    if !result.is_empty() {
        Ok(GuraType::ObjectWithWs(result, indentation_level))
    } else {
        Ok(GuraType::BreakParent)
    }
}

/// Matches with a key - value pair taking into consideration the indentation levels.
fn pair(text: &mut Input) -> RuleResult {
    let pos_before_pair = text.pos;

    // TODO: try to simplify
    if let GuraType::Indentation(current_indentation_level) =
        matches(text, vec![Box::new(ws_with_indentation)])?
    {
        let matched_key = matches(text, vec![Box::new(key)])?;

        if let GuraType::String(key_value) = matched_key {
            maybe_match(text, vec![Box::new(ws)])?;

            // Check indentation
            let last_indentation_block = get_last_indentation_level(text);

            // Check if indentation is divisible by 4
            if current_indentation_level % 4 != 0 {
                return Err(Box::new(InvalidIndentationError::new(format!(
                    "Indentation block ({}) must be divisible by 4",
                    current_indentation_level
                ))));
            }

            if let Some(last_indentation_block_val) = last_indentation_block {
                match current_indentation_level.cmp(&last_indentation_block_val) {
                    Ordering::Greater => text.indentation_levels.push(current_indentation_level),
                    Ordering::Less => {
                        text.remove_last_indentation_level();

                        // As the indentation was consumed, it is needed to return to line beginning to get the indentation level
                        // again in the previous matching.Otherwise, the other match would get indentation level = 0
                        text.pos = pos_before_pair;
                        return Ok(GuraType::BreakParent); // This breaks the parent loop
                    }
                    Ordering::Equal => (),
                }
            } else {
                text.indentation_levels.push(current_indentation_level);
            }

            // If it is a BreakParent indicator then is an empty expression, and therefore invalid
            let matched_any = matches(text, vec![Box::new(any_type)])?;
            let mut result: Box<GuraType> = Box::new(matched_any.clone());
            match matched_any {
                GuraType::BreakParent => {
                    return Err(Box::new(ParseError::new(
                        // text.pos + 1,
                        text.pos,
                        text.line,
                        String::from("Invalid pair"),
                    )));
                }
                GuraType::ObjectWithWs(object_values, indentation_level) => {
                    if indentation_level == current_indentation_level {
                        return Err(Box::new(InvalidIndentationError::new(format!(
                            "Wrong level for parent with key {}",
                            key_value
                        ))));
                    } else {
                        let diff = current_indentation_level.max(indentation_level)
                            - current_indentation_level.min(indentation_level);
                        if diff != 4 {
                            return Err(Box::new(InvalidIndentationError::new(String::from(
                                "Difference between different indentation levels must be 4",
                            ))));
                        }
                    }

                    result = Box::new(GuraType::Object(object_values));
                }
                _ => (),
            }

            // Prevents issues with indentation inside a list that break objects
            if let GuraType::Array(_) = *result {
                text.remove_last_indentation_level();
                text.indentation_levels.push(current_indentation_level);
            }

            maybe_match(text, vec![Box::new(new_line)])?;

            Ok(GuraType::Pair(key_value, result, current_indentation_level))
        } else {
            Err(Box::new(ParseError::new(
                text.pos,
                text.line,
                String::from("Invalid key"),
            )))
        }
    } else {
        Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Invalid indentation value"),
        )))
    }
}

/// Auxiliary function for dumping
fn dump_content(content: &GuraType) -> String {
    let sequences_to_scape = sequences_to_scape();

    match content {
        GuraType::Null => "null".to_string(),
        GuraType::String(str_content) => {
            let mut result = String::new();

            // Escapes everything that needs to be escaped
            let content_chars = get_graphemes_cluster(str_content);
            for char in content_chars.into_iter() {
                let char_str = char.as_str();
                let char_to_append = if sequences_to_scape.contains_key(&char_str) {
                    sequences_to_scape.get(&char_str).unwrap().clone()
                } else {
                    char
                };
                result.push_str(&char_to_append);
            }

            format!("\"{}\"", result)
        }
        GuraType::Integer(number) => number.to_string(),
        GuraType::BigInteger(number) => number.to_string(),
        GuraType::Float(number) => {
            let value: String;
            if number.is_nan() {
                value = String::from("nan");
            } else if number.is_infinite() {
                value = if number.is_sign_positive() {
                    String::from("inf")
                } else {
                    String::from("-inf")
                };
            } else {
                value = format!("{}", PrettyPrintFloatWithFallback(*number));
            }

            value
        }
        GuraType::Bool(bool_value) => bool_value.to_string(),
        GuraType::Pair(key, value, _) => format!("{}: {}", key, value),
        GuraType::Object(values) => {
            if values.is_empty() {
                return "empty".to_string();
            }

            let mut result = String::new();
            for (key, gura_value) in values.iter() {
                result.push_str(&format!("{}:", key));

                // If the value is an object, splits the stringified value by
                // newline and indents each line before adding it to the result
                if let GuraType::Object(obj) = &**gura_value {
                    let dumped = dump_content(gura_value);
                    let stringified_value = dumped.trim_end();
                    if !obj.is_empty() {
                        result.push('\n');

                        for line in stringified_value.split('\n') {
                            result.push_str(&format!("{}{}\n", INDENT, line));
                        }
                    } else {
                        // Prevents indentation on empty objects
                        result.push_str(&format!(" {}\n", stringified_value));
                    }
                } else {
                    result.push_str(&format!(" {}\n", dump_content(gura_value)));
                }
            }

            result
        }
        GuraType::Array(array) => {
            // Lists are a special case: if it has an object, and indented representation must be returned. In case
            // of primitive values or nested arrays, a plain representation is more appropriated
            let should_multiline = array.iter().any(|e| {
                if let GuraType::Object(obj) = &*e {
                    !obj.is_empty()
                } else {
                    false
                }
            });

            if !should_multiline {
                let stringify_values: Vec<String> =
                    array.iter().map(|elem| dump_content(elem)).collect();
                let joined = stringify_values.iter().cloned().join(", ");
                return format!("[{}]", joined);
            }

            let mut result = String::from("[");
            let last_idx = array.len() - 1;

            for (idx, elem) in array.iter().enumerate() {
                let dumped = dump_content(elem);
                let stringified_value = dumped.trim_end();

                result.push('\n');

                // If the stringified value contains multiple lines, indents all
                // of them and adds them all to the result
                if stringified_value.contains('\n') {
                    let splitted = stringified_value.split('\n');
                    let splitted: Vec<String> = splitted
                        .map(|element| format!("{}{}", INDENT, element))
                        .collect();
                    result += &splitted.iter().cloned().join("\n");
                } else {
                    // Otherwise indent the value and add to result
                    result.push_str(&format!("{}{}", INDENT, stringified_value));
                }

                // Add a comma if this entry is not the final entry in the list
                if idx < last_idx {
                    result.push(',');
                }
            }

            result.push_str("\n]");
            result
        }
        _ => String::new(),
    }
}

/// Generates a Gura string from a GuraType (aka.stringify).
///
/// # Examples
///
/// ```
/// use gura::{object, dump, GuraType};
///
/// let object = object! {
///     a_number: 55,
///     nested: {
///         array: [1, 2, 3],
///         nested_ar: [1, [2, 3], 4]
///     },
///     a_string: "Gura Rust"
/// };
///
/// let stringified = dump(&object);
///
/// let expected = "a_number: 55
/// nested:
///     array: [1, 2, 3]
///     nested_ar: [1, [2, 3], 4]
/// a_string: \"Gura Rust\"";
///
/// assert_eq!(stringified, expected);
/// ```
pub fn dump(content: &GuraType) -> String {
    dump_content(content).trim().to_string()
}
