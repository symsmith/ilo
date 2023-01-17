use std::fmt::Display;

pub struct ErrorDetails {
	error_type: ErrorType,
	message: String,
	line: i64,
	column: i64,
}

impl ErrorDetails {
	pub fn new(error_type: ErrorType, message: String, line: i64, column: i64) -> Self {
		Self {
			error_type,
			message,
			line,
			column,
		}
	}
}

pub enum ErrorType {
	LexicalError,
	ParsingError,
	RuntimeError,
	TypeError,
}

impl Display for ErrorType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} error",
			match self {
				Self::LexicalError => "Lexical",
				Self::ParsingError => "Syntax",
				Self::RuntimeError => "Runtime",
				Self::TypeError => "Type",
			},
		)
	}
}

pub fn report_error(error_details: ErrorDetails) {
	display_error(error_details);
}

fn display_error(error_details: ErrorDetails) {
	println!(
		"{} at line {}, column {}: {}.",
		error_details.error_type, error_details.line, error_details.column, error_details.message
	);
}
