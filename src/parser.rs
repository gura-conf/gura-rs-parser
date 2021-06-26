use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    f64::{INFINITY, NAN, NEG_INFINITY},
    fmt::{self},
    usize, vec,
};

use crate::errors::{
    DuplicatedKeyError, DuplicatedVariableError, InvalidIndentationError, VariableNotDefinedError,
};

// Number chars
const BASIC_NUMBERS_CHARS: &str = "0-9";
const HEX_OCT_BIN: &str = "A-Fa-fxob";
const INF_AND_NAN: &str = "in"; // The rest of the chars are defined in hex_oct_bin

// IMPORTANT: '-' char must be last, otherwise it will be interpreted as a range
// const ACCEPTABLE_NUMBER_CHARS: Option<String> = Some(BASIC_NUMBERS_CHARS + &HEX_OCT_BIN + &INF_AND_NAN + "Ee+._-");

// Acceptable chars for keys
const KEY_ACCEPTABLE_CHARS: &str = "0-9A-Za-z_";

/// Special characters to be escaped
fn escape_sequences() -> HashMap<char, char> {
    [
        ('b', '\x08'),
        ('f', '\x0c'),
        ('n', '\n'),
        ('r', '\r'),
        ('t', '\t'),
        ('"', '"'),
        ('\\', '\\'),
        ('$', '$'),
    ]
    .iter()
    .cloned()
    .collect()
}

// TODO: refactor to another file the errors and types
type RuleResult = Result<GuraType, Box<dyn Error>>;
type Rules = Vec<Box<dyn Fn(&mut Parser) -> RuleResult>>;

// ValueError
#[derive(Debug)]
struct ValueError {}

impl Error for ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad character range")
    }
}

// ParseError
#[derive(Debug, Clone)]
struct ParseError {
    message: String,
    pos: usize,
    line: usize,
}

impl ParseError {
    fn new(pos: usize, line: usize, message: String) -> Self {
        ParseError { pos, line, message }
    }
}

#[derive(Debug, Clone)]
pub enum VariableValueType {
    String(String),
    Number(f64),
}

/* Data types to be returned by match expression methods */
// TODO: change to CamelCase
// TODO: differentiate between all the types and only the valid types for final result map
#[derive(Debug, Clone)]
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
    Integer(u64),
    Float(f64),
    List(Vec<Box<GuraType>>),
    WsOrNewLine,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} at line {} position {}",
            self.message, self.line, self.pos
        )
    }
}

// Base parser
struct Parser {
    text: String,
    pos: usize,
    line: usize,
    len: usize,
    cache: HashMap<String, Vec<String>>,
    variables: HashMap<String, VariableValueType>,
    indentation_levels: Vec<usize>,
    imported_files: HashSet<String>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            cache: HashMap::new(),
            pos: 0,
            line: 0,
            len: 0,
            text: String::new(),
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
        self.text = text.to_string();
        self.pos = 0;
        self.line = 0;
        self.len = text.len() - 1;
    }
}

/**
* Computes imports and matches the first expression of the file.Finally consumes all the useless lines.
*
* @returns Dict with all the extracted values from Gura string.
*/
fn start(text: &mut Parser) -> RuleResult {
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

fn any_type(text: &mut Parser) -> RuleResult {
    let result = maybe_match(text, vec![Box::new(primitive_type)]);

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
fn primitive_type(text: &mut Parser) -> RuleResult {
    maybe_match(text, vec![Box::new(ws)]);
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
fn useless_line(text: &mut Parser) -> RuleResult {
    matches(text, vec![Box::new(ws)])?;
    let comment = maybe_match(text, vec![Box::new(comment)]);
    let initial_line = text.line;
    maybe_match(text, vec![Box::new(new_line)]);
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
fn complex_type(text: &mut Parser) -> RuleResult {
    matches(text, vec![Box::new(list), Box::new(object)])
}

/**
* Consumes null keyword and return null.
*
* @returns Null.
*/
fn null(text: &mut Parser) -> RuleResult {
    keyword(text, &vec![String::from("null")])?;
    Ok(GuraType::Null)
}

/**
* Parses boolean values.
*
* @returns Matched boolean value.
*/
fn boolean(text: &mut Parser) -> RuleResult {
    let value = keyword(text, &vec![String::from("true"), String::from("false")])? == "true";
    Ok(GuraType::Bool(value))
}

/**
 * Matches with a simple / multiline basic string.
 *
 * @returns Matched string.
 */
fn basic_string(text: &mut Parser) -> RuleResult {
    let quote = keyword(text, &vec![String::from("\"\"\""), String::from("\"")])?;

    let is_multiline = quote == "\"\"\"";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")));
    }

    let mut chars: Vec<char> = Vec::new();

    let escape_sequences_map = escape_sequences();

    loop {
        let closing_quote = maybe_keyword(text, &vec![quote.clone()]);
        if closing_quote.is_some() {
            break;
        }

        let current_char = char(text, &None)?;
        if current_char == '\\' {
            let escape = char(text, &None)?;

            // Checks backslash followed by a newline to trim all whitespaces
            if is_multiline && escape == '\n' {
                eat_ws_and_new_lines(text)
            } else {
                // Supports Unicode of 16 and 32 bits representation
                if escape == 'u' || escape == 'U' {
                    let num_chars_code_point = if escape == 'u' { 4 } else { 8 };
                    let mut code_point: Vec<char> = Vec::with_capacity(num_chars_code_point);
                    for _ in 0..num_chars_code_point {
                        let code_point_char = char(text, &Some(String::from("0-9a-fA-F")))?;
                        code_point.push(code_point_char);
                    }
                    let code_point_str = code_point.iter().cloned().collect::<String>();
                    let hex_value = u32::from_str_radix(code_point_str.as_str(), 16);
                    // TODO: fix to use ? instead of this if
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
                            chars.push(char_value)
                        }
                    };
                } else {
                    // Gets escaped char
                    let escaped_char = match escape_sequences_map.get(&escape) {
                        Some(good_escape_char) => good_escape_char.clone(),
                        None => current_char,
                    };
                    chars.push(escaped_char);
                }
            }
        } else {
            // Computes variables values in string
            if current_char == '$' {
                let var_name = get_var_name(text);
                let var_value_str: String = match get_variable_value(text, &var_name)? {
                    VariableValueType::Number(number) => number.to_string(),
                    VariableValueType::String(value) => value,
                };

                let mut chars_vec = var_value_str.chars().collect::<Vec<char>>();
                chars.append(&mut chars_vec)
            } else {
                chars.push(current_char);
            }
        }
    }

    let final_string = chars.iter().cloned().collect::<String>();
    Ok(GuraType::String(
        final_string,
    ))
}

/**
 * Gets a variable name char by char.
 *
 * @returns Variable name.
 */
fn get_var_name(text: &mut Parser) -> String {
    let key_acceptable_chars = Some(String::from(KEY_ACCEPTABLE_CHARS));
    let mut var_name = String::new();
    let mut var_name_char = maybe_char(text, &key_acceptable_chars);
    while var_name_char.is_some() {
        var_name.push(var_name_char.unwrap());
        var_name_char = maybe_char(text, &key_acceptable_chars)
    }

    var_name
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
    text: &mut Parser,
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
fn variable_value(text: &mut Parser) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    keyword(text, &vec![String::from("$")])?;
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
fn assert_end(text: &Parser) -> Result<(), ParseError> {
    if text.pos < text.len {
        Err(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected end of string but got {}",
                // text.text.chars().nth(text.pos + 1).unwrap()
                text.text.chars().nth(text.pos).unwrap()
            ),
        ))
    } else {
        Ok(())
    }
}

/// Generates a list of char from a list of char which could container char ranges (i.e. a-z or 0-9)
/// :param chars: List of chars to process
/// :return: List of char with ranges processed
// FIXME: it's not generating a good split char ranges
fn split_char_ranges(text: &mut Parser, chars: &String) -> Result<Vec<String>, ValueError> {
    if text.cache.contains_key(chars) {
        return Ok(text.cache.get(chars).unwrap().to_vec());
    }

    let length = chars.len();
    // let mut chars_vec = ;
    let mut result: Vec<String> = Vec::new();
    let mut index = 0;

    while index < length {
        if index + 2 < length && chars.chars().nth(index + 1).unwrap() == '-' {
            if chars.chars().nth(index).unwrap() >= chars.chars().nth(index + 2).unwrap() {
                return Err(ValueError {});
            }

            let some_chars = chars.get(index..index + 3).unwrap();
            result.push(some_chars.to_string());
            index += 3;
        } else {
            result.push(chars.chars().nth(index).unwrap().to_string());
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
fn char(text: &mut Parser, chars: &Option<String>) -> Result<char, Box<dyn Error>> {
    // if text.pos >= text.len {
    if text.pos > text.len {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected {} but got end of string",
                match chars {
                    None => String::from("character"),
                    Some(chars) => format!("{}", chars),
                }
            ),
        )));
    }

    // let next_char = text.text.chars().nth(text.pos + 1).unwrap();
    let next_char = text.text.chars().nth(text.pos).unwrap();
    match chars {
        None => {
            text.pos += 1;
            return Ok(next_char);
        }
        Some(chars_value) => {
            for char_range in split_char_ranges(text, &chars_value)? {
                if char_range.len() == 1 {
                    if next_char == char_range.chars().nth(0).unwrap() {
                        text.pos += 1;
                        return Ok(next_char);
                    }
                } else {
                    let bottom = char_range.chars().nth(0).unwrap();
                    let top = char_range.chars().nth(2).unwrap();
                    if bottom <= next_char && next_char <= top {
                        text.pos += 1;
                        return Ok(next_char);
                    }
                }
            }

            return Err(Box::new(ParseError::new(
                // text.pos + 1,
                text.pos,
                text.line,
                format!(
                    "Expected {} but got {}",
                    format!("[{}]", chars_value),
                    next_char
                ),
            )));
        }
    }
}

/// Matches specific keywords
/// :param keywords: Keywords to match
/// :raise: ParseError if any of the specified keywords matched
/// :return: The first matched keyword
// TODO: Change keywords to Vec<&str>
fn keyword(text: &mut Parser, keywords: &Vec<String>) -> Result<String, Box<dyn Error>> {
    // if text.pos >= text.len {
    if text.pos > text.len {
        return Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected {} but got end of string",
                keywords.iter().join(",")
            ),
        )));
    }

    for keyword in keywords {
        // let low = text.pos + 1;
        let low = text.pos;
        let high = low + keyword.len();
        // println!("Keyword -> {} | Text -> {:?} ({} to {})", keyword, text.text.get(low..high), low, high);

        if let Some(current_substring) = text.text.get(low..high) {
            if current_substring == keyword {
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
            "Expected {} but got {}",
            keywords.iter().join(", "),
            // text.text.chars().nth(text.pos + 1).unwrap()
            text.text.chars().nth(text.pos).unwrap()
        ),
    )))
}

/// Matches specific rules which name must be implemented as a method in corresponding parser. A rule does not match
/// if its method raises ParseError
/// :param rules: Rules to match
/// :raise: ParseError if any of the specified rules matched
/// :return: The first matched rule method's result
fn matches(text: &mut Parser, rules: Rules) -> RuleResult {
    let mut last_error_pos: usize = 0;
    let mut last_exception: Option<Box<dyn Error>> = None;
    // let last_error_rules: Vec<Box<dyn Error>> = Vec::with_capacity(rules.len());

    for rule in rules {
        let initial_pos = text.pos;

        match rule(text) {
            Err(e) => {
                if let Some(err) = e.downcast_ref::<ParseError>() {
                    text.pos = initial_pos;

                    if err.pos > last_error_pos {
                        last_error_pos = err.pos;
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
    //             text.text.chars().nth(last_error_pos).unwrap()
    //         ),
    //     )))
    // }
}

/// Like char() but returns None instead of raising ParseError
/// :param chars: Chars to match. If it is None, it will return the next char in text
/// :return: Char if matched, None otherwise
fn maybe_char(text: &mut Parser, chars: &Option<String>) -> Option<char> {
    match char(text, chars) {
        Err(_) => None,
        result => result.ok(),
    }
}

/// Like match() but returns None instead of raising ParseError
/// :param rules: Rules to match
/// :return: Rule result if matched, None otherwise
fn maybe_match(text: &mut Parser, rules: Rules) -> Option<GuraType> {
    match matches(text, rules) {
        Err(_) => None,
        result => result.ok(),
    }
}

/// Like keyword() but returns None instead of raising ParseError
/// :param keywords: Keywords to match
/// :return: Keyword if matched, None otherwise
/// TODO: change to Vec<str>!!
fn maybe_keyword(text: &mut Parser, keywords: &Vec<String>) -> Option<String> {
    match keyword(text, keywords) {
        Err(_) => None,
        result => result.ok(),
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
    let text_parser: &mut Parser = &mut Parser::new();
    text_parser.restart_params(text);
    let result = start(text_parser)?;
    assert_end(text_parser)?;
    
    if let GuraType::ObjectWithWs(values, _) = result {
        Ok(GuraType::Object(values))
    } else {
        Ok(result)
    }
}

/**
* Matches with a new line.
*/
fn new_line(text: &mut Parser) -> RuleResult {
    let new_line_chars = Some(String::from("\x0c\x0b\r\n"));
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
fn comment(text: &mut Parser) -> RuleResult {
    keyword(text, &vec![String::from("#")])?;
    while text.pos < text.len {
        // let char = text.text.chars().nth(text.pos + 1).unwrap();
        let char = text.text.chars().nth(text.pos).unwrap();
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
fn ws_with_indentation(text: &mut Parser) -> RuleResult {
    let mut current_indentation_level = 0;

    while text.pos < text.len {
        let blank = maybe_keyword(text, &vec![String::from(" "), String::from("\t")]);

        if blank.is_none() {
            // If it is not a blank or new line, returns from the method
            break;
        }

        // Tabs are not allowed
        if blank.unwrap() == "\t" {
            return Err(Box::new(InvalidIndentationError::new(String::from(
                "Tabs are not allowed to define indentation blocks",
            ))));
        }

        current_indentation_level += 1
    }

    Ok(GuraType::Indentation(current_indentation_level))
}

/**
* Matches white spaces (blanks and tabs).
*/
fn ws(text: &mut Parser) -> RuleResult {
    while maybe_keyword(text, &vec![String::from(" "), String::from("\t")]).is_some() {
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
fn quoted_string_with_var(text: &mut Parser) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    let quote = keyword(text, &vec![String::from("\"")])?
        .chars()
        .nth(0)
        .unwrap();
    let mut chars: Vec<char> = Vec::new();

    loop {
        let current_char = char(text, &None)?;

        if current_char == quote {
            break;
        }

        // Computes variables values in string
        if current_char == '$' {
            let var_name = get_var_name(text);
            match get_variable_value(text, &var_name) {
                Ok(some_var) => {
                    let mut var_chars: Vec<char> = match some_var {
                        VariableValueType::String(var_value_str) => var_value_str.chars().collect(),
                        VariableValueType::Number(var_value_number) => {
                            var_value_number.to_string().chars().collect()
                        }
                    };
                    chars.append(&mut var_chars);
                }
                _ => {
                    return Err(Box::new(VariableNotDefinedError::new(String::from(
                        format!("{} is not defined", var_name),
                    ))));
                }
            }
        } else {
            chars.push(current_char);
        }
    }

    let final_string = chars.iter().cloned().collect::<String>();
    Ok(GuraType::String(
        final_string,
    ))
}

/**
* Consumes all the whitespaces and new lines.
*/
fn eat_ws_and_new_lines(text: &mut Parser) {
    let ws_and_new_lines_chars = Some(String::from(" \x0c\x0b\r\n\t"));
    while maybe_char(text, &ws_and_new_lines_chars).is_some() {
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
fn get_variable_value(text: &mut Parser, key: &String) -> Result<VariableValueType, Box<dyn Error>> {
    match text.variables.get(key) {
        Some(ref value) => match value {
            VariableValueType::Number(number_value) => {
                return Ok(VariableValueType::Number(*number_value))
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
    text: &mut Parser,
    original_text: &String,
    parent_dir_path: String,
    imported_files: HashSet<String>,
) -> Result<(String, HashSet<String>), ParseError> {
    text.restart_params(original_text);
    let imported_files = compute_imports(text, Some(parent_dir_path), imported_files)?;
    Ok((text.text.clone(), imported_files))
}

/**
* Matches import sentence.
*
* @returns MatchResult with file name of imported file.
*/
fn gura_import(text: &mut Parser) -> RuleResult {
    keyword(text, &vec![String::from("import")])?;
    char(text, &Some(String::from(" ")))?;
    let string_match = matches(text, vec![Box::new(quoted_string_with_var)])?;

    if let GuraType::Import(file_to_import) = string_match {
        matches(text, vec![Box::new(ws)])?;
        maybe_match(text, vec![Box::new(new_line)]);
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
fn variable(text: &mut Parser) -> RuleResult {
    keyword(text, &vec![String::from("$")])?;
    let matched_key = matches(text, vec![Box::new(key)])?;

    if let GuraType::String(key_value) = matched_key {
        maybe_match(text, vec![Box::new(ws)]);

        let match_result = matches(
            text,
            vec![
                Box::new(basic_string),
                Box::new(literal_string),
                Box::new(number),
                Box::new(variable_value),
            ],
        )?;

        if let GuraType::VariableValue(var_value) = match_result {
            if text.variables.contains_key(&key_value) {
                return Err(Box::new(DuplicatedVariableError::new(format!(
                    "Variable '{}' has been already declared",
                    key_value
                ))));
            }

            // Store as variable
            text.variables.insert(key_value, var_value);
            Ok(GuraType::Variable)
        } else {
            Err(Box::new(ParseError::new(
                text.pos,
                text.line,
                String::from("Invalid variable value"),
            )))
        }
    } else {
        Err(Box::new(ParseError::new(
            text.pos,
            text.line,
            String::from("Key not found"),
        )))
    }
}

/**
* Removes, if exists, the last indentation level.
* TODO: put inside struct
*/
fn remove_last_indentation_level(text: &mut Parser) {
    if text.indentation_levels.len() > 0 {
        text.indentation_levels.pop();
    }
}

/**
* Matches with a key.A key is an unquoted string followed by a colon (:).
*
* @throws ParseError if key is not a valid string.
* @returns Matched key.
*/
fn key(text: &mut Parser) -> RuleResult {
    let matched_key = matches(text, vec![Box::new(unquoted_string)]);

    if matched_key.is_ok() {
        // TODO: try char
        keyword(text, &vec![String::from(":")])?;
        matched_key
    } else {
        Err(Box::new(ParseError::new(
            // text.pos + 1,
            text.pos,
            text.line,
            format!(
                "Expected string but got \"{}\"",
                // text.text.chars().nth(text.pos + 1).unwrap()
                text.text.chars().nth(text.pos).unwrap()
            ),
        )))
    }
}

/**
* Gets the last indentation level or null in case it does not exist.
*
* @returns Last indentation level or null if it does not exist.
*/
fn get_last_indentation_level(text: &mut Parser) -> Option<usize> {
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
fn unquoted_string(text: &mut Parser) -> RuleResult {
    let key_acceptable_chars = Some(String::from(KEY_ACCEPTABLE_CHARS));
    let mut chars = vec![char(text, &key_acceptable_chars)?];

    loop {
        let matched_char = maybe_char(text, &key_acceptable_chars);
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

    Ok(GuraType::String(
        trimmed_str,
    ))
}

/**
* Parses a string checking if it is a number and get its correct value.
*
* @throws ParseError if the extracted string is not a valid number.
* @returns Returns an number or a float depending of type inference.
*/
fn number(text: &mut Parser) -> RuleResult {
    #[derive(Debug, PartialEq, Eq)]
    enum NumberType {
        Integer,
        Float,
    }

    let acceptable_number_chars: Option<String> = Some(String::from(
        BASIC_NUMBERS_CHARS.to_string() + &HEX_OCT_BIN + &INF_AND_NAN + "Ee+._-",
    ));

    let mut number_type = NumberType::Integer;

    let mut chars = vec![char(text, &acceptable_number_chars)?];

    loop {
        let matched_char = maybe_char(text, &acceptable_number_chars);
        match matched_char {
            Some(a_char) => {
                if String::from("Ee.").contains(a_char) {
                    number_type = NumberType::Float
                }

                chars.push(a_char)
            }
            None => break,
        };
    }

    // Replaces underscores as Rust does not support them in the same way Gura does
    let result = chars
        .iter()
        .cloned()
        .collect::<String>()
        .trim_end()
        .replace("_", "");

    // Checks hexadecimal, octal and binary format
    let prefix = result[0..2].to_string();
    let prefix = prefix.as_str();
    if vec!["0x", "0o", "0b"].contains(&prefix) {
        let base: u32;
        let without_prefix = result[2..].to_string();
        match prefix {
            "0x" => base = 16,
            "0o" => base = 8,
            _ => base = 2,
        };

        let int_value = u64::from_str_radix(&without_prefix, base).unwrap();
        return Ok(GuraType::Integer(
            int_value,
        ));
    }

    // Checks inf or NaN
    // Checks result len to prevent "attempt to subtract with overflow" error
    let last_three_chars = if result.len() >= 3 {
        result[result.len() - 3..].to_string()
    } else {
        String::from("")
    };

    let last_three_chars = last_three_chars.as_str();
    match last_three_chars {
        "inf" => {
            return Ok(GuraType::Float(
                if result.chars().next().unwrap() == '-' {
                    NEG_INFINITY
                } else {
                    INFINITY
                },
            ));
        }
        "nan" => {
            return Ok(GuraType::Float(NAN));
        }
        _ => {
            // It's a normal number
            if number_type == NumberType::Integer {
                if let Ok(value) = result.parse::<u64>() {
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
fn list(text: &mut Parser) -> RuleResult {
    let mut result: Vec<Box<GuraType>> = Vec::new();

    maybe_match(text, vec![Box::new(ws)]);
    // TODO: try char
    keyword(text, &vec![String::from("[")])?;
    loop {
        // Discards useless lines between elements of array
        match maybe_match(text, vec![Box::new(useless_line)]) {
            Some(_) => continue,
            _ => {
                let item: Box<GuraType>;
                match maybe_match(text, vec![Box::new(any_type)]) {
                    None => break,
                    Some(value) => item = Box::new(value),
                }

                result.push(item);

                maybe_match(text, vec![Box::new(ws)]);
                maybe_match(text, vec![Box::new(new_line)]);
                // TODO: try char()
                if maybe_keyword(text, &vec![String::from(",")]).is_none() {
                    break;
                }
            }
        }
    }

    maybe_match(text, vec![Box::new(ws)]);
    maybe_match(text, vec![Box::new(new_line)]);
    // TODO: try char()
    keyword(text, &vec![String::from("]")])?;
    Ok(GuraType::List(result))
}

/**
* Matches with a simple / multiline literal string.
*
* @returns Matched string.
*/
fn literal_string(text: &mut Parser) -> RuleResult {
    let quote = keyword(text, &vec![String::from("'''"), String::from("'")])?;

    let is_multiline = quote == "'''";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if is_multiline {
        maybe_char(text, &Some(String::from("\n")));
    }

    let mut chars: Vec<char> = Vec::new();

    loop {
        match maybe_keyword(text, &vec![String::from(quote.clone())]) {
            Some(_) => break,
            _ => {
                let matched_char = char(text, &None)?;
                chars.push(matched_char);
            }
        }
    }

    let final_string = chars.iter().cloned().collect::<String>();
    Ok(GuraType::String(
        final_string,
    ))
}

/**
 * Match any Gura expression.
 *
 * @throws DuplicatedKeyError if any of the defined key was declared more than once.
 * @returns Object with Gura string data.
 */
fn object(text: &mut Parser) -> RuleResult {
    let mut result: HashMap<String, Box<GuraType>> = HashMap::new();
    let mut indentation_level = 0;
    while text.pos < text.len {
        match maybe_match(
            text,
            vec![Box::new(variable), Box::new(pair), Box::new(useless_line)],
        ) {
            None => break,
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

        if maybe_keyword(text, &vec![String::from("]"), String::from(",")]).is_some() {
            // Breaks if it is the end of a list
            remove_last_indentation_level(text);
            text.pos -= 1;
            break;
        }
    }

    Ok(GuraType::ObjectWithWs(result, indentation_level))
}

/**
 * Matches with a key - value pair taking into consideration the indentation levels.
 *
 * @returns Matched key - value pair.null if the indentation level is lower than the last one(to indicate the ending of a parent object).
 */
fn pair(text: &mut Parser) -> RuleResult {
    let pos_before_pair = text.pos;

    // let currentIndentationLevel = maybe_match(text, vec![Box::new(wsWithIndentation)]);
    if let GuraType::Indentation(current_indentation_level) =
        matches(text, vec![Box::new(ws_with_indentation)])?
    {
        let matched_key = matches(text, vec![Box::new(key)])?;

        if let GuraType::String(key_value) = matched_key {
            maybe_match(text, vec![Box::new(ws)]);
            maybe_match(text, vec![Box::new(new_line)]);

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
                    remove_last_indentation_level(text);

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
                    )))
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
            maybe_match(text, vec![Box::new(new_line)]);

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
