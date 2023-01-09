use std::fmt::{Debug, Display};
use substring::Substring;

use crate::errors::{report_error, ErrorDetails};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType<'a> {
	// Single character tokens
	LeftBrace, // {}
	RightBrace,
	LeftBracket, // []
	RightBracket,
	Comma, // ,
	Colon, // :
	Interrogation,
	LeftParen,
	RightParen,

	// 1-2-3 character tokens
	Arrow,
	Bang,
	BangEqual,
	Dot,
	DotDotDot,
	Equal,
	EqualEqual,
	Greater,
	GreaterEqual,
	Less,
	LessEqual,
	Minus,
	MinusEqual,
	MinusMinus,
	Plus,
	PlusEqual,
	PlusPlus,
	Slash,
	SlashEqual,
	Star,
	StarEqual,

	// Literals
	Identifier(&'a String),
	NumberLit(&'a i64),
	StringLit(&'a String),

	// Reserved keywords
	And,
	Ask,
	Boolean,
	Break,
	Cmd,
	Continue,
	Default,
	Delete,
	Else,
	Empty,
	False,
	For,
	Function,
	If,
	In,
	Keys,
	Match,
	Number,
	Or,
	Out,
	Return,
	Size,
	String,
	True,
	While,

	EOL,
	EOF,
}

#[derive(Debug)]
pub struct Token<'a> {
	token_type: TokenType<'a>,
	/// Textual representation of the token, as is in the source code
	lexeme: String,
	/// Line of the start of the token
	line: i64,
	/// Column of the start of the token
	column: i64,
}

impl<'a> Token<'a> {
	fn new(
		token_type: TokenType<'a>,
		lexeme: String,
		line: i64,
		column: i64,
	) -> Self {
		Self {
			token_type,
			lexeme,
			line,
			column,
		}
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let value: String = match self.token_type {
			TokenType::Identifier(str) => str.to_owned(),
			TokenType::NumberLit(num) => num.to_string(),
			_ => String::new(),
		};
		write!(f, "{:?} {} {}", self.token_type, self.lexeme, &value)
	}
}

pub struct Lexer<'a> {
	source: String,
	tokens: Vec<Token<'a>>,
	/// Offset of the start of the current lexeme
	start: i64,
	/// Offset of the current scanned character
	current: i64,
	/// Line of the start of the current lexeme
	line: i64,
	/// Column of the start of the current lexeme
	column: i64,
}

impl<'a> Lexer<'a> {
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

	pub fn scan_tokens(&mut self) -> Result<&Vec<Token<'a>>, ()> {
		let mut has_error = false;
		while !self.is_at_end() {
			self.start = self.current;
			match self.scan_token() {
				Err(()) => has_error = true,
				Ok(()) => (),
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
			Ok(&self.tokens)
		}
	}

	fn is_at_end(&self) -> bool {
		self.current >= self.source.chars().count() as i64
	}

	fn scan_token(&mut self) -> Result<(), ()> {
		let mut has_error = false;
		match self.advance() {
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
					while !(self.peek() == '/' && self.previous() == '*')
						&& !self.is_at_end()
					{
						if self.advance() == '\n' {
							self.line += 1;
							self.column = 1;
						} else {
							self.column += 1;
						}
					}
					self.advance();
					self.column += 1;
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
			c => {
				has_error = true;
				report_error(ErrorDetails::LexicalError {
					message: format!("Unexpected character `{c}`"),
					line: self.line,
					column: self.column,
				});
				self.column += 1;
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

	fn previous(&self) -> char {
		if self.current == 0 {
			return '\0';
		}
		self.char_at(self.current - 1)
	}

	fn char_at(&self, index: i64) -> char {
		self.source.chars().collect::<Vec<char>>()[index as usize]
	}

	fn add_token(&mut self, token_type: TokenType<'a>) {
		let lexeme = self
			.source
			.substring(self.start as usize, self.current as usize);

		self.tokens.push(Token {
			token_type,
			lexeme: lexeme.into(),
			line: self.line,
			column: self.column,
		});

		if token_type == TokenType::EOL {
			self.line += 1;
			self.column = 1;
		} else {
			self.column += lexeme.chars().count() as i64;
		}
	}
}
