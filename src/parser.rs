use itertools::Itertools;
use std::{collections::{HashMap, HashSet}, error::Error, fmt::{self}, ops::AddAssign, usize};

use crate::errors::InvalidIndentationError;

// TODO: refactor to anothe file the errors and types

type Rules = Vec<Box<dyn Fn() -> Result<GuraType, ParseError>>>;



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
enum PrimitiveType {

}

/* Data types to be returned by match expression methods */
#[derive(Debug)]
enum GuraType {
  USELESS_LINE,
  PAIR(String, Box<GuraType>),
  COMMENT,
  IMPORT(String),
  VARIABLE,
  EXPRESSION(Box<GuraType>),
  PRIMITIVE(PrimitiveType),
  LIST(Vec<Box<GuraType>>)
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

	/// Checks that the parser has reached the end of file, otherwise it will raise a ParseError
	/// :raise: ParseError if EOL has not been reached
	fn assert_end(&self) -> Result<(), ParseError> {
		if self.pos < self.len {
			Err(ParseError::new(
				self.pos + 1,
				self.line,
				format!("Expected end of string but got {}", self.text.chars().nth(self.pos + 1).unwrap())
			))
		} else {
			Ok(())
		}
	}

	/// Generates a list of char from a list of char which could container char ranges (i.e. a-z or 0-9)
	/// :param chars: List of chars to process
	/// :return: List of char with ranges processed
	fn split_char_ranges(&mut self, chars: &String) -> Result<String, ValueError> {
		if self.cache.contains_key(chars) {
			return Ok(self.cache.get(chars).unwrap().to_string());
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
	
		self.cache.insert(chars.to_string(), result.clone());
		Ok(result)
	}
	
	/// Matches a list of specific chars and returns the first that matched. If any matched, it will raise a ParseError
	/// :param chars: Chars to match. If it is None, it will return the next char in text
	/// :raise: ParseError if any of the specified char (i.e. if chars != None) matched
	/// :return: Matched char
	fn char(&mut self, chars: Option<String>) -> Result<String, ParseError> {
		if self.pos >= self.len {
			return Err(ParseError::new(
				self.pos + 1,
				self.line,
				format!("Expected {} but got end of string", match chars {
					None => String::from("character"),
					Some(chars) => format!("{}", chars)
				})
			));
		}
	
		let next_char = String::from(self.text.chars().nth(self.pos + 1).unwrap());
		match chars {
			None => {
				self.pos += 1;
				return Ok(next_char);
			},
			Some(chars_value) => {
				for char_range in self.split_char_ranges(&chars_value) {
					if char_range.len() == 1 {
						if next_char == char_range {
							self.pos += 1;
							return Ok(next_char);
						}
					} else {
						let mut char_range_chr = char_range.chars();
						let bottom = String::from(char_range_chr.nth(0).unwrap());
						let top = String::from(char_range_chr.nth(2).unwrap());
						if bottom <= next_char && next_char <= top {
							self.pos += 1;
							return Ok(next_char);
						}
					}
				}
				
				return Err(ParseError::new(
					self.pos + 1,
					self.line,
					format!("Expected {} but got {}", format!("[{}]", chars_value), next_char)
				))
			}
		}
	
	}

	/// Matches specific keywords
	/// :param keywords: Keywords to match
	/// :raise: ParseError if any of the specified keywords matched
	/// :return: The first matched keyword
	fn keyword(&mut self, keywords: Vec<String>) -> Result<String, ParseError> {
		if self.pos >= self.len {
			return Err(ParseError::new(
				self.pos + 1,
				self.line,
				format!("Expected {} but got end of string", keywords.iter().join(","))
			))
		}
	
		for keyword in &keywords {
			let low = self.pos + 1;
			let high = low + keyword.len();
	
			if self.text.get(low .. high).unwrap() == keyword {
				self.pos += keyword.len();
				return Ok(keyword.to_string());
			}
		}
	
		Err(ParseError::new(
			self.pos + 1,
			self.line,
			format!(
				"Expected {} but got {}",
				keywords.iter().join(", "),
				self.text.chars().nth(self.pos + 1).unwrap()
			)
		))
	}

	/// Matches specific rules which name must be implemented as a method in corresponding parser. A rule does not match
	/// if its method raises ParseError
	/// :param rules: Rules to match
	/// :raise: ParseError if any of the specified rules matched
	/// :return: The first matched rule method's result
	fn matches(&mut self, rules: Rules) -> Result<GuraType, ParseError> {
		let mut last_error_pos: usize = 0;
		let mut last_exception = None;
		// let last_error_rules = Vec::with_capacity(rules.len());
	
		for rule in rules {
			let initial_pos = self.pos;

			let result = rule();

			if result.is_ok() {
				return result;
			} else {
				let err = result.unwrap_err();
				self.pos = initial_pos;
	
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
		// 	let last_error_pos = (self.text.len() - 1).min(last_error_pos);
		// 	Err(ParseError::new(
		// 		last_error_pos,
		// 		self.line,
		// 		format!("Expected {} but got {}", last_error_rules.iter().join(","), self.text[last_error_pos])
		// 	))
		// }
	}

	/// Like char() but returns None instead of raising ParseError
	/// :param chars: Chars to match. If it is None, it will return the next char in text
	/// :return: Char if matched, None otherwise
	fn maybe_char(&mut self, chars: Option<String>) -> Option<String> {
		match self.char(chars) {
			Err(_) => None,
			result => result.ok()
		}
	}
	

	/// Like match() but returns None instead of raising ParseError
	/// :param rules: Rules to match
	/// :return: Rule result if matched, None otherwise
	fn maybe_match(&mut self, rules: Rules) -> Option<GuraType> {
		match self.matches(rules) {
			Err(_) => None,
			result => result.ok()
		}
	}
	

	/// Like keyword() but returns None instead of raising ParseError
	/// :param keywords: Keywords to match
	/// :return: Keyword if matched, None otherwise
	fn maybe_keyword(&mut self, keywords: Vec<String>) -> Option<String> {
		match self.keyword(keywords) {
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
	fn parse (&self, text: &String) -> HashMap<String, GuraType> {
		self.restartParams(text);
		let result = self.start();
		self.assertEnd();
		if !result.is_none() { result } else { HashMap::new() }
	}

	/**
	 * Sets the params to start parsing from a specific text.
	 *
	 * @param text - Text to set as the internal text to be parsed.
	 */
	fn restartParams (&mut self, text: &String) {
		self.text = text.to_string();
		self.pos = 0;
		self.line = 0;
		self.len = text.len() - 1;
	}

	/**
	* Matches with a new line.
	*/
	fn newLine (&mut self) {
		let newLineChars = Some(String::from("\x0c\x0b\r\n"));
		let res = self.char(newLineChars);
		if res.is_ok() {
			self.line += 1
		}
	}

	/**
	* Matches with a comment.
	*
	* @returns MatchResult indicating the presence of a comment.
	*/
  	fn comment (&mut self) -> GuraType {
		self.keyword(vec![String::from("#")]);
		while self.pos < self.len {
			let char = self.text.chars().nth(self.pos + 1).unwrap();
			self.pos += 1;
			if String::from("\x0c\x0b\r\n").contains(char) {
				self.line += 1;
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
  	fn wsWithIndentation (&self) -> Result<usize, InvalidIndentationError> {
		let mut currentIndentationLevel = 0;

		while self.pos < self.len {
			let blank = self.maybe_keyword(vec![
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
	fn ws (&self) {
		while self.maybe_keyword(vec![String::from(" "), String::from("\t")]).is_some() {
			continue
		}
	}

	/**
	* Consumes all the whitespaces and new lines.
	*/
	fn eatWsAndNewLines (&self) {
		let wsAndNewLinesChars = Some(String::from(" \x0c\x0b\r\n\t"));
		while self.maybe_char(wsAndNewLinesChars).is_some() {
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
		&self,
		originalText: &String,
		parentDirPath: String,
		importedFiles: HashSet<String>
	) -> (String, HashSet<String>) {
		self.restartParams(originalText);
		let importedFiles = self.computeImports(parentDirPath, importedFiles);
		(self.text, importedFiles)
	}

	/**
	* Matches import sentence.
	*
	* @returns MatchResult with file name of imported file.
	*/
  	fn guraImport (&self) -> Result<GuraType, ParseError> {
		self.keyword(vec![String::from("import")]);
		self.char(Some(String::from(" ")));
		let string_match = self.matches(vec![self.quotedStringWithVar])?;

		if let GuraType::IMPORT(fileToImport) = string_match {
			self.matches(vec![self.ws]);
			self.maybe_match(vec![self.newLine]);
			Ok(GuraType::IMPORT(fileToImport))
		} else {
			Err(ParseError::new(self.pos, self.line, String::from("Gura import invalid")))
		}
  }
}


