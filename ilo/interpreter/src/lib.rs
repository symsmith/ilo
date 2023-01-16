use error_manager::{report_error, ErrorDetails};
use lexer::{Token, TokenType};
use parser::{Expr, Statement};
use std::fmt::Display;

#[derive(Clone)]
enum Value {
	Boolean(bool),
	Number(f64),
	String(String),
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(boolean) => write!(f, "{boolean}"),
			Self::Number(number) => {
				write!(f, "{}", if number == &0.0 { &0.0 } else { number })
			}
			Self::String(string) => write!(f, "{string}"),
		}
	}
}

pub struct Interpreter {}

impl Interpreter {
	pub fn new() -> Self {
		Self {}
	}

	pub fn interpret(&self, statements: Vec<Statement>) -> Result<(), ()> {
		for statement in statements {
			self.execute(statement)?;
		}
		Ok(())
	}

	fn report_runtime_error(&self, token: &Token, message: String) -> Result<Value, ()> {
		report_error(ErrorDetails::RuntimeError {
			message,
			line: token.line(),
			column: token.column(),
		});
		Err(())
	}

	fn execute(&self, statement: Statement) -> Result<(), ()> {
		match statement {
			Statement::Expr { expr } => {
				self.evaluate(expr)?;
			}
			Statement::Out { expr } => {
				self.execute_output(expr)?;
			}
		}

		Ok(())
	}

	fn execute_output(&self, expr: Expr) -> Result<(), ()> {
		let value = self.evaluate(expr)?;
		println!("{value}");
		Ok(())
	}

	fn evaluate(&self, expr: Expr) -> Result<Value, ()> {
		match expr {
			Expr::Primary { value } => self.evaluate_primary(value),
			Expr::Unary { operator, expr } => self.evaluate_unary(operator, *expr),
			Expr::Binary {
				left_expr,
				operator,
				right_expr,
			} => self.evaluate_binary(*left_expr, operator, *right_expr),
			Expr::Grouping { expr } => self.evaluate(*expr),
		}
	}

	fn evaluate_primary(&self, value: Token) -> Result<Value, ()> {
		match value.token_type() {
			TokenType::True => Ok(Value::Boolean(true)),
			TokenType::False => Ok(Value::Boolean(false)),
			TokenType::NumberLiteral(number) => Ok(Value::Number(number)),
			TokenType::StringLiteral(string) => Ok(Value::String(string)),
			_ => self.report_runtime_error(&value, format!("Illegal value '{}'", value.lexeme())),
		}
	}

	fn evaluate_unary(&self, operator: Token, expr: Expr) -> Result<Value, ()> {
		let value = self.evaluate(expr)?;
		match operator.token_type() {
			TokenType::Bang => {
				if let Value::Boolean(value) = value {
					Ok(Value::Boolean(!value))
				} else {
					self.report_runtime_error(
						&operator,
						format!("Unary not (!) must be applied to a boolean (found '{value}')",),
					)
				}
			}
			TokenType::Minus => {
				if let Value::Number(value) = value {
					Ok(Value::Number(-value))
				} else {
					self.report_runtime_error(
						&operator,
						format!("Unary minus (-) must be applied to a number (found '{value}')",),
					)
				}
			}
			_ => self.report_runtime_error(
				&operator,
				format!("Illegal unary operator '{}'", operator.lexeme()),
			),
		}
	}

	fn evaluate_binary(
		&self,
		left_expr: Expr,
		operator: Token,
		right_expr: Expr,
	) -> Result<Value, ()> {
		let left_value = self.evaluate(left_expr)?;
		let right_value = self.evaluate(right_expr)?;

		match operator.token_type() {
			TokenType::BangEqual | TokenType::EqualEqual => {
				self.evaluate_equality(left_value, operator, right_value)
			}
			TokenType::Greater
			| TokenType::GreaterEqual
			| TokenType::Less
			| TokenType::LessEqual => self.evaluate_comparison(left_value, operator, right_value),
			TokenType::Plus
			| TokenType::Minus
			| TokenType::Star
			| TokenType::Slash
			| TokenType::Percent
			| TokenType::Caret => self.evaluate_math_operation(left_value, operator, right_value),
			_ => self.report_runtime_error(
				&operator,
				format!("Illegal binary operator '{}'", operator.lexeme()),
			),
		}
	}

	fn evaluate_equality(
		&self,
		left_value: Value,
		operator: Token,
		right_value: Value,
	) -> Result<Value, ()> {
		let error =
			|| {
				self.report_runtime_error(
					&operator,
					format!(
					"Operands of the {} operator ({}) must have the same type (found {} and {})",
					if operator.token_type() == TokenType::EqualEqual {"Equal"} else {"Not equal"},
					operator.lexeme(),
					left_value,
					right_value
				),
				)
			};
		match left_value.clone() {
			Value::Boolean(left_value) => match right_value {
				Value::Boolean(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => error(),
			},
			Value::Number(left_value) => match right_value {
				Value::Number(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => error(),
			},
			Value::String(left_value) => match right_value {
				Value::String(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => error(),
			},
		}
	}

	fn evaluate_comparison(
		&self,
		left_value: Value,
		operator: Token,
		right_value: Value,
	) -> Result<Value, ()> {
		let error = || {
			self.report_runtime_error(
				&operator,
				format!(
					"Comparison can only be performed between two numbers (found {} and {})",
					left_value, right_value
				),
			)
		};
		match left_value {
			Value::Number(left_value) => match right_value {
				Value::Number(right_value) => Ok(Value::Boolean(match operator.token_type() {
					TokenType::Greater => left_value > right_value,
					TokenType::GreaterEqual => left_value >= right_value,
					TokenType::Less => left_value < right_value,
					TokenType::LessEqual => left_value <= right_value,
					_ => unreachable!("Operator cannot be anything else"),
				})),
				_ => error(),
			},
			_ => error(),
		}
	}

	fn evaluate_math_operation(
		&self,
		left_value: Value,
		operator: Token,
		right_value: Value,
	) -> Result<Value, ()> {
		let error = || {
			self.report_runtime_error(
				&operator,
				format!(
					"{} ({}) can only be performed between two numbers{} (found {} and {})",
					match operator.token_type() {
						TokenType::Plus => "Addition",
						TokenType::Minus => "Substraction",
						TokenType::Star => "Multiplication",
						TokenType::Slash => "Division",
						TokenType::Percent => "Modulo",
						TokenType::Caret => "Power",
						_ => unreachable!("Operator cannot be anything else"),
					},
					operator.lexeme(),
					if operator.token_type() == TokenType::Plus {
						" or two strings"
					} else {
						""
					},
					left_value,
					right_value
				),
			)
		};
		match left_value.clone() {
			Value::Number(left_value) => match right_value {
				Value::Number(right_value) => Ok(Value::Number(match operator.token_type() {
					TokenType::Plus => left_value + right_value,
					TokenType::Minus => left_value - right_value,
					TokenType::Star => left_value * right_value,
					TokenType::Slash => left_value / right_value,
					TokenType::Percent => left_value.rem_euclid(right_value),
					TokenType::Caret => left_value.powf(right_value),
					_ => unreachable!("Operator cannot be anything else"),
				})),
				_ => error(),
			},
			Value::String(left_value) => match right_value {
				Value::String(right_value) => {
					if operator.token_type() == TokenType::Plus {
						Ok(Value::String(format!("{left_value}{right_value}")))
					} else {
						self.report_runtime_error(
							&operator,
							"Only addition (+) can be used between two strings".into(),
						)
					}
				}
				Value::Number(right_value) => {
					if operator.token_type() == TokenType::Star {
						if right_value.round() != right_value {
							return self.report_runtime_error(&operator, format!("Addition between a string and a number requires an integer (found {right_value})"));
						}

						let right_value = right_value as i64;

						if right_value < 0 {
							return self.report_runtime_error(&operator, format!("Addition between a string and a number requires a positive integer (found {right_value})"));
						}

						let mut result = String::new();
						for _ in 0..right_value {
							result.push_str(&left_value);
						}
						Ok(Value::String(result))
					} else {
						self.report_runtime_error(
							&operator,
							"Only multiplication (*) can be used between a string and a number"
								.into(),
						)
					}
				}
				_ => error(),
			},
			_ => error(),
		}
	}
}
