use std::{error::Error, fmt};

// TODO: Refactor using macros

// ParseError
#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    pub pos: usize,
    line: usize,
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


#[derive(Debug, Clone)]
pub struct VariableNotDefinedError {
	msg: String,
}

impl VariableNotDefinedError {
	pub fn new(msg: String) -> Self {
		VariableNotDefinedError {
			msg,
		}
	}
}

impl fmt::Display for VariableNotDefinedError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl Error for VariableNotDefinedError {}

#[derive(Debug, Clone)]
pub struct InvalidIndentationError {
	msg: String,
}

impl InvalidIndentationError {
	pub fn new(msg: String) -> Self {
		InvalidIndentationError {
			msg,
		}
	}
}

impl fmt::Display for InvalidIndentationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl Error for InvalidIndentationError {}

#[derive(Debug, Clone)]
pub struct DuplicatedVariableError {
	msg: String,
}

impl DuplicatedVariableError {
	pub fn new(msg: String) -> Self {
		DuplicatedVariableError {
			msg,
		}
	}
}

impl fmt::Display for DuplicatedVariableError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl Error for DuplicatedVariableError {}

#[derive(Debug, Clone)]
pub struct DuplicatedKeyError {
	msg: String,
}

impl DuplicatedKeyError {
	pub fn new(msg: String) -> Self {
		DuplicatedKeyError {
			msg,
		}
	}
}

impl fmt::Display for DuplicatedKeyError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl Error for DuplicatedKeyError {}


// ValueError
#[derive(Debug)]
pub struct ValueError {}

impl Error for ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad character range")
    }
}