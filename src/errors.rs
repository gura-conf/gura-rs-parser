#[derive(Debug, Clone)]
pub struct VariableNotDefinedError {
	var_name: String,
}

impl VariableNotDefinedError {
	pub fn new(var_name: String) -> Self {
		VariableNotDefinedError {
			var_name,
		}
	}
}

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