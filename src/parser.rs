use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fmt::{self},
    ops::AddAssign,
    usize, vec,
};

use crate::errors::{DuplicatedVariableError, InvalidIndentationError, VariableNotDefinedError};

const KEY_ACCEPTABLE_CHARS: Option<String> = Some(String::from("0-9A-Za-z_"));

// Special characters to be escaped
const ESCAPE_SEQUENCES: HashMap<char, char> = [
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
.collect();

// TODO: refactor to another file the errors and types
type RuleResult = Result<GuraType, ParseError>;
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

// TODO: implement
#[derive(Debug)]
enum PrimitiveTypeEnum {
    Bool(bool),
    String(String),
    Number(f64),
}

#[derive(Debug)]
enum VariableValueType {
    String(String),
    Number(f64),
}

type PrimitiveType = Option<PrimitiveTypeEnum>;

/* Data types to be returned by match expression methods */
// TODO: change to CamelCase
// TODO: differentiate between all the types and only the valid types for final result map
#[derive(Debug)]
enum GuraType {
    USELESS_LINE,
    PAIR(String, Box<GuraType>),
    COMMENT,
    IMPORT(String),
    VARIABLE,
    VariableValue(VariableValueType),
    EXPRESSION((Box<GuraType>, usize)),
    PRIMITIVE(PrimitiveType),
    LIST(Vec<Box<GuraType>>),
    WsOrNewLine,
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
    cache: HashMap<String, String>,
    variables: HashMap<String, VariableValueType>,
    indentationLevels: Vec<usize>,
    importedFiles: HashSet<String>,
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
            indentationLevels: Vec::new(),
            importedFiles: HashSet::new(),
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
    computeImports(text, None, HashSet::new());
    let result = matches(text, vec![Box::new(expression)])?;
    eatWsAndNewLines(text);
    Ok(result.0)
}

/**
 * Matches with any primitive or complex type.
 *
 * @returns The corresponding matched value.
 */

fn anyType(text: &mut Parser) -> RuleResult {
    let result = maybe_match(text, vec![Box::new(primitiveType)]);

    if let Some(result) = result {
        Ok(result)
    } else {
        matches(text, vec![Box::new(complexType)])
    }
}

/**
 * Matches with a primitive value: null, bool, strings(all of the four kind of string), number or variables values.
 *
 * @returns The corresponding matched value.
 */
fn primitiveType(text: &mut Parser) -> RuleResult {
    maybe_match(text, vec![Box::new(ws)]);
    matches(
        text,
        vec![
            Box::new(null),
            Box::new(boolean),
            Box::new(basicString),
            Box::new(literalString),
            Box::new(number),
            Box::new(variableValue),
        ],
    )
}

/**
* Matches with a useless line.A line is useless when it contains only whitespaces and / or a comment finishing in a new line.
*
* @throws ParseError if the line contains valid data.
* @returns MatchResult indicating the presence of a useless line.
*/
fn uselessLine(text: &mut Parser) -> RuleResult {
    matches(text, vec![Box::new(ws)])?;
    let comment = maybe_match(text, vec![Box::new(comment)]);
    let initialLine = text.line;
    maybe_match(text, vec![Box::new(new_Line)]);
    let is_new_line = (text.line - initialLine) == 1;

    if comment.is_none() && !is_new_line {
        return Err(ParseError::new(
            text.pos + 1,
            text.line,
            String::from("It is a valid line"),
        ));
    }

    Ok(GuraType::USELESS_LINE)
}

/**
* Matches with a list or another complex expression.
*
* @returns List or Dict, depending the correct matching.
*/
fn complexType(text: &mut Parser) -> RuleResult {
    matches(text, vec![Box::new(list), Box::new(expression)])
}

/**
* Consumes null keyword and return null.
*
* @returns Null.
*/
fn null(text: &mut Parser) -> RuleResult {
    keyword(text, vec![String::from("null")]);
    Ok(GuraType::PRIMITIVE(None))
}

/**
* Parses boolean values.
*
* @returns Matched boolean value.
*/
fn boolean(text: &mut Parser) -> RuleResult {
    let value = keyword(text, vec![String::from("true"), String::from("false")])? == "true";
    Ok(GuraType::PRIMITIVE(Some(PrimitiveTypeEnum::Bool(value))))
}

/**
 * Matches with a simple / multiline basic string.
 *
 * @returns Matched string.
 */
fn basicString(text: &mut Parser) -> RuleResult {
    let quote = keyword(text, vec![String::from("\"\"\""), String::from("\"")])?;

    let isMultiline = quote == "\"\"\"";

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if isMultiline {
        maybe_char(text, Some(String::from("\n")));
    }

    let chars: Vec<char> = Vec::new();

    loop {
        let closingQuote = maybe_keyword(text, vec![quote]);
        if closingQuote.is_some() {
            break;
        }

        let current_char = char(text, None)?;
        if current_char == '\\' {
            let escape = char(text, None)?;

            // Checks backslash followed by a newline to trim all whitespaces
            if isMultiline && escape == '\n' {
                eatWsAndNewLines(text)
            } else {
                // Supports Unicode of 16 and 32 bits representation
                if escape == 'u' || escape == 'U' {
                    let numCharsCodePoint = if escape == 'u' { 4 } else { 8 };
                    let mut codePoint: Vec<char> = Vec::with_capacity(numCharsCodePoint);
                    for _ in 0..numCharsCodePoint {
                        let code_point_char = char(text, Some(String::from("0-9a-fA-F")))?;
                        codePoint.push(code_point_char);
                    }
                    let codePointStr = codePoint.iter().cloned().collect::<String>();
                    let hexValue = u32::from_str_radix(codePointStr.as_str(), 16);
                    // TODO: fix to use ? instead of this if
                    match hexValue {
                        Err(_) => {
                            return Err(ParseError::new(
                                text.pos,
                                text.line,
                                String::from("Bad hex value"),
                            ));
                        }
                        Ok(hexValue) => {
                            let charValue = char::from_u32(hexValue).unwrap(); // Converts from UNICODE to string
                            chars.push(charValue)
                        }
                    };
                } else {
                    // Gets escaped char
                    let escaped_char = match ESCAPE_SEQUENCES.get(&escape) {
                        Some(good_escape_char) => good_escape_char.clone(),
                        None => current_char,
                    };
                    chars.push(escaped_char);
                }
            }
        } else {
            // Computes variables values in string
            if current_char == '$' {
                let varName = getVarName(text);
                let var_value = getVariableValue(text, varName)?;
                // TODO: implement
                chars.push();
            } else {
                chars.push(current_char);
            }
        }
    }

    let final_string = chars.iter().cloned().collect::<String>();
    Ok(GuraType::PRIMITIVE(Some(PrimitiveTypeEnum::String(
        final_string,
    ))))
}

/**
 * Gets a variable name char by char.
 *
 * @returns Variable name.
 */
fn getVarName(text: &mut Parser) -> String {
    let mut varName = String::new();
    let varNameChar = maybe_char(text, KEY_ACCEPTABLE_CHARS);
    while varNameChar.is_some() {
        varName.push(varNameChar.unwrap());
        varNameChar = maybe_char(text, KEY_ACCEPTABLE_CHARS)
    }

    varName
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
fn computeImports(
    text: &mut Parser,
    parentDirPath: Option<String>,
    importedFiles: HashSet<String>,
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
    //   this.restartParams(finalContent + this.text.substring(this.pos + 1))
    // }

    // return importedFiles
}

/**
 * Matches with an already defined variable and gets its value.
 *
 * @returns Variable value.
 */
fn variableValue(text: &mut Parser) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    keyword(text, vec![String::from("$")])?;
    let key = matches(text, vec![Box::new(unquotedString)]);

    Ok(GuraType::VARIABLE(getVariableValue(text, key)))
}

/// Checks that the parser has reached the end of file, otherwise it will raise a ParseError
/// :raise: ParseError if EOL has not been reached
fn assert_end(text: &Parser) -> Result<(), ParseError> {
    if text.pos < text.len {
        Err(ParseError::new(
            text.pos + 1,
            text.line,
            format!(
                "Expected end of string but got {}",
                text.text.chars().nth(text.pos + 1).unwrap()
            ),
        ))
    } else {
        Ok(())
    }
}

/// Generates a list of char from a list of char which could container char ranges (i.e. a-z or 0-9)
/// :param chars: List of chars to process
/// :return: List of char with ranges processed
fn split_char_ranges(text: &mut Parser, chars: &String) -> Result<String, ValueError> {
    if text.cache.contains_key(chars) {
        return Ok(text.cache.get(chars).unwrap().to_string());
    }

    let length = chars.len();
    let mut chars_vec = chars.chars();
    let mut result = String::with_capacity(length);
    let mut index = 0;

    while index < length {
        if index + 2 < length && chars_vec.nth(index + 1).unwrap() == '-' {
            if chars_vec.nth(index).unwrap() >= chars_vec.nth(index + 2).unwrap() {
                return Err(ValueError {});
            }

            result.add_assign(chars.get(index..index + 3).unwrap());
            index += 3;
        } else {
            result.push(chars_vec.nth(index).unwrap());
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
fn char(text: &mut Parser, chars: Option<String>) -> Result<char, ParseError> {
    if text.pos >= text.len {
        return Err(ParseError::new(
            text.pos + 1,
            text.line,
            format!(
                "Expected {} but got end of string",
                match chars {
                    None => String::from("character"),
                    Some(chars) => format!("{}", chars),
                }
            ),
        ));
    }

    let next_char = text.text.chars().nth(text.pos + 1).unwrap();
    match chars {
        None => {
            text.pos += 1;
            return Ok(next_char);
        }
        Some(chars_value) => {
            for char_range in split_char_ranges(text, &chars_value) {
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

            return Err(ParseError::new(
                text.pos + 1,
                text.line,
                format!(
                    "Expected {} but got {}",
                    format!("[{}]", chars_value),
                    next_char
                ),
            ));
        }
    }
}

/// Matches specific keywords
/// :param keywords: Keywords to match
/// :raise: ParseError if any of the specified keywords matched
/// :return: The first matched keyword
// TODO: Change keywords to Vec<&str>
fn keyword(text: &mut Parser, keywords: Vec<String>) -> Result<String, ParseError> {
    if text.pos >= text.len {
        return Err(ParseError::new(
            text.pos + 1,
            text.line,
            format!(
                "Expected {} but got end of string",
                keywords.iter().join(",")
            ),
        ));
    }

    for keyword in &keywords {
        let low = text.pos + 1;
        let high = low + keyword.len();

        if text.text.get(low..high).unwrap() == keyword {
            text.pos += keyword.len();
            return Ok(keyword.to_string());
        }
    }

    Err(ParseError::new(
        text.pos + 1,
        text.line,
        format!(
            "Expected {} but got {}",
            keywords.iter().join(", "),
            text.text.chars().nth(text.pos + 1).unwrap()
        ),
    ))
}

/// Matches specific rules which name must be implemented as a method in corresponding parser. A rule does not match
/// if its method raises ParseError
/// :param rules: Rules to match
/// :raise: ParseError if any of the specified rules matched
/// :return: The first matched rule method's result
fn matches(text: &mut Parser, rules: Rules) -> RuleResult {
    let mut last_error_pos: usize = 0;
    let mut last_exception = None;
    // let last_error_rules = Vec::with_capacity(rules.len());

    for rule in rules {
        let initial_pos = text.pos;

        let result = rule(text);

        if result.is_ok() {
            return result;
        } else {
            let err = result.unwrap_err();
            text.pos = initial_pos;

            if err.pos > last_error_pos {
                last_exception = Some(err.clone());
                last_error_pos = err.pos;
                // last_error_rules.clear();
                // last_error_rules.append(rule)
            }
            // else {
            // 	if err.pos == last_error_pos {
            // 		last_error_rules.append(rule)
            // 	}
            // }
        }
    }

    // Unwrap is safe as if this line is reached no rule matched
    Err(last_exception.unwrap())
    // if last_error_rules.len() == 1 {
    // 	Err(last_exception)
    // } else {
    // 	let last_error_pos = (text.text.len() - 1).min(last_error_pos);
    // 	Err(ParseError::new(
    // 		last_error_pos,
    // 		text.line,
    // 		format!("Expected {} but got {}", last_error_rules.iter().join(","), text.text[last_error_pos])
    // 	))
    // }
}

/// Like char() but returns None instead of raising ParseError
/// :param chars: Chars to match. If it is None, it will return the next char in text
/// :return: Char if matched, None otherwise
fn maybe_char(text: &mut Parser, chars: Option<String>) -> Option<char> {
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
fn maybe_keyword(text: &mut Parser, keywords: Vec<String>) -> Option<String> {
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
* @returns Object with all the parsed values.
*/
fn parse(text_parser: &mut Parser, text: &String) -> Option<GuraType> {
    text_parser.restart_params(text);
    let result = start(text_parser);
    assert_end(text_parser);
    match result {
        Ok(content) => Some(content),
        _ => None,
    }
}

/**
* Matches with a new line.
*/
fn new_Line(text: &mut Parser) -> RuleResult {
    let newLineChars = Some(String::from("\x0c\x0b\r\n"));
    let res = char(text, newLineChars);
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
    keyword(text, vec![String::from("#")]);
    while text.pos < text.len {
        let char = text.text.chars().nth(text.pos + 1).unwrap();
        text.pos += 1;
        if String::from("\x0c\x0b\r\n").contains(char) {
            text.line += 1;
            break;
        }
    }

    Ok(GuraType::COMMENT)
}

/**
* Matches with white spaces taking into consideration indentation levels.
*
* @returns Indentation level.
*/
fn wsWithIndentation(text: &mut Parser) -> Result<usize, InvalidIndentationError> {
    let mut currentIndentationLevel = 0;

    while text.pos < text.len {
        let blank = maybe_keyword(text, vec![String::from(" "), String::from("\t")]);

        if blank.is_none() {
            // If it is not a blank or new line, returns from the method
            break;
        }

        // Tabs are not allowed
        if blank.unwrap() == "\t" {
            return Err(InvalidIndentationError::new(String::from(
                "Tabs are not allowed to define indentation blocks",
            )));
        }

        currentIndentationLevel += 1
    }

    Ok(currentIndentationLevel)
}

/**
* Matches white spaces (blanks and tabs).
*/
fn ws(text: &mut Parser) -> RuleResult {
    while maybe_keyword(text, vec![String::from(" "), String::from("\t")]).is_some() {
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
fn quotedStringWithVar(text: &mut Parser) -> RuleResult {
    // TODO: consider using char(text, vec![String::from("\"")])
    let quote = keyword(text, vec![String::from("\"")])?
        .chars()
        .nth(0)
        .unwrap();
    let mut chars: Vec<char> = Vec::new();

    loop {
        let current_char = char(text, None)?;

        if current_char == quote {
            break;
        }

        // Computes variables values in string
        if current_char == '$' {
            let varName = getVarName(text);
            match getVariableValue(text, varName) {
                Ok(some_var) => {
                    let var_chars: Vec<char> = match some_var {
                        VariableValueType::String(var_value_str) => var_value_str.chars().collect(),
                        VariableValueType::Number(var_value_number) => {
                            var_value_number.to_string().chars().collect()
                        }
                        _ => {
                            return Err(ParseError::new(
                                text.pos,
                                text.line,
                                String::from(format!("Variable {} has an invalid type", varName)),
                            ));
                        }
                    };
                    chars.append(&mut var_chars);
                }
                // TODO: change this by VariableNotDefinedError
                _ => {
                    return Err(ParseError::new(
                        text.pos,
                        text.line,
                        String::from("VariableNotDefinedError"),
                    ));
                }
            }
        } else {
            chars.push(current_char);
        }
    }

    let final_string = chars.iter().cloned().collect::<String>();
    Ok(GuraType::PRIMITIVE(Some(PrimitiveTypeEnum::String(
        final_string,
    ))))
}

/**
* Consumes all the whitespaces and new lines.
*/
fn eatWsAndNewLines(text: &mut Parser) {
    let wsAndNewLinesChars = Some(String::from(" \x0c\x0b\r\n\t"));
    while maybe_char(text, wsAndNewLinesChars).is_some() {
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
fn getVariableValue(
    text: &mut Parser,
    key: String,
) -> Result<VariableValueType, VariableNotDefinedError> {
    match text.variables.get(&key) {
        Some(&value) => {
            match value {
                VariableValueType::Number(number_value) => {
                    return Ok(VariableValueType::Number(number_value))
                }
                VariableValueType::String(str_value) => {
                    return Ok(VariableValueType::String(str_value))
                }
                // TODO: change to ParseError
                // _ => return Err(ParseError::new(text.pos, text.line, String::from("Invalid value")))
                _ => return Err(VariableNotDefinedError::new(String::from("Invalid value"))),
            }
        }
        _ => match env::var(key) {
            Ok(value) => Ok(VariableValueType::String(value)),
            Err(e) => Err(VariableNotDefinedError::new(format!(
                "Variable '{}' is not defined in Gura nor as environment variable",
                key
            ))),
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
fn getTextWithImports(
    text: &mut Parser,
    originalText: &String,
    parentDirPath: String,
    importedFiles: HashSet<String>,
) -> Result<(String, HashSet<String>), ParseError> {
    text.restart_params(originalText);
    let importedFiles = computeImports(text, Some(parentDirPath), importedFiles)?;
    Ok((text.text, importedFiles))
}

/**
* Matches import sentence.
*
* @returns MatchResult with file name of imported file.
*/
fn guraImport(text: &mut Parser) -> RuleResult {
    keyword(text, vec![String::from("import")]);
    char(text, Some(String::from(" ")));
    let string_match = matches(text, vec![Box::new(quotedStringWithVar)])?;

    if let GuraType::IMPORT(fileToImport) = string_match {
        matches(text, vec![Box::new(ws)]);
        maybe_match(text, vec![Box::new(new_Line)]);
        Ok(GuraType::IMPORT(fileToImport))
    } else {
        Err(ParseError::new(
            text.pos,
            text.line,
            String::from("Gura import invalid"),
        ))
    }
}

/**
 * Matches with a variable definition.
 *
 * @throws DuplicatedVariableError if the current variable has been already defined.
 * @returns Match result indicating that a variable has been added.
 */
fn variable(text: &mut Parser) -> RuleResult {
    keyword(text, vec![String::from("$")]);
    let matched_key = matches(text, vec![Box::new(key)])?;

    if let GuraType::PRIMITIVE(Some(PrimitiveTypeEnum::String(key_value))) = matched_key {
        maybe_match(text, vec![Box::new(ws)]);

        let matchResult = matches(
            text,
            vec![
                Box::new(basicString),
                Box::new(literalString),
                Box::new(number),
                Box::new(variableValue),
            ],
        )?;

        if let GuraType::VariableValue(var_value) = matchResult {
            if text.variables.contains_key(&key_value) {
                // TODO: fix to return DuplicatedVariableError
                // return Err(DuplicatedVariableError::new(format!("Variable '{}' has been already declared", key)));
                return Err(ParseError::new(
                    0,
                    0,
                    format!("Variable '{}' has been already declared", key_value),
                ));
            }

            // Store as variable
            text.variables.insert(key_value, var_value);
            Ok(GuraType::VARIABLE)
        } else {
            Err(ParseError::new(
                0,
                0,
                String::from("Invalid variable value"),
            ))
        }
    } else {
        Err(ParseError::new(0, 0, String::from("Key not found")))
    }
}
