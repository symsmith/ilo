use error_manager::{report_error, ErrorDetails, ErrorType};
use lexer::{Token, TokenType};

#[derive(Debug)]
pub enum Statement {
	Expr { expr: Expr },
	Out { expr: Expr },
	Assignment { ident: Token, value: Expr },
}

#[derive(Debug)]
pub enum Expr {
	Primary {
		value: Token,
	},
	Unary {
		operator: Token,
		expr: Box<Expr>,
	},
	Binary {
		left_expr: Box<Expr>,
		operator: Token,
		right_expr: Box<Expr>,
	},
	/* uncomment when working on ? : syntax
	Ternary {
		operator: TernaryOperator,
		left_expr: Box<Expr>,
		middle_expr: Box<Expr>,
		right_expr: Box<Expr>,
	},*/
	Grouping {
		expr: Box<Expr>,
	},
	Variable {
		name: Token,
	},
}

pub struct Parser {
	tokens: Vec<Token>,
	current: i64,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, current: 0 }
	}

	pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
		let mut statements: Vec<Statement> = vec![];

		let mut has_error = false;

		while !self.is_at_end() {
			if self.match_one(TokenType::EOL) {
				continue;
			}

			let statement = self.statement();

			if let Ok(statement) = statement {
				statements.push(statement);
			} else {
				has_error = true;
				self.synchronize();
			}
		}

		if has_error {
			Err(())
		} else {
			Ok(statements)
		}
	}

	fn synchronize(&mut self) {
		self.advance();

		while !self.is_at_end() {
			if self.previous().token_type() == TokenType::EOL {
				return;
			}

			match self.peek().token_type() {
				TokenType::Function
				| TokenType::For
				| TokenType::If
				| TokenType::While
				| TokenType::Out
				| TokenType::Return
				| TokenType::Match
				| TokenType::Size
				| TokenType::Cmd
				| TokenType::Delete
				| TokenType::Keys => {
					return;
				}
				_ => self.advance(),
			};
		}
	}

	fn advance(&mut self) -> Token {
		if !self.is_at_end() {
			self.current += 1;
		}

		self.previous()
	}

	fn backtrack(&mut self) {
		if self.current != 0 {
			self.current -= 1;
		}
	}

	fn previous(&self) -> Token {
		self.tokens[self.current as usize - 1].to_owned()
	}

	fn is_at_end(&self) -> bool {
		self.peek().token_type() == TokenType::EOF
	}

	fn peek(&self) -> Token {
		self.tokens[self.current as usize].to_owned()
	}

	fn match_one(&mut self, token_type: TokenType) -> bool {
		if self.next_is(token_type) {
			self.advance();
			return true;
		}

		false
	}

	fn match_any(&mut self, types: Vec<TokenType>) -> bool {
		for token_type in types {
			if self.next_is(token_type) {
				self.advance();
				return true;
			}
		}

		false
	}

	fn next_is(&self, token_type: TokenType) -> bool {
		if self.is_at_end() {
			return false;
		}

		self.peek().token_type() == token_type
	}

	fn report_parsing_error(&self, message: String, token: Token) {
		report_error(ErrorDetails::new(
			ErrorType::ParsingError,
			message,
			token.line(),
			token.column(),
		))
	}

	fn consume_or_report(
		&mut self,
		token_type: TokenType,
		error_message: String,
	) -> Result<Token, ()> {
		if self.next_is(token_type) {
			return Ok(self.advance());
		}
		self.report_parsing_error(error_message, self.peek());
		Err(())
	}

	fn consume_eol_or_report(&mut self, error_message: String) -> Result<Token, ()> {
		if self.peek().token_type() == TokenType::EOF || self.next_is(TokenType::EOL) {
			return Ok(self.advance());
		}
		self.report_parsing_error(error_message, self.peek());
		Err(())
	}

	fn statement(&mut self) -> Result<Statement, ()> {
		if self.match_one(TokenType::Out) {
			return self.output_statement();
		} else if self.match_one(TokenType::Identifier) {
			if self.peek().token_type() == TokenType::Equal {
				return self.assign_statement();
			} else {
				// if we are at an expression statement using an identifier,
				// it is already consumed by now, so we backtrack
				self.backtrack();
			}
		}

		self.expression_statement()
	}

	fn output_statement(&mut self) -> Result<Statement, ()> {
		self.consume_or_report(
			TokenType::LeftParen,
			"'out' is a reserved keyword to output an expression. Usage: out(...)".into(),
		)?;

		let expr = self.expression()?;

		self.consume_or_report(
			TokenType::RightParen,
			"Missing ')' after output statement ('out')".into(),
		)?;

		self.consume_eol_or_report("Line must end after an output statement".into())?;

		Ok(Statement::Out { expr })
	}

	fn assign_statement(&mut self) -> Result<Statement, ()> {
		let ident = self.previous();

		self.advance();

		let value = self.expression()?;

		self.consume_eol_or_report("Line must end after an assignment".into())?;

		Ok(Statement::Assignment { ident, value })
	}

	fn expression_statement(&mut self) -> Result<Statement, ()> {
		let expr = self.expression()?;

		self.consume_eol_or_report("Line must end after an expression statement".into())?;

		Ok(Statement::Expr { expr })
	}

	fn expression(&mut self) -> Result<Expr, ()> {
		self.equality()
	}

	fn equality(&mut self) -> Result<Expr, ()> {
		let mut expr = self.comparison()?;

		while self.match_any(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
			let operator = self.previous();
			let right = self.comparison()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn comparison(&mut self) -> Result<Expr, ()> {
		let mut expr = self.term()?;

		while self.match_any(vec![
			TokenType::GreaterEqual,
			TokenType::Greater,
			TokenType::LessEqual,
			TokenType::Less,
		]) {
			let operator = self.previous();
			let right = self.term()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn term(&mut self) -> Result<Expr, ()> {
		let mut expr = self.modulo()?;

		while self.match_any(vec![TokenType::Minus, TokenType::Plus]) {
			let operator = self.previous();
			let right = self.modulo()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn modulo(&mut self) -> Result<Expr, ()> {
		let mut expr = self.factor()?;

		while self.match_one(TokenType::Percent) {
			let operator = self.previous();
			let right = self.factor()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn factor(&mut self) -> Result<Expr, ()> {
		let mut expr = self.exponentiation()?;

		while self.match_any(vec![TokenType::Slash, TokenType::Star]) {
			let operator = self.previous();
			let right = self.exponentiation()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn exponentiation(&mut self) -> Result<Expr, ()> {
		let mut expr = self.unary()?;

		while self.match_one(TokenType::Caret) {
			let operator = self.previous();
			let right = self.unary()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn unary(&mut self) -> Result<Expr, ()> {
		if self.match_any(vec![TokenType::Minus, TokenType::Bang]) {
			let operator = self.previous();
			let expr = self.unary()?;
			return Ok(Expr::Unary {
				operator,
				expr: Box::new(expr),
			});
		}

		self.primary()
	}

	fn primary(&mut self) -> Result<Expr, ()> {
		if self.match_any(vec![TokenType::False, TokenType::True]) {
			return Ok(Expr::Primary {
				value: self.previous(),
			});
		}

		match self.peek().token_type() {
			TokenType::StringLiteral(_) | TokenType::NumberLiteral(_) => {
				self.advance();
				return Ok(Expr::Primary {
					value: self.previous(),
				});
			}
			TokenType::Identifier => {
				self.advance();
				return Ok(Expr::Variable {
					name: self.previous(),
				});
			}
			_ => (),
		}

		if self.match_one(TokenType::LeftParen) {
			let expr = self.expression()?;
			self.consume_or_report(
				TokenType::RightParen,
				"Expected closing ')' after expression".into(),
			)?;
			return Ok(Expr::Grouping {
				expr: Box::new(expr),
			});
		}

		self.report_parsing_error(
			format!("Incorrect token '{}'", self.peek().lexeme()),
			self.peek(),
		);
		Err(())
	}
}
