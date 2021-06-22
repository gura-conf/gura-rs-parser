use itertools::Itertools;
use std::{collections::{HashMap, HashSet}, error::Error, fmt::{self}, ops::AddAssign, usize};

use crate::errors::InvalidIndentationError;

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

}

type PrimitiveType = Option<PrimitiveTypeEnum>;

/* Data types to be returned by match expression methods */
// TODO: change to CamelCase
#[derive(Debug)]
enum GuraType {
  USELESS_LINE,
  PAIR(String, Box<GuraType>),
  COMMENT,
  IMPORT(String),
  VARIABLE,
  EXPRESSION(Box<GuraType>),
  PRIMITIVE(PrimitiveType),
  LIST(Vec<Box<GuraType>>),
  WsOrNewLine
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
		ParseError {
			pos,
			line,
			message,
		}
	}
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at line {} position {}", self.message, self.line, self.pos)
    }
}


// Base parser
struct Parser {
    text: String,
    pos: usize,
    line: usize,
    len: usize,
	cache: HashMap<String, String>,
	variables: HashMap<String, GuraType>,
	indentationLevels: Vec<usize>,
	importedFiles: HashSet<String>
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
			importedFiles: HashSet::new()
		}
	}

	/**
	* Sets the params to start parsing from a specific text.
	*
	* @param text - Text to set as the internal text to be parsed.
	*/
	fn restart_params (&mut self, text: &String) {
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
fn start (text: &mut Parser) -> RuleResult {
	computeImports(text, None, HashSet::new());
	let result = matches(text, vec![
		Box::new(expression)
	])?;
	eatWsAndNewLines(text);
	Ok(result.0)
}

/**
   * Matches with any primitive or complex type.
   *
   * @returns The corresponding matched value.
   */
   
fn anyType (text: &mut Parser) -> RuleResult  {
    let result = maybe_match(text, vec![
		Box::new(primitiveType)
	]);

    if let Some(result) = result {
      Ok(result)
    } else {
		matches(text, vec![
			Box::new(complexType)
		])
	}
}

/**
   * Matches with a primitive value: null, bool, strings(all of the four kind of string), number or variables values.
   *
   * @returns The corresponding matched value.
   */
fn primitiveType (text: &mut Parser) -> RuleResult {
    maybe_match(text, vec![
		Box::new(ws)
	]);
    matches(text, vec![
		Box::new(null), 
		Box::new(boolean), 
		Box::new(basicString), 
		Box::new(literalString), 
		Box::new(number), 
		Box::new(variableValue)
	])
}

/**
* Consumes null keyword and return null.
*
* @returns Null.
*/
fn null (text: &mut Parser) -> RuleResult {
    keyword(text, vec![String::from("null")]);
    Ok(GuraType::PRIMITIVE(None))
  }

/**
* Parses boolean values.
*
* @returns Matched boolean value.
*/
fn boolean (text: &mut Parser) -> RuleResult {
    let value = keyword(text, vec![String::from("true"), String::from("false")]) == "true";
    Ok(GuraType::PRIMITIVE(value))
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
fn computeImports (text: &mut Parser, parentDirPath: Option<String>, importedFiles: HashSet<String>)
	-> Result<HashSet<String>, ParseError> {
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

/// Checks that the parser has reached the end of file, otherwise it will raise a ParseError
/// :raise: ParseError if EOL has not been reached
fn assert_end(text: &Parser) -> Result<(), ParseError> {
	if text.pos < text.len {
		Err(ParseError::new(
			text.pos + 1,
			text.line,
			format!("Expected end of string but got {}", text.text.chars().nth(text.pos + 1).unwrap())
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
				return Err(ValueError{});
			}
			
			result.add_assign(chars.get(index .. index + 3).unwrap());
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
fn char(text: &mut Parser, chars: Option<String>) -> Result<String, ParseError> {
	if text.pos >= text.len {
		return Err(ParseError::new(
			text.pos + 1,
			text.line,
			format!("Expected {} but got end of string", match chars {
				None => String::from("character"),
				Some(chars) => format!("{}", chars)
			})
		));
	}

	let next_char = String::from(text.text.chars().nth(text.pos + 1).unwrap());
	match chars {
		None => {
			text.pos += 1;
			return Ok(next_char);
		},
		Some(chars_value) => {
			for char_range in split_char_ranges(text, &chars_value) {
				if char_range.len() == 1 {
					if next_char == char_range {
						text.pos += 1;
						return Ok(next_char);
					}
				} else {
					let mut char_range_chr = char_range.chars();
					let bottom = String::from(char_range_chr.nth(0).unwrap());
					let top = String::from(char_range_chr.nth(2).unwrap());
					if bottom <= next_char && next_char <= top {
						text.pos += 1;
						return Ok(next_char);
					}
				}
			}
			
			return Err(ParseError::new(
				text.pos + 1,
				text.line,
				format!("Expected {} but got {}", format!("[{}]", chars_value), next_char)
			))
		}
	}

}

/// Matches specific keywords
/// :param keywords: Keywords to match
/// :raise: ParseError if any of the specified keywords matched
/// :return: The first matched keyword
fn keyword(text: &mut Parser, keywords: Vec<String>) -> Result<String, ParseError> {
	if text.pos >= text.len {
		return Err(ParseError::new(
			text.pos + 1,
			text.line,
			format!("Expected {} but got end of string", keywords.iter().join(","))
		))
	}

	for keyword in &keywords {
		let low = text.pos + 1;
		let high = low + keyword.len();

		if text.text.get(low .. high).unwrap() == keyword {
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
		)
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
fn maybe_char(text: &mut Parser, chars: Option<String>) -> Option<String> {
	match char(text, chars) {
		Err(_) => None,
		result => result.ok()
	}
}


/// Like match() but returns None instead of raising ParseError
/// :param rules: Rules to match
/// :return: Rule result if matched, None otherwise
fn maybe_match(text: &mut Parser, rules: Rules) -> Option<GuraType> {
	match matches(text, rules) {
		Err(_) => None,
		result => result.ok()
	}
}


/// Like keyword() but returns None instead of raising ParseError
/// :param keywords: Keywords to match
/// :return: Keyword if matched, None otherwise
/// TODO: change to Vec<str>!!
fn maybe_keyword(text: &mut Parser, keywords: Vec<String>) -> Option<String> {
	match keyword(text, keywords) {
		Err(_) => None,
		result => result.ok()
	}
}

  /**
* Parses a text in Gura format.
*
* @param text - Text to be parsed.
* @throws ParseError if the syntax of text is invalid.
* @returns Object with all the parsed values.
*/
fn parse (text_parser: &Parser, text: &String) -> HashMap<String, GuraType> {
	text_parser.restart_params(text);
	let result = start(text_parser);
	assert_end(text_parser);
	if !result.is_none() { result } else { HashMap::new() }
}

/**
* Matches with a new line.
*/
fn new_Line (text: &mut Parser) -> RuleResult {
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
  fn comment (text: &mut Parser) -> GuraType {
	keyword(text, vec![String::from("#")]);
	while text.pos < text.len {
		let char = text.text.chars().nth(text.pos + 1).unwrap();
		text.pos += 1;
		if String::from("\x0c\x0b\r\n").contains(char) {
			text.line += 1;
			break;
		}
	}

	GuraType::COMMENT
  }

/**
* Matches with white spaces taking into consideration indentation levels.
*
* @returns Indentation level.
*/
  fn wsWithIndentation (text: &mut Parser) -> Result<usize, InvalidIndentationError> {
	let mut currentIndentationLevel = 0;

	while text.pos < text.len {
		let blank = maybe_keyword(text, vec![
			String::from(" "),
			String::from("\t")
		]);

		if blank.is_none() {
			// If it is not a blank or new line, returns from the method
			break
		}

		// Tabs are not allowed
		if blank.unwrap() == "\t"{
			return Err(InvalidIndentationError::new(
				String::from("Tabs are not allowed to define indentation blocks")
			));
		}

		currentIndentationLevel += 1
	}

	Ok(currentIndentationLevel)
  }

/**
* Matches white spaces (blanks and tabs).
*/
fn ws (text: &mut Parser) -> RuleResult {
	while maybe_keyword(text, vec![String::from(" "), String::from("\t")]).is_some() {
		continue;
	}

	Ok(GuraType::WsOrNewLine)
}

/**
* Consumes all the whitespaces and new lines.
*/
fn eatWsAndNewLines (text: &mut Parser) {
	let wsAndNewLinesChars = Some(String::from(" \x0c\x0b\r\n\t"));
	while maybe_char(text, wsAndNewLinesChars).is_some() {
		continue
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
  fn getTextWithImports (
	text: &Parser,
	originalText: &String,
	parentDirPath: String,
	importedFiles: HashSet<String>
) -> (String, HashSet<String>) {
	text.restart_params(originalText);
	let importedFiles = computeImports(parentDirPath, importedFiles);
	(text.text, importedFiles)
}

/**
* Matches import sentence.
*
* @returns MatchResult with file name of imported file.
*/
  fn guraImport (text: &mut Parser) -> RuleResult {
	keyword(text, vec![String::from("import")]);
	char(text, Some(String::from(" ")));
	let string_match = matches(vec![
		Box::new(quotedStringWithVar)
	])?;

	if let GuraType::IMPORT(fileToImport) = string_match {
		matches(text, vec![
			Box::new(ws)
		]);
		maybe_match(text, vec![
			Box::new(new_Line)
		]);
		Ok(GuraType::IMPORT(fileToImport))
	} else {
		Err(ParseError::new(text.pos, text.line, String::from("Gura import invalid")))
	}
}


