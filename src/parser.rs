use crate::errors::{
    DuplicatedKeyError, DuplicatedVariableError, InvalidIndentationError, ParseError, ValueError,
    VariableNotDefinedError,
};
use itertools::Itertools;
use std::collections::hash_map::{Iter, IterMut};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    error::Error,
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt,
    ops::{Add, AddAssign, Deref, Index},
    usize, vec,
};
use unicode_segmentation::UnicodeSegmentation;

// Number chars
const BASIC_NUMBERS_CHARS: &str = "0-9";
const HEX_OCT_BIN: &str = "A-Fa-fxob";
const INF_AND_NAN: &str = "in"; // The rest of the chars are defined in hex_oct_bin

// IMPORTANT: '-' char must be last, otherwise it will be interpreted as a range
// const ACCEPTABLE_NUMBER_CHARS: Option<String> = Some(BASIC_NUMBERS_CHARS + &HEX_OCT_BIN + &INF_AND_NAN + "Ee+._-");

// Acceptable chars for keys
const KEY_ACCEPTABLE_CHARS: &str = "0-9A-Za-z_";

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

#[derive(Debug, Clone)]
pub enum VariableValueType {
    String(String),
    Integer(isize),
    Float(f64),
}

/* Data types to be returned by match expression methods */
// TODO: Rename to Value
#[derive(Debug, Clone, PartialEq)]
pub enum GuraType {
    Null,
    Indentation(usize),
    UselessLine,
    Pair(String, Box<GuraType>, usize),
    Comment,
    Import(String),
    Variable,
    VariableValue(VariableValueType),
    ObjectWithWs(HashMap<String, Box<GuraType>>, usize),
    Object(HashMap<String, Box<GuraType>>),
    Bool(bool),
    String(String),
    Integer(isize),
    Float(f64),
    Array(Vec<Box<GuraType>>),
    WsOrNewLine,
}

impl fmt::Display for GuraType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&dump(self))
    }
}

/// Implements indexing by `&str` to easily access object members:
impl Index<&str> for GuraType {
    type Output = GuraType;

    fn index(&self, index: &str) -> &GuraType {
        match *self {
            GuraType::Object(ref object) => &object[index],
            _ => &GuraType::Null,
        }
    }
}

impl Index<String> for GuraType {
    type Output = GuraType;

    fn index(&self, index: String) -> &GuraType {
        self.index(index.deref())
    }
}

impl Index<&String> for GuraType {
    type Output = GuraType;

    fn index(&self, index: &String) -> &GuraType {
        self.index(index.deref())
    }
}

/// Implements Eq with primitive types
// TODO: refactor with macros
impl PartialEq<bool> for GuraType {
    fn eq(&self, other: &bool) -> bool {
        match self {
            &GuraType::Bool(value) => value == *other,
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
            &GuraType::Integer(value) => value == *other,
            _ => false,
        }
    }
}

impl PartialEq<GuraType> for isize {
    fn eq(&self, other: &GuraType) -> bool {
        other.eq(self)
    }
}

impl PartialEq<f32> for GuraType {
    fn eq(&self, other: &f32) -> bool {
        match self {
            &GuraType::Float(value) => value == *other as f64,
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
            &GuraType::Float(value) => value == *other,
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
    pub fn iter(&self) -> Result<Iter<'_, String, Box<GuraType>>, &str> {
        match self {
            GuraType::Object(hash_map) => Ok(hash_map.iter()),
            _ => Err("This struct is not an object"),
        }
    }

    pub fn iter_mut(&mut self) -> Result<IterMut<'_, String, Box<GuraType>>, &str> {
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

    /**
    	* Sets the params to start parsing from a specific text.
    	*
    	* @param text - Text to set as the internal text to be parsed.
    	*/
    fn restart_params(&mut self, text: &String) {
        let graph = get_graphemes_cluster(text);
        self.text = graph;
        self.pos = 0;
        self.line = 0;
        self.len = self.text.len();
    }

    /**
     * Removes, if exists, the last indentation level.
     */
    fn remove_last_indentation_level(&mut self) {
        if self.indentation_levels.len() > 0 {
            self.indentation_levels.pop();
        }
    }
}

/// Generates a Vec with every Grapheme cluster from an String
fn get_graphemes_cluster(text: &String) -> Vec<String> {
    UnicodeSegmentation::graphemes(text.as_str(), true)
        .map(String::from)
        .collect()
}

/**
* Computes imports and matches the first expression of the file.Finally consumes all the useless lines.
*
* @returns Dict with all the extracted values from Gura string.
*/
fn start(text: &mut Input) -> RuleResult {
    compute_imports(text, None, HashSet::new())?;
    let result = matches(text, vec![Box::new(object)])?;
    eat_ws_and_new_lines(text);
    Ok(result)
}

/**
 * Matches with any primitive or complex type.
 *
 * @returns The corresponding matched value.
 */

fn any_type(text: &mut Input) -> RuleResult {
    let result = maybe_match(text, vec![Box::new(primitive_type)])?;

    if let Some(result) = result {
        Ok(result)
    } else {
        matches(text, vec![Box::new(complex_type)])
    }
}

/**
 * Matches with a primitive value: null, bool, strings(all of the four kind of string), number or variables values.
 *
 * @returns The corresponding matched value.
 */
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
        ],
    )
}

/**
* Matches with a useless line.A line is useless when it contains only whitespaces and / or a comment finishing in a new line.
*
* @throws ParseError if the line contains valid data.
* @returns MatchResult indicating the presence of a useless line.
*/
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

/**
* Matches with a list or another complex expression.
*
* @returns List or Dict, depending the correct matching.
*/
fn complex_type(text: &mut Input) -> RuleResult {
    matches(text, vec![Box::new(list), Box::new(object)])
}

/**
* Consumes null keyword and return null.
*
* @returns Null.
*/
fn null(text: &mut Input) -> RuleResult {
    keyword(text, &vec!["null"])?;
    Ok(GuraType::Null)
}

/**
* Parses boolean values.
*
* @returns Matched boolean value.
*/
fn boolean(text: &mut Input) -> RuleResult {
    let value = keyword(text, &vec!["true", "false"])? == "true";
    Ok(GuraType::Bool(value))
}

/**
 * Matches with a simple / multiline basic string.
 *
 * @returns Matched string.
 */
fn basic_string(text: &mut Input) -> RuleResult {
    let quote = keyword(text, &vec!["\"\"\"", "\""])?;

    let is_multiline = quote == "\"\"\"";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")))?;
    }

    let mut final_string: String = String::new();

    let escape_sequences_map = escape_sequences();

    loop {
        let closing_quote = maybe_keyword(text, &vec![&quote])?;
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
                    final_string.push_str(&escaped_char);
                }
            }
        } else {
            // Computes variables values in string
            if current_char == "$" {
                let var_name = get_var_name(text)?;
                let var_value_str: String = match get_variable_value(text, &var_name)? {
                    VariableValueType::Integer(number) => number.to_string(),
                    VariableValueType::Float(number) => number.to_string(),
                    VariableValueType::String(value) => value,
                };

                final_string.push_str(&var_value_str);
            } else {
                final_string.push_str(&current_char);
            }
        }
    }

    Ok(GuraType::String(final_string))
}

/**
 * Gets a variable name char by char.
 *
 * @returns Variable name.
 */
fn get_var_name(text: &mut Input) -> Result<String, Box<dyn Error>> {
    let key_acceptable_chars = Some(String::from(KEY_ACCEPTABLE_CHARS));
    let mut var_name = String::new();
    while let Some(var_name_char) = maybe_char(text, &key_acceptable_chars)? {
        var_name.push_str(&var_name_char);
    }

    Ok(var_name)
}

/**
* Computes all the import sentences in Gura file taking into consideration relative paths to imported files.
*
* @param parentDirPath - Current parent directory path to join with imported files.
* @param importedFiles - Set with already imported files to raise an error in case of importing the same file
more than once.
* @returns Set with imported files after all the imports to reuse in the importation process of the imported
Gura files.
*/
fn compute_imports(
    text: &mut Input,
    parent_dir_path: Option<String>,
    imported_files: HashSet<String>,
) -> Result<HashSet<String>, ParseError> {
    // TODO: implement after all the rest works
    Ok(HashSet::new())
    // let filesToImport: Vec<(String, Option<String>)> = Vec::new();

    // // First, consumes all the import sentences to replace all of them
    // while text.pos < text.len {
    //   let matchResult = maybe_match([guraImport, variable, uselessLine]);
    //   if matchResult.is_none() {
    //     break;
    //   }

    //   // Checks, it could be a comment
    //   if let Some(GuraType::IMPORT(fileToImport)) = matchResult {
    //     filesToImport.push((fileToImport, parentDirPath));
    //   }
    // }

    // let finalContent = ''
    // if filesToImport.len() > 0 {
    //   for (fileToImport, originFilePath) in filesToImport {
    //     // Gets the final file path considering parent directory
    //     if !originFilePath.is_none() {
    //       let fileToImport = path.join(originFilePath, fileToImport);
    //     }

    //     // Files can be imported only once.This prevents circular reference
    //     if text.importedFiles.has(fileToImport) {
    //        	return Err(DuplicatedImportError::new(
    // 		   format!("The file {} has been already imported", fileToImport)
    // 	   	);
    //     }

    //     // Checks if file exists

    //     // if (!existsSync(fileToImport)) {
    //     //    	return Err(FileNotFoundError::new(
    // 	// 	   format!("The file {} does not exist", fileToImport)
    // 	//    	));
    //     // }

    //     // Gets content considering imports
    //     // let content = readFileSync(fileToImport, "utf-8")
    //     // let auxParser = new GuraParser();
    //     // let parentDirPath = path.dirname(fileToImport);
    //     // let (contentWithImport, importedFiles) = auxParser.getTextWithImports(
    //     //   content,
    //     //   parentDirPath,
    //     //   this.importedFiles
    //     // );
    //     // finalContent += contentWithImport + '\n';
    //     // importedFiles.add(fileToImport);

    //     // text.importedFiles.add(fileToImport);
    //   }

    //   // Sets as new text
    ////   this.restartParams(finalContent + this.text.substring(this.pos + 1))
    //   this.restartParams(finalContent + this.text.substring(this.pos))
    // }

    // return importedFiles
}

/**
 * Matches with an already defined variable and gets its value.
 *
 * @returns Variable value.
 */
fn variable_value(text: &mut Input) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    keyword(text, &vec!["$"])?;

    // TODO: refactor with if let
    match matches(text, vec![Box::new(unquoted_string)])? {
        GuraType::String(key_name) => {
            let var_value = get_variable_value(text, &key_name)?;
            Ok(GuraType::VariableValue(var_value))
        }
        _ => Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Invalid variable name"),
        ))),
    }
}

/// Checks that the parser has reached the end of file, otherwise it will raise a ParseError
/// :raise: ParseError if EOL has not been reached
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

/// Generates a list of char from a list of char which could container char ranges (i.e. a-z or 0-9)
/// :param chars: List of chars to process
/// returns Vec of Grapheme clusters vectors
fn split_char_ranges(text: &mut Input, chars: &String) -> Result<Vec<Vec<String>>, ValueError> {
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

/// Matches a list of specific chars and returns the first that matched. If any matched, it will raise a ParseError
/// :param chars: Chars to match. If it is None, it will return the next char in text
/// :raise: ParseError if any of the specified char (i.e. if chars != None) matched
/// :return: Matched char
fn char(text: &mut Input, chars: &Option<String>) -> Result<String, Box<dyn Error>> {
    if text.pos >= text.len {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected '{}' but got end of string",
                match chars {
                    None => String::from("character"),
                    Some(chars) => format!("{}", chars),
                }
            ),
        )));
    }

    match chars {
        None => {
            let next_char = &text.text[text.pos];
            text.pos += 1;
            return Ok(next_char.to_string());
        }
        Some(chars_value) => {
            for char_range in split_char_ranges(text, &chars_value)? {
                if char_range.len() == 1 {
                    let next_char = &text.text[text.pos];
                    if *next_char == char_range[0] {
                        text.pos += 1;
                        return Ok(next_char.to_string());
                    }
                } else {
                    if char_range.len() == 3 {
                        let next_char = &text.text[text.pos];
                        let bottom = &char_range[0];
                        let top = &char_range[2];
                        if bottom <= next_char && next_char <= top {
                            text.pos += 1;
                            return Ok(next_char.to_string());
                        }
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

/// Matches specific keywords
/// :param keywords: Keywords to match
/// :raise: ParseError if any of the specified keywords matched
/// :return: The first matched keyword
fn keyword(text: &mut Input, keywords: &Vec<&str>) -> Result<String, Box<dyn Error>> {
    if text.pos >= text.len {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
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
        if high < text.len {
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

/// Matches specific rules which name must be implemented as a method in corresponding parser. A rule does not match
/// if its method raises ParseError
/// :param rules: Rules to match
/// :raise: ParseError if any of the specified rules matched
/// :return: The first matched rule method's result
fn matches(text: &mut Input, rules: Rules) -> RuleResult {
    let mut last_error_pos: Option<usize> = None;
    let mut last_exception: Option<Box<dyn Error>> = None;
    // let last_error_rules: Vec<Box<dyn Error>> = Vec::with_capacity(rules.len());

    for rule in rules {
        let initial_pos = text.pos;

        match rule(text) {
            Err(e) => {
                if let Some(err) = e.downcast_ref::<ParseError>() {
                    text.pos = initial_pos;

                    if last_error_pos.is_none() || err.pos > last_error_pos.unwrap() {
                        last_error_pos = Some(err.pos);
                        last_exception = Some(e);
                        // last_error_rules.clear();
                        // last_error_rules.push(rule.)
                    }
                    // else {
                    // 	if err.pos == last_error_pos {
                    // 		last_error_rules.append(rule)
                    // 	}
                    // }
                } else {
                    // Any other kind of exception must be raised
                    return Err(e);
                }
            }
            result => return result,
        }
    }

    // Unwrap is safe as if this line is reached no rule matched
    // Err(last_exception.unwrap())
    Err(last_exception.unwrap())
    // if last_error_rules.len() > 0 {
    // } else {
    //     let last_error_pos = (text.text.len() - 1).min(last_error_pos);
    //     Err(Box::new(ParseError::new(
    //         last_error_pos,
    //         text.line,
    //         format!(
    //             "Expected {} but got {}",
    //             last_error_rules.iter().join(","),
    //             graphene(&text.text).nth(last_error_pos).unwrap()
    //         ),
    //     )))
    // }
}

/// Like char() but returns None instead of raising ParseError
/// :param chars: Chars to match. If it is None, it will return the next char in text
/// :return: Char if matched, None otherwise
// TODO: consider changing chars: &Option<&str>
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
/// :param rules: Rules to match
/// :return: Rule result if matched, None otherwise
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
/// :param keywords: Keywords to match
/// :return: Keyword if matched, None otherwise
fn maybe_keyword(text: &mut Input, keywords: &Vec<&str>) -> Result<Option<String>, Box<dyn Error>> {
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

/**
 * Parses a text in Gura format.
 *
 * @param text - Text to be parsed.
 * @throws ParseError if the syntax of text is invalid.
 * @returns Result with all the parsed values.
 */
pub fn parse(text: &String) -> RuleResult {
    let text_parser: &mut Input = &mut Input::new();
    text_parser.restart_params(text);
    let result = start(text_parser)?;
    assert_end(text_parser)?;

    if let GuraType::ObjectWithWs(values, _) = result {
        Ok(GuraType::Object(values))
    } else {
        Ok(result)
    }
}

/// Matches with a new line. I.e any of the following chars:
/// * \n - U+000A
/// * \f - U+000C
/// * \v - U+000B
/// * \r - U+0008
fn new_line(text: &mut Input) -> RuleResult {
    let new_line_chars = Some(String::from("\n\x0c\x0b\x08"));
    let res = char(text, &new_line_chars);
    if res.is_ok() {
        text.line += 1;
    }

    Ok(GuraType::WsOrNewLine)
}

/**
* Matches with a comment.
*
* @returns MatchResult indicating the presence of a comment.
*/
fn comment(text: &mut Input) -> RuleResult {
    keyword(text, &vec!["#"])?;
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

/**
* Matches with white spaces taking into consideration indentation levels.
*
* @returns Indentation level.
*/
fn ws_with_indentation(text: &mut Input) -> RuleResult {
    let mut current_indentation_level = 0;

    while text.pos < text.len {
        match maybe_keyword(text, &vec![" ", "\t"])? {
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

/**
* Matches white spaces (blanks and tabs).
*/
fn ws(text: &mut Input) -> RuleResult {
    while maybe_keyword(text, &vec![" ", "\t"])?.is_some() {
        continue;
    }

    Ok(GuraType::WsOrNewLine)
}

/**
* Matches with a quoted string(with a single quotation mark) taking into consideration a variable inside it.
* There is no special character escaping here.
*
* @returns Matched string.
*/
fn quoted_string_with_var(text: &mut Input) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    let quote = keyword(text, &vec!["\""])?;
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
                        VariableValueType::String(var_value_str) => var_value_str.to_string(),
                        VariableValueType::Integer(var_value_number) => {
                            var_value_number.to_string()
                        }
                        VariableValueType::Float(var_value_number) => var_value_number.to_string(),
                    };
                    final_string.push_str(&var_value);
                }
                _ => {
                    return Err(Box::new(VariableNotDefinedError::new(String::from(
                        format!("{} is not defined", var_name),
                    ))));
                }
            }
        } else {
            final_string.push_str(&current_char);
        }
    }

    Ok(GuraType::String(final_string))
}

/**
* Consumes all the whitespaces and new lines.
*/
fn eat_ws_and_new_lines(text: &mut Input) {
    let ws_and_new_lines_chars = Some(String::from(" \x0c\x0b\r\n\t"));
    while let Ok(Some(_)) = maybe_char(text, &ws_and_new_lines_chars) {
        continue;
    }
}

/**
* Gets a variable value for a specific key from defined variables in file or as environment variable.
*
* @param key - Key to retrieve.
* @throws VariableNotDefinedError if the variable is not defined in file nor environment.
* @returns Variable value.
*/
fn get_variable_value(text: &mut Input, key: &String) -> Result<VariableValueType, Box<dyn Error>> {
    match text.variables.get(key) {
        Some(ref value) => match value {
            VariableValueType::Integer(number_value) => {
                return Ok(VariableValueType::Integer(*number_value))
            }
            VariableValueType::Float(number_value) => {
                return Ok(VariableValueType::Float(*number_value))
            }
            VariableValueType::String(str_value) => {
                return Ok(VariableValueType::String(str_value.clone()))
            }
        },
        _ => match env::var(key.clone()) {
            Ok(value) => Ok(VariableValueType::String(value)),
            Err(_) => Err(Box::new(VariableNotDefinedError::new(format!(
                "Variable '{}' is not defined in Gura nor as environment variable",
                key
            )))),
        },
    }
}

/**
* Gets final text taking in consideration imports in original text.
*
* @param originalText - Text to be parsed.
* @param parentDirPath - Parent directory to keep relative paths reference.
* @param importedFiles - Set with imported files to check if any was imported more than once.
* @returns Final text with imported files' text on it.
*/
fn get_text_with_imports(
    text: &mut Input,
    original_text: &String,
    parent_dir_path: String,
    imported_files: HashSet<String>,
) -> Result<(Vec<String>, HashSet<String>), ParseError> {
    text.restart_params(original_text);
    let imported_files = compute_imports(text, Some(parent_dir_path), imported_files)?;
    Ok((text.text.clone(), imported_files))
}

/**
* Matches import sentence.
*
* @returns MatchResult with file name of imported file.
*/
fn gura_import(text: &mut Input) -> RuleResult {
    keyword(text, &vec!["import"])?;
    char(text, &Some(String::from(" ")))?;
    let string_match = matches(text, vec![Box::new(quoted_string_with_var)])?;

    if let GuraType::Import(file_to_import) = string_match {
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

/**
 * Matches with a variable definition.
 *
 * @throws DuplicatedVariableError if the current variable has been already defined.
 * @returns Match result indicating that a variable has been added.
 */
fn variable(text: &mut Input) -> RuleResult {
    keyword(text, &vec!["$"])?;
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

        let final_var_value: VariableValueType = match match_result {
            GuraType::String(var_value) => VariableValueType::String(var_value),
            GuraType::Integer(var_value) => VariableValueType::Integer(var_value),
            GuraType::Float(var_value) => VariableValueType::Float(var_value),
            GuraType::VariableValue(var_value) => {
                if text.variables.contains_key(&key_value) {
                    return Err(Box::new(DuplicatedVariableError::new(format!(
                        "Variable '{}' has been already declared",
                        key_value
                    ))));
                }

                var_value
            }
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

/**
* Matches with a key.A key is an unquoted string followed by a colon (:).
*
* @throws ParseError if key is not a valid string.
* @returns Matched key.
*/
fn key(text: &mut Input) -> RuleResult {
    let matched_key = matches(text, vec![Box::new(unquoted_string)]);

    if matched_key.is_ok() {
        // TODO: try char
        keyword(text, &vec![":"])?;
        matched_key
    } else {
        Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!("Expected string but got '{}'", text.text[text.pos]),
        )))
    }
}

/**
* Gets the last indentation level or null in case it does not exist.
*
* @returns Last indentation level or null if it does not exist.
*/
fn get_last_indentation_level(text: &mut Input) -> Option<usize> {
    if text.indentation_levels.len() == 0 {
        None
    } else {
        Some(text.indentation_levels[text.indentation_levels.len() - 1])
    }
}

/**
 * Parses an unquoted string.Useful for keys.
 *
 * @returns Parsed unquoted string.
 */
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

/**
* Parses a string checking if it is a number and get its correct value.
*
* @throws ParseError if the extracted string is not a valid number.
* @returns Returns an number or a float depending of type inference.
*/
fn number(text: &mut Input) -> RuleResult {
    #[derive(Debug, PartialEq, Eq)]
    enum NumberType {
        Integer,
        Float,
    }

    let acceptable_number_chars: Option<String> = Some(String::from(
        BASIC_NUMBERS_CHARS.to_string() + &HEX_OCT_BIN + &INF_AND_NAN + "Ee+._-",
    ));

    let mut number_type = NumberType::Integer;

    let mut chars = char(text, &acceptable_number_chars)?.to_string();

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
        "inf" => {
            return Ok(GuraType::Float(if result.chars().next().unwrap() == '-' {
                NEG_INFINITY
            } else {
                INFINITY
            }));
        }
        "nan" => {
            return Ok(GuraType::Float(NAN));
        }
        _ => {
            // It's a normal number
            if number_type == NumberType::Integer {
                if let Ok(value) = result.parse::<isize>() {
                    return Ok(GuraType::Integer(value));
                }
            } else {
                if number_type == NumberType::Float {
                    if let Ok(value) = result.parse::<f64>() {
                        return Ok(GuraType::Float(value));
                    }
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

/**
 * Matches with a list.
 *
 * @returns Matched list.
 */
fn list(text: &mut Input) -> RuleResult {
    let mut result: Vec<Box<GuraType>> = Vec::new();

    maybe_match(text, vec![Box::new(ws)])?;
    // TODO: try char
    keyword(text, &vec!["["])?;
    loop {
        // Discards useless lines between elements of array
        match maybe_match(text, vec![Box::new(useless_line)])? {
            Some(_) => continue,
            _ => {
                let item: Box<GuraType>;
                match maybe_match(text, vec![Box::new(any_type)])? {
                    None => break,
                    Some(value) => item = Box::new(value),
                }

                result.push(item);

                maybe_match(text, vec![Box::new(ws)])?;
                maybe_match(text, vec![Box::new(new_line)])?;
                // TODO: try char()
                if maybe_keyword(text, &vec![","])?.is_none() {
                    break;
                }
            }
        }
    }

    maybe_match(text, vec![Box::new(ws)])?;
    maybe_match(text, vec![Box::new(new_line)])?;
    // TODO: try char()
    keyword(text, &vec!["]"])?;
    Ok(GuraType::Array(result))
}

/**
* Matches with a simple / multiline literal string.
*
* @returns Matched string.
*/
fn literal_string(text: &mut Input) -> RuleResult {
    let quote = keyword(text, &vec!["'''", "'"])?;

    let is_multiline = quote == "'''";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")))?;
    }

    let mut final_string = String::new();

    loop {
        match maybe_keyword(text, &vec![&quote])? {
            Some(_) => break,
            _ => {
                let matched_char = char(text, &None)?;
                final_string.push_str(&matched_char);
            }
        }
    }

    Ok(GuraType::String(final_string))
}

/**
 * Match any Gura expression.
 *
 * @throws DuplicatedKeyError if any of the defined key was declared more than once.
 * @returns Object with Gura string data.
 */
fn object(text: &mut Input) -> RuleResult {
    let mut result: HashMap<String, Box<GuraType>> = HashMap::new();
    let mut indentation_level = 0;
    while text.pos < text.len {
        match maybe_match(
            text,
            vec![Box::new(variable), Box::new(pair), Box::new(useless_line)],
        )? {
            None | Some(GuraType::Null) => break,
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

        if maybe_keyword(text, &vec!["]", ","])?.is_some() {
            // Breaks if it is the end of a list
            text.remove_last_indentation_level();
            text.pos -= 1;
            break;
        }
    }

    if result.len() > 0 {
        Ok(GuraType::ObjectWithWs(result, indentation_level))
    } else {
        Ok(GuraType::Null)
    }
}

/**
 * Matches with a key - value pair taking into consideration the indentation levels.
 *
 * @returns Matched key - value pair.null if the indentation level is lower than the last one(to indicate the ending of a parent object).
 */
fn pair(text: &mut Input) -> RuleResult {
    let pos_before_pair = text.pos;

    // TODO: try to simplify
    if let GuraType::Indentation(current_indentation_level) =
        matches(text, vec![Box::new(ws_with_indentation)])?
    {
        let matched_key = matches(text, vec![Box::new(key)])?;

        if let GuraType::String(key_value) = matched_key {
            maybe_match(text, vec![Box::new(ws)])?;
            maybe_match(text, vec![Box::new(new_line)])?;

            // Check indentation
            let last_indentation_block = get_last_indentation_level(text);

            // Check if indentation is divisible by 4
            if current_indentation_level % 4 != 0 {
                return Err(Box::new(InvalidIndentationError::new(format!(
                    "Indentation block ({}) must be divisible by 4",
                    current_indentation_level
                ))));
            }

            if last_indentation_block.is_none()
                || current_indentation_level > last_indentation_block.unwrap()
            {
                text.indentation_levels.push(current_indentation_level)
            } else {
                if current_indentation_level < last_indentation_block.unwrap() {
                    text.remove_last_indentation_level();

                    // As the indentation was consumed, it is needed to return to line beginning to get the indentation level
                    // again in the previous matching.Otherwise, the other match would get indentation level = 0
                    text.pos = pos_before_pair;
                    return Ok(GuraType::Null); // This breaks the parent loop
                }
            }

            // If it == null then is an empty expression, and therefore invalid
            let matched_any = matches(text, vec![Box::new(any_type)])?;
            let mut result: Box<GuraType> = Box::new(matched_any.clone());
            match matched_any {
                GuraType::Null => {
                    return Err(Box::new(ParseError::new(
                        // text.pos + 1,
                        text.pos,
                        text.line,
                        String::from("Invalid pair"),
                    )));
                }
                GuraType::ObjectWithWs(object_values, indentation_level) => {
                    if indentation_level == current_indentation_level {
                        return Err(Box::new(InvalidIndentationError::new(String::from(
                            format!("Wrong level for parent with key {}", key_value),
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
            maybe_match(text, vec![Box::new(new_line)])?;

            Ok(GuraType::Pair(key_value, result, current_indentation_level))
        } else {
            return Err(Box::new(ParseError::new(
                text.pos,
                text.line,
                String::from("Invalid key"),
            )));
        }
    } else {
        return Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Invalid indentation value"),
        )));
    }
}

/// Auxiliary function for dumping
fn dump_content(content: &GuraType, indentation_level: usize) -> String {
    let mut result = String::new();
    let indentation = " ".repeat(indentation_level * 4);
    match content {
        GuraType::Null => result.add_assign("null"),
        GuraType::Pair(key, value, _) => result.add_assign(&format!("{}: {}", key, value)),
        GuraType::Object(values) => {
            // Only prevents new line in the first level
            if indentation_level > 0 {
                result.add_assign("\n");
            }

            for (key, value) in values {
                result.add_assign(&format!(
                    "{}{}: {}",
                    indentation,
                    key,
                    dump_content(value, indentation_level + 1)
                ));
            }
        }
        GuraType::Bool(bool_value) => result.add_assign(&bool_value.to_string().add("\n")),
        GuraType::String(str_content) => result.add_assign(&format!("'{}'\n", str_content)),
        GuraType::Integer(number) => result.add_assign(&format!("{}\n", number)),
        GuraType::Float(number) => result.add_assign(&format!("{}\n", number)),
        GuraType::Array(array) => {
            // FIXME: prints array in an ugly way
            result.add_assign("[");
            let joined = array.iter().join(", ");
            result.add_assign(&joined);
            result.add_assign("]\n");
        }
        _ => (),
    };
    result
}

/**
 * Generates a Gura string from a dictionary(aka.stringify).
 *
 * @param data - Object to stringify.
 * @returns String with the data in Gura format.
 */
pub fn dump(content: &GuraType) -> String {
    dump_content(content, 0)
}
