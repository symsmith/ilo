use std::fmt::Debug;
use substring::Substring;

use error_manager::{report_error, ErrorDetails, ErrorType};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
	// Single character tokens
	LeftBrace,     // }
	RightBrace,    // }
	LeftBracket,   // [
	RightBracket,  // ]
	Comma,         // ,
	Colon,         // :
	Interrogation, // ?
	LeftParen,     // (
	RightParen,    // )

	// 1-2-3 character tokens
	Arrow,        // ->
	Bang,         // !
	BangEqual,    // !=
	Caret,        // ^
	CaretEqual,   // ^=
	Dot,          // .
	DotDotDot,    // ...
	Equal,        // =
	EqualEqual,   // ==
	Greater,      // >
	GreaterEqual, // >=
	Less,         // <
	LessEqual,    // <=
	Minus,        // -
	MinusEqual,   // -=
	MinusMinus,   // --
	Percent,      // %
	PercentEqual, // %=
	Plus,         // +
	PlusEqual,    // +=
	PlusPlus,     // ++
	Slash,        // /
	SlashEqual,   // /=
	Star,         // *
	StarEqual,    // *=

	// Literals
	Identifier,
	NumberLiteral(f64),
	StringLiteral(String),

	// Reserved keywords
	And,      // and
	Ask,      // ask
	Boolean,  // boolean
	Break,    // break
	Cmd,      // cmd
	Continue, // continue
	Default,  // default
	Delete,   // delete
	Else,     // else
	Empty,    // empty
	False,    // false
	For,      // for
	Function, // function
	If,       // if
	In,       // in
	Keys,     // keys
	Match,    // match
	Number,   // number
	Or,       // or
	Out,      // out
	Return,   // return
	Size,     // size
	String,   // string
	True,     // true
	While,    // while

	EOL, // End of line (\n)
	EOF, // End of file
}

#[derive(Debug, Clone)]
pub struct Token {
	token_type: TokenType,
	/// Textual representation of the token, as is in the source code
	lexeme: String,
	/// Line of the start of the token
	line: i64,
	/// Column of the start of the token
	column: i64,
}

impl Token {
	pub fn token_type(&self) -> TokenType {
		self.token_type.clone()
	}

	pub fn lexeme(&self) -> &str {
		self.lexeme.as_ref()
	}

	pub fn line(&self) -> i64 {
		self.line
	}

	pub fn column(&self) -> i64 {
		self.column
	}
}

pub struct Lexer {
	source: String,
	tokens: Vec<Token>,
	/// Offset of the start of the current lexeme
	start: i64,
	/// Offset of the current scanned character
	current: i64,
	/// Line of the start of the current lexeme
	line: i64,
	/// Column of the start of the current lexeme
	column: i64,
}

impl Lexer {
	pub fn new(source: String) -> Self {
		Self {
			source,
			tokens: vec![],
			start: 0,
			current: 0,
			line: 1,
			column: 1,
		}
	}

	fn report_lexical_error(&self, message: String, line: i64, column: i64) {
		report_error(ErrorDetails::new(
			ErrorType::LexicalError,
			message,
			line,
			column,
		));
	}

	pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ()> {
		let mut has_error = false;
		while !self.is_at_end() {
			self.start = self.current;
			if let Err(()) = self.scan_token() {
				has_error = true
			}
		}

		self.tokens.push(Token {
			token_type: TokenType::EOF,
			lexeme: String::new(),
			line: self.line,
			column: self.column,
		});

		if has_error {
			Err(())
		} else {
			Ok(self.tokens.clone())
		}
	}

	fn is_at_end(&self) -> bool {
		self.current >= self.source.chars().count() as i64
	}

	fn scan_token(&mut self) -> Result<(), ()> {
		let mut has_error = false;
		match self.advance() {
			' ' | '\r' => self.column += 1,
			'\t' => self.column += 2,
			'\n' => self.add_token(TokenType::EOL),
			'{' => self.add_token(TokenType::LeftBrace),
			'}' => self.add_token(TokenType::RightBrace),
			'[' => self.add_token(TokenType::LeftBracket),
			']' => self.add_token(TokenType::RightBracket),
			',' => self.add_token(TokenType::Comma),
			':' => self.add_token(TokenType::Colon),
			'?' => self.add_token(TokenType::Interrogation),
			'(' => self.add_token(TokenType::LeftParen),
			')' => self.add_token(TokenType::RightParen),
			'-' => {
				if self.match_char('>') {
					self.add_token(TokenType::Arrow)
				} else if self.match_char('=') {
					self.add_token(TokenType::MinusEqual)
				} else if self.match_char('-') {
					self.add_token(TokenType::MinusMinus)
				} else {
					self.add_token(TokenType::Minus)
				}
			}
			'!' => {
				if self.match_char('=') {
					self.add_token(TokenType::BangEqual)
				} else {
					self.add_token(TokenType::Bang)
				}
			}
			'.' => {
				if self.match_char('.') && self.match_char('.') {
					self.add_token(TokenType::DotDotDot)
				} else {
					self.add_token(TokenType::Dot)
				}
			}
			'=' => {
				if self.match_char('=') {
					self.add_token(TokenType::EqualEqual)
				} else {
					self.add_token(TokenType::Equal)
				}
			}
			'>' => {
				if self.match_char('=') {
					self.add_token(TokenType::GreaterEqual)
				} else {
					self.add_token(TokenType::Greater)
				}
			}
			'<' => {
				if self.match_char('=') {
					self.add_token(TokenType::LessEqual)
				} else {
					self.add_token(TokenType::Less)
				}
			}
			'+' => {
				if self.match_char('=') {
					self.add_token(TokenType::PlusEqual)
				} else if self.match_char('+') {
					self.add_token(TokenType::PlusPlus)
				} else {
					self.add_token(TokenType::Plus)
				}
			}
			'/' => {
				if self.match_char('/') {
					// single line comment
					self.column += 2;
					while self.peek() != '\n' && !self.is_at_end() {
						self.advance();
						self.column += 1;
					}
				} else if self.match_char('*') {
					/* multiline comment */
					self.column += 2;
					while !(self.peek() == '/' && self.previous() == '*') && !self.is_at_end() {
						if self.advance() == '\n' {
							self.line += 1;
							self.column = 1;
						} else {
							self.column += 1;
						}
					}
					if self.is_at_end() {
						has_error = true;
						self.report_lexical_error(
							"Unterminated comment".into(),
							self.line,
							self.column,
						);
					} else {
						self.advance();
						self.column += 1;
					}
				} else if self.match_char('=') {
					self.add_token(TokenType::SlashEqual)
				} else {
					self.add_token(TokenType::Slash);
				}
			}
			'*' => {
				if self.match_char('=') {
					self.add_token(TokenType::StarEqual)
				} else {
					self.add_token(TokenType::Star)
				}
			}
			'^' => {
				if self.match_char('=') {
					self.add_token(TokenType::CaretEqual)
				} else {
					self.add_token(TokenType::Caret)
				}
			}
			'%' => {
				if self.match_char('=') {
					self.add_token(TokenType::PercentEqual)
				} else {
					self.add_token(TokenType::Percent)
				}
			}
			'"' => match self.string() {
				Ok(()) => (),
				Err(()) => {
					has_error = true;
					self.report_lexical_error("Unterminated string".into(), self.line, self.column);
				}
			},
			c => {
				if c.is_ascii_digit() {
					self.number();
				} else if is_alpha(c) {
					self.identifier();
				} else {
					has_error = true;
					self.report_lexical_error(
						format!("Unexpected character `{c}`"),
						self.line,
						self.column,
					);
					self.column += 1;
				}
			}
		};
		if has_error {
			Err(())
		} else {
			Ok(())
		}
	}

	fn advance(&mut self) -> char {
		self.current += 1;
		self.char_at(self.current - 1)
	}

	fn match_char(&mut self, expected: char) -> bool {
		if self.is_at_end() {
			return false;
		}
		if self.char_at(self.current) != expected {
			return false;
		}

		self.current += 1;
		true
	}

	fn peek(&self) -> char {
		if self.is_at_end() {
			return '\0';
		}
		self.char_at(self.current)
	}

	fn peek_next(&self) -> char {
		if self.current + 1 >= self.source.chars().count() as i64 {
			return '\0';
		}
		self.char_at(self.current + 1)
	}

	fn previous(&self) -> char {
		if self.current == 0 {
			return '\0';
		}
		self.char_at(self.current - 1)
	}

	fn char_at(&self, index: i64) -> char {
		self.source.chars().collect::<Vec<char>>()[index as usize]
	}

	fn add_token(&mut self, token_type: TokenType) {
		let lexeme = self
			.source
			.substring(self.start as usize, self.current as usize);

		self.tokens.push(Token {
			token_type: token_type.clone(),
			lexeme: lexeme.into(),
			line: self.line,
			column: self.column,
		});

		match token_type {
			TokenType::EOL => {
				self.line += 1;
				self.column = 1;
			}
			TokenType::StringLiteral(lit) => {
				let newlines = lit.match_indices('\n');
				let count = newlines.clone().count();
				self.line += count as i64;
				if let Some(last) = newlines.last() {
					self.column = lit
						.substring(last.0 + 1, lit.chars().count())
						.chars()
						.count() as i64 + 2;
				} else {
					self.column += lit.chars().count() as i64 + 2;
				}
			}
			_ => self.column += lexeme.chars().count() as i64,
		}
	}

	fn string(&mut self) -> Result<(), ()> {
		while self.peek() != '"' && !self.is_at_end() {
			self.advance();
		}

		if self.is_at_end() {
			return Err(());
		}

		self.advance();

		let literal = self
			.source
			.substring(self.start as usize + 1, self.current as usize - 1)
			.into();
		self.add_token(TokenType::StringLiteral(literal));

		Ok(())
	}

	fn number(&mut self) {
		while self.peek().is_ascii_digit() {
			self.advance();
		}

		if self.peek() == '.' && self.peek_next().is_ascii_digit() {
			self.advance();

			while self.peek().is_ascii_digit() {
				self.advance();
			}
		}

		self.add_token(TokenType::NumberLiteral(
			self.source
				.substring(self.start as usize, self.current as usize)
				.parse()
				.unwrap(),
		));
	}

	fn identifier(&mut self) {
		while is_alpha_numeric(self.peek()) {
			self.advance();
		}

		let ident: String = self
			.source
			.substring(self.start as usize, self.current as usize)
			.into();

		match ident.as_str() {
			"and" => self.add_token(TokenType::And),
			"ask" => self.add_token(TokenType::Ask),
			"boolean" => self.add_token(TokenType::Boolean),
			"break" => self.add_token(TokenType::Break),
			"cmd" => self.add_token(TokenType::Cmd),
			"continue" => self.add_token(TokenType::Continue),
			"default" => self.add_token(TokenType::Default),
			"delete" => self.add_token(TokenType::Delete),
			"else" => self.add_token(TokenType::Else),
			"empty" => self.add_token(TokenType::Empty),
			"f" => self.add_token(TokenType::Function),
			"false" => self.add_token(TokenType::False),
			"for" => self.add_token(TokenType::For),
			"if" => self.add_token(TokenType::If),
			"in" => self.add_token(TokenType::In),
			"keys" => self.add_token(TokenType::Keys),
			"match" => self.add_token(TokenType::Match),
			"number" => self.add_token(TokenType::Number),
			"or" => self.add_token(TokenType::Or),
			"out" => self.add_token(TokenType::Out),
			"return" => self.add_token(TokenType::Return),
			"size" => self.add_token(TokenType::Size),
			"string" => self.add_token(TokenType::String),
			"true" => self.add_token(TokenType::True),
			"while" => self.add_token(TokenType::While),
			_ => self.add_token(TokenType::Identifier),
		}
	}
}

fn is_alpha(character: char) -> bool {
	character.is_ascii_alphabetic() || character == '_'
}

fn is_alpha_numeric(character: char) -> bool {
	is_alpha(character) || character.is_ascii_digit()
}
