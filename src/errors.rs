use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Raises when Gura syntax is invalid
    ParseError,
    /// Raises when a variable is not defined
    VariableNotDefinedError,
    /// Raises when indentation is invalid
    InvalidIndentationError,
    /// Raises when a variable is defined more than once
    DuplicatedVariableError,
    /// Raises when a key is defined more than once
    DuplicatedKeyError,
    /// Raises when an imported file was not found
    FileNotFoundError,
    /// Raises when a file is imported more than once
    DuplicatedImportError,
}

/// A Gura error with position, line and custom message
#[derive(Debug, PartialEq)]
pub struct GuraError {
    pub pos: usize,
    pub line: usize,
    pub msg: String,
    pub kind: Error, // TODO: rename to type
}

impl fmt::Display for GuraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} at line {} position {}",
            self.msg, self.line, self.pos
        )
    }
}

// ValueError (for internal usage)
#[derive(Debug)]
pub struct ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad character range")
    }
}
