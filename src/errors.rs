pub enum ErrorDetails {
	LexicalError {
		message: String,
		line: i64,
		column: i64,
	},
	ParsingError {
		message: String,
		line: i64,
		column: i64,
	},
	RuntimeError {
		message: String,
		line: i64,
		column: i64,
	},
}

pub fn report_error(error_details: ErrorDetails) {
	display_error(error_details);
}

fn display_error(error_details: ErrorDetails) {
	match error_details {
		ErrorDetails::LexicalError {
			message,
			line,
			column,
		} => println!("Lexical error at line {line}, column {column}: {message}.",),
		ErrorDetails::ParsingError {
			message,
			line,
			column,
		} => {
			println!("Syntax error at line {line}, column {column}: {message}.",)
		}
		ErrorDetails::RuntimeError {
			message,
			line,
			column,
		} => {
			println!("Runtime error at line {line}, column {column}: {message}.",)
		}
	}
}
