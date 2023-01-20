use error_manager::{report_error, ErrorDetails, ErrorType};
use lexer::{Token, TokenType};
use parser::{Expr, Statement};
use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug)]
enum Value {
	Boolean(bool),
	Number(f64),
	String(String),
}

impl Value {
	fn get_type(&self) -> String {
		match self {
			Self::Boolean(_) => String::from("boolean"),
			Self::Number(_) => String::from("number"),
			Self::String(_) => String::from("string"),
		}
	}
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

struct Environment {
	values: HashMap<String, Value>,
}

impl Environment {
	fn new() -> Self {
		Self {
			values: HashMap::new(),
		}
	}

	fn define_or_assign(&mut self, name: String, value: Value) -> Result<(), Value> {
		if let Some(current_value) = self.values.get(&name) {
			let current_value = current_value.to_owned();
			if current_value.get_type() == value.get_type() {
				self.values.insert(name, value);
			} else {
				return Err(current_value);
			}
		} else {
			self.values.insert(name, value);
		}
		Ok(())
	}

	fn get(&mut self, name: String) -> Result<Value, ()> {
		if let Some(value) = self.values.get(&name) {
			Ok(value.to_owned())
		} else {
			Err(())
		}
	}
}

pub struct Interpreter {
	environment: Environment,
}

impl Interpreter {
	pub fn new() -> Self {
		Self {
			environment: Environment::new(),
		}
	}

	pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<String, ()> {
		let mut result = String::new();
		for statement in statements {
			result = format!("{}", self.execute(statement)?);
		}
		Ok(result)
	}

	fn report_runtime_error(&self, token: &Token, message: String) -> Result<Value, ()> {
		report_error(ErrorDetails::new(
			ErrorType::RuntimeError,
			message,
			token.line(),
			token.column(),
		));
		Err(())
	}

	fn report_type_error(&self, token: &Token, message: String) -> Result<Value, ()> {
		report_error(ErrorDetails::new(
			ErrorType::TypeError,
			message,
			token.line(),
			token.column(),
		));
		Err(())
	}

	fn execute(&mut self, statement: Statement) -> Result<Value, ()> {
		match statement {
			Statement::Expr { expr } => self.evaluate(expr),
			Statement::Out { expr } => self.execute_output(expr),
			Statement::Assignment { ident, value } => self.execute_assignment(ident, value),
		}
	}

	fn execute_output(&mut self, expr: Expr) -> Result<Value, ()> {
		let value = self.evaluate(expr)?;
		println!("{value}");
		Ok(Value::String(String::new()))
	}

	fn execute_assignment(&mut self, ident: Token, value: Expr) -> Result<Value, ()> {
		let value = self.evaluate(value)?;
		if let Err(old_value) = self
			.environment
			.define_or_assign(ident.lexeme().into(), value.clone())
		{
			self.report_type_error(&ident, format!("Variable {} already exists, but has a different type (tried to replace {} with {})", ident.lexeme(), old_value, value))
		} else {
			Ok(Value::String(String::new()))
		}
	}

	fn evaluate(&mut self, expr: Expr) -> Result<Value, ()> {
		match expr {
			Expr::Primary { value } => self.evaluate_primary(value),
			Expr::Unary { operator, expr } => self.evaluate_unary(operator, *expr),
			Expr::Binary {
				left_expr,
				operator,
				right_expr,
			} => self.evaluate_binary(*left_expr, operator, *right_expr),
			Expr::Grouping { expr } => self.evaluate(*expr),
			Expr::Variable { name } => self.evaluate_variable(name),
		}
	}

	fn evaluate_variable(&mut self, name: Token) -> Result<Value, ()> {
		if let Ok(value) = self.environment.get(name.lexeme().into()) {
			Ok(value)
		} else {
			self.report_runtime_error(&name, format!("Undefined symbol {}", name.lexeme()))
		}
	}

	fn evaluate_primary(&self, value: Token) -> Result<Value, ()> {
		match value.token_type() {
			TokenType::True => Ok(Value::Boolean(true)),
			TokenType::False => Ok(Value::Boolean(false)),
			TokenType::NumberLiteral(number) => Ok(Value::Number(number)),
			TokenType::StringLiteral(string) => Ok(Value::String(string)),
			_ => unreachable!("Value cannot be anything else"),
		}
	}

	fn evaluate_unary(&mut self, operator: Token, expr: Expr) -> Result<Value, ()> {
		let value = self.evaluate(expr)?;
		match operator.token_type() {
			TokenType::Bang => {
				if let Value::Boolean(value) = value {
					Ok(Value::Boolean(!value))
				} else {
					self.report_type_error(
						&operator,
						format!("Unary not (!) must be applied to a boolean (found {value})",),
					)
				}
			}
			TokenType::Minus => {
				if let Value::Number(value) = value {
					Ok(Value::Number(-value))
				} else {
					self.report_type_error(
						&operator,
						format!("Unary minus (-) must be applied to a number (found {value})",),
					)
				}
			}
			_ => unreachable!("Operator cannot be anything else"),
		}
	}

	fn evaluate_binary(
		&mut self,
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
			_ => unreachable!("Operator cannot be anything else"),
		}
	}

	fn evaluate_equality(
		&self,
		left_value: Value,
		operator: Token,
		right_value: Value,
	) -> Result<Value, ()> {
		match left_value {
			Value::Boolean(left_value) => match right_value {
				Value::Boolean(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => Ok(Value::Boolean(
					operator.token_type() != TokenType::EqualEqual,
				)),
			},
			Value::Number(left_value) => match right_value {
				Value::Number(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => Ok(Value::Boolean(
					operator.token_type() != TokenType::EqualEqual,
				)),
			},
			Value::String(left_value) => match right_value {
				Value::String(right_value) => Ok(Value::Boolean(
					if operator.token_type() == TokenType::EqualEqual {
						left_value == right_value
					} else {
						left_value != right_value
					},
				)),
				_ => Ok(Value::Boolean(
					operator.token_type() != TokenType::EqualEqual,
				)),
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
			self.report_type_error(
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
			self.report_type_error(
				&operator,
				format!(
					"{} ({}) can only be performed between two numbers{} (found {} and {})",
					match operator.token_type() {
						TokenType::Plus => "Addition",
						TokenType::Minus => "Substraction",
						TokenType::Star => "Multiplication",
						TokenType::Slash => "Division",
						TokenType::Percent => "Modulo",
						TokenType::Caret => "Exponentiation",
						_ => unreachable!("Operator cannot be anything else"),
					},
					operator.lexeme(),
					if operator.token_type() == TokenType::Plus {
						" or two strings"
					} else if operator.token_type() == TokenType::Star {
						" or a string and a number"
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
							return self.report_runtime_error(&operator, format!("Multiplication between a string and a number requires an integer (found {right_value})"));
						}

						let right_value = right_value as i64;

						if right_value < 0 {
							return self.report_runtime_error(&operator, format!("Multiplication between a string and a number requires a positive integer (found {right_value})"));
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
