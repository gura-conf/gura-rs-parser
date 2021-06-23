use std::{error::Error, fmt};

// TODO: Refactor using macros

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