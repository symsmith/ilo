use crate::{
	errors::{report_error, ErrorDetails},
	lexer::{Token, TokenType},
};

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
}

pub struct Parser {
	tokens: Vec<Token>,
	current: i64,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, current: 0 }
	}

	pub fn parse(&mut self) -> Result<Expr, ()> {
		self.expression()
	}

	fn advance(&mut self) -> Token {
		if !self.is_at_end() {
			self.current += 1;
		}

		self.previous()
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

	fn consume_or_report(
		&mut self,
		token_type: TokenType,
		error_message: String,
	) -> Result<Token, ()> {
		if self.next_is(token_type) {
			return Ok(self.advance());
		}
		report_error(ErrorDetails::ParsingError {
			message: error_message,
			line: self.peek().line(),
			column: self.peek().column(),
		});
		Err(())
	}

	fn expression(&mut self) -> Result<Expr, ()> {
		self.equality()
	}

	fn equality(&mut self) -> Result<Expr, ()> {
		let mut expr = self.comparison()?;

		while self.match_any(vec![TokenType::BangEqual, TokenType::EqualEqual])
		{
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

		while self.match_any(vec![TokenType::Percent]) {
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
		let mut expr = self.power()?;

		while self.match_any(vec![TokenType::Slash, TokenType::Star]) {
			let operator = self.previous();
			let right = self.power()?;
			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			};
		}

		Ok(expr)
	}

	fn power(&mut self) -> Result<Expr, ()> {
		let mut expr = self.unary()?;

		while self.match_any(vec![TokenType::Caret]) {
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
			_ => (),
		}

		if self.match_any(vec![TokenType::LeftParen]) {
			let expr = self.expression()?;
			self.consume_or_report(
				TokenType::RightParen,
				"Expected closing ')' after expression".into(),
			)?;
			return Ok(Expr::Grouping {
				expr: Box::new(expr),
			});
		}

		report_error(ErrorDetails::ParsingError {
			message: format!("Incorrect token '{}'", self.peek().lexeme()),
			line: self.peek().line(),
			column: self.peek().column(),
		});
		Err(())
	}
}
