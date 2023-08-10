use error_manager::{report_error, ErrorDetails, ErrorType};
use lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Statement {
	Expr {
		expr: Expr,
	},
	Assignment {
		ident: Token,
		value: Expr,
	},
	Block {
		statements: Vec<Statement>,
	},
	If {
		condition: Expr,
		then: Box<Statement>,
		otherwise: Option<Box<Statement>>,
	},
	While {
		condition: Expr,
		body: Box<Statement>,
	},
}

impl Statement {
	pub fn first_token(&self) -> &Token {
		match self {
			Statement::Expr { expr } => expr.first_token(),
			Statement::Assignment { ident, value: _ } => ident,
			Statement::Block { statements: _ } => {
				unreachable!("first_token should not be accessed on a block")
			}
			Statement::If {
				condition,
				then: _,
				otherwise: _,
			} => condition.first_token(),
			Statement::While { condition, body: _ } => condition.first_token(),
		}
	}
}

#[derive(Debug, Clone)]
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
	Grouping {
		expr: Box<Expr>,
	},
	Variable {
		name: Token,
	},
	Call {
		callee: Box<Expr>,
		closing_paren: Token,
		arguments: Vec<Box<Expr>>,
	},
}

impl Expr {
	pub fn first_token(&self) -> &Token {
		match self {
			Expr::Primary { value } => value,
			Expr::Unary { operator, expr: _ } => operator,
			Expr::Binary {
				left_expr,
				operator: _,
				right_expr: _,
			} => left_expr.first_token(),
			Expr::Grouping { expr } => expr.first_token(),
			Expr::Variable { name } => name,
			Expr::Call {
				callee,
				closing_paren: _,
				arguments: _,
			} => callee.first_token(),
		}
	}
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
				| TokenType::Return
				| TokenType::Match
				| TokenType::Delete => {
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

	fn ignore_empty_lines(&mut self) {
		while self.peek().token_type() == TokenType::EOL && !self.is_at_end() {
			self.advance();
		}
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
		if self.match_one(TokenType::Identifier) {
			if self.peek().token_type() == TokenType::Equal {
				return self.assign_statement();
			} else {
				// if we are at an expression statement using an identifier,
				// it is already consumed by now, so we backtrack
				self.backtrack();
			}
		} else if self.match_one(TokenType::LeftBrace) {
			return Ok(Statement::Block {
				statements: self.block_statement()?,
			});
		} else if self.match_one(TokenType::If) {
			return self.if_statement();
		} else if self.match_one(TokenType::While) {
			return self.while_statement();
		}

		self.expression_statement()
	}

	fn assign_statement(&mut self) -> Result<Statement, ()> {
		let ident = self.previous();

		self.advance();

		let value = if self.match_one(TokenType::Empty) {
			self.empty_type()
		} else {
			self.expression()
		}?;

		self.consume_eol_or_report("Line must end after an assignment".into())?;

		Ok(Statement::Assignment { ident, value })
	}

	fn empty_type(&mut self) -> Result<Expr, ()> {
		if !self.match_one(TokenType::LeftParen) {
			return Ok(Expr::Primary {
				value: self.previous(),
			});
		}

		if !self.match_any(vec![TokenType::Boolean, TokenType::Number]) {
			if self.peek().token_type() == TokenType::String {
				self.report_parsing_error(
					"Empty string variables must be initialized like: `a = \"\"̀ ".into(),
					self.peek(),
				)
			} else {
				self.report_parsing_error(
					format!(
						"Empty variables must be initialized like: `a = empty(TYPE)`, where {}",
						"`TYPE` is either `boolean` or `number`"
					),
					self.peek(),
				);
			}
			return Err(());
		}

		let primary_type = self.previous();

		let result = Expr::Primary {
			value: primary_type,
		};

		self.consume_or_report(
			TokenType::RightParen,
			"Missing `)` after empty variable assignment".into(),
		)?;

		Ok(result)
	}

	fn block_statement(&mut self) -> Result<Vec<Statement>, ()> {
		self.consume_or_report(TokenType::EOL, "Expected new line after block start".into())?;

		let mut statements: Vec<Statement> = vec![];

		while !self.next_is(TokenType::RightBrace) && !self.is_at_end() {
			if self.match_one(TokenType::EOL) {
				continue;
			}

			statements.push(self.statement()?);
		}

		self.consume_or_report(
			TokenType::RightBrace,
			"Expected `}` after block statement".into(),
		)?;

		Ok(statements)
	}

	fn if_statement(&mut self) -> Result<Statement, ()> {
		let condition = self.expression()?;

		self.consume_or_report(
			TokenType::LeftBrace,
			"Block statement needed after the condition in an `if` statement".into(),
		)?;

		let then_branch = Statement::Block {
			statements: self.block_statement()?,
		};

		let mut else_branch: Option<Box<Statement>> = None;

		self.ignore_empty_lines();

		if self.match_one(TokenType::Else) {
			if self.match_one(TokenType::If) {
				else_branch = Some(Box::new(self.if_statement()?));
			} else {
				self.consume_or_report(
					TokenType::LeftBrace,
					"Block statement needed after the condition in an `else` statement".into(),
				)?;
				else_branch = Some(Box::new(Statement::Block {
					statements: self.block_statement()?,
				}));
			}
		}

		Ok(Statement::If {
			condition,
			then: Box::new(then_branch),
			otherwise: else_branch,
		})
	}

	fn while_statement(&mut self) -> Result<Statement, ()> {
		let condition = self.expression()?;

		self.consume_or_report(
			TokenType::LeftBrace,
			"Block statement needed after the condition in a `while` statement".into(),
		)?;

		let body = Statement::Block {
			statements: self.block_statement()?,
		};

		Ok(Statement::While {
			condition,
			body: Box::new(body),
		})
	}

	fn expression_statement(&mut self) -> Result<Statement, ()> {
		let expr = self.expression()?;

		self.consume_eol_or_report("Line must end after an expression statement".into())?;

		Ok(Statement::Expr { expr })
	}

	fn expression(&mut self) -> Result<Expr, ()> {
		self.or()
	}

	fn or(&mut self) -> Result<Expr, ()> {
		let mut expr = self.and()?;

		while self.match_one(TokenType::Or) {
			let operator = self.previous();
			let right = self.and()?;

			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			}
		}

		Ok(expr)
	}

	fn and(&mut self) -> Result<Expr, ()> {
		let mut expr = self.equality()?;

		while self.match_one(TokenType::And) {
			let operator = self.previous();
			let right = self.equality()?;

			expr = Expr::Binary {
				left_expr: Box::new(expr),
				operator,
				right_expr: Box::new(right),
			}
		}

		Ok(expr)
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
		if self.match_one(TokenType::Empty) {
			return Ok(Expr::Primary {
				value: self.previous(),
			});
		}

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

		self.call()
	}

	fn call(&mut self) -> Result<Expr, ()> {
		let mut expr = self.primary()?;

		loop {
			if self.match_one(TokenType::LeftParen) {
				expr = self.finish_call(expr)?;
			} else {
				break;
			}
		}

		Ok(expr)
	}

	fn finish_call(&mut self, expr: Expr) -> Result<Expr, ()> {
		let mut arguments: Vec<Box<Expr>> = vec![];

		if !self.next_is(TokenType::RightParen) {
			arguments.push(Box::new(self.expression()?));
			while self.match_one(TokenType::Comma) {
				arguments.push(Box::new(self.expression()?));
			}
		}

		let closing_paren = self.consume_or_report(
			TokenType::RightParen,
			"Expected closing `)` after function arguments".into(),
		)?;

		Ok(Expr::Call {
			callee: Box::new(expr),
			closing_paren,
			arguments,
		})
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
				"Expected closing `)` after expression".into(),
			)?;
			return Ok(Expr::Grouping {
				expr: Box::new(expr),
			});
		}

		let peek = self.peek();
		let mut incorrect_lexeme = peek.lexeme();

		match peek.token_type() {
			TokenType::Empty => {
				self.report_parsing_error(
					"The `empty` keyword can only be used in variable declarations or assignments"
						.into(),
					peek,
				);
			}
			_ => {
				if incorrect_lexeme == "\n" {
					incorrect_lexeme = "EOL";
				}

				self.report_parsing_error(format!("Unexpected token `{}`", incorrect_lexeme), peek);
			}
		}

		Err(())
	}
}
