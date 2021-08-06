use std::{error::Error, fmt};

// ParseError
#[derive(Debug, Clone)]
pub struct ParseError {
    pub pos: usize,
    line: usize,
    message: String,
}

impl ParseError {
    pub fn new(pos: usize, line: usize, message: String) -> Self {
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

// ValueError
#[derive(Debug)]
pub struct ValueError {}

impl Error for ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad character range")
    }
}

/// Defines Gura error with Display method
macro_rules! gura_error {
    ($error_name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $error_name {
            msg: String,
        }

        impl $error_name {
            pub fn new(msg: String) -> Self {
                $error_name { msg }
            }
        }

        impl fmt::Display for $error_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl Error for $error_name {}
    };
}

// Define extra common errors
gura_error!(VariableNotDefinedError);

gura_error!(InvalidIndentationError);

gura_error!(DuplicatedVariableError);

gura_error!(DuplicatedKeyError);

gura_error!(FileNotFoundError);

gura_error!(DuplicatedImportError);
