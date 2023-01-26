use error_manager::{report_error, ErrorDetails, ErrorType};
use lexer::{Token, TokenType};
use parser::{Expr, Statement};
use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
enum Value {
	Empty,

	EmptyBoolean,
	Boolean(bool),

	EmptyNumber,
	Number(f64),

	String(String),
}

impl Value {
	fn get_type(&self) -> String {
		match self {
			Self::EmptyBoolean | Self::Boolean(_) => String::from("boolean"),
			Self::EmptyNumber | Self::Number(_) => String::from("number"),
			Self::String(_) => String::from("string"),
			Self::Empty => unreachable!("should not have to get type of empty"),
		}
	}

	fn as_empty(&self) -> Self {
		match self {
			Self::Boolean(_) => Self::EmptyBoolean,
			Self::Number(_) => Self::EmptyNumber,
			_ => unreachable!("should not get empty type of an empty type or a string"),
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
			Self::EmptyBoolean | Self::EmptyNumber => write!(f, "empty"),
			Self::Empty => unreachable!("should not have to output empty"),
		}
	}
}

struct Environment {
	scopes: Vec<HashMap<String, Value>>,
}

enum EnvError {
	InvalidType(Value),
	EmptyDeclarationNoType,
}

impl Environment {
	fn new() -> Self {
		Self {
			scopes: vec![HashMap::with_capacity(2)],
		}
	}

	fn enter_scope(&mut self) {
		self.scopes.push(HashMap::with_capacity(2));
	}

	fn leave_scope(&mut self) {
		self.scopes.pop();
	}

	fn get(&self, name: String) -> Option<Value> {
		for scope in self.scopes.iter().rev() {
			if let Some(value) = scope.get(&name) {
				return Some(value.to_owned());
			}
		}
		None
	}

	fn define_or_assign(&mut self, name: String, value: Value) -> Result<(), EnvError> {
		for scope in self.scopes.iter_mut().rev() {
			if let Some(current_value) = scope.get(&name) {
				let current_value = current_value.to_owned();

				let mut value = value;
				if value == Value::Empty {
					value = current_value.as_empty();
				}

				if current_value.get_type() == value.get_type() {
					scope.insert(name, value);
					return Ok(());
				} else {
					return Err(EnvError::InvalidType(current_value));
				}
			}
		}

		// Variable is not defined in any scope
		if value == Value::Empty {
			return Err(EnvError::EmptyDeclarationNoType);
		}

		if let Some(scope) = self.scopes.iter_mut().last() {
			scope.insert(name, value);
		} else {
			unreachable!("scopes list should not be empty");
		}

		Ok(())
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
			Statement::Block { statements } => self.execute_block(statements),
			Statement::If {
				condition,
				then,
				otherwise,
			} => self.execute_if(condition, *then, otherwise),
		}
	}

	fn execute_output(&mut self, expr: Expr) -> Result<Value, ()> {
		let value = self.evaluate(expr)?;
		println!("{value}");
		Ok(Value::String(String::new()))
	}

	fn execute_assignment(&mut self, ident: Token, value: Expr) -> Result<Value, ()> {
		let value = self.evaluate(value)?;

		if let Err(error) = self
			.environment
			.define_or_assign(ident.lexeme().into(), value.clone())
		{
			match error {
				EnvError::EmptyDeclarationNoType => self.report_runtime_error(
					&ident,
					format!(
						"Variable {} cannot be initialized as empty, type must be specified",
						ident.lexeme()
					),
				),
				EnvError::InvalidType(current_value) => self.report_type_error(
					&ident,
					format!(
						"Variable {} already exists, but has a different type (tried to replace {} with {})",
						ident.lexeme(),
						current_value, value
					),
				),
			}
		} else {
			Ok(Value::String(String::new()))
		}
	}

	fn execute_block(&mut self, statements: Vec<Statement>) -> Result<Value, ()> {
		self.environment.enter_scope();

		let result = self.interpret(statements)?;

		self.environment.leave_scope();

		Ok(Value::String(result))
	}

	fn execute_if(
		&mut self,
		condition: Expr,
		then: Statement,
		otherwise: Option<Box<Statement>>,
	) -> Result<Value, ()> {
		let condition_value = self.evaluate(condition.clone())?;

		if condition_value == Value::Boolean(true) {
			self.execute(then)?;
		} else if condition_value == Value::Boolean(false) {
			if let Some(else_branch) = otherwise {
				self.execute(*else_branch)?;
			}
		} else {
			self.report_type_error(
				condition.first_token(),
				"If statement should have a boolean expression as condition".into(),
			)?;
		}

		Ok(Value::String(String::from("")))
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
		if let Some(value) = self.environment.get(name.lexeme().into()) {
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
			TokenType::Boolean => Ok(Value::EmptyBoolean),
			TokenType::Number => Ok(Value::EmptyNumber),
			TokenType::Empty => Ok(Value::Empty),
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

		if operator.token_type() == TokenType::Or || operator.token_type() == TokenType::And {
			self.evaluate_binary_logic(left_value, operator, right_expr)
		} else {
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
	}

	fn evaluate_binary_logic(
		&mut self,
		left_value: Value,
		operator: Token,
		right_expr: Expr,
	) -> Result<Value, ()> {
		match operator.token_type() {
			TokenType::And => {
				if left_value != Value::Boolean(true) {
					return Ok(Value::Boolean(false));
				}
			}
			TokenType::Or => {
				if left_value == Value::Boolean(true) {
					return Ok(left_value);
				}
			}
			_ => unreachable!("operator cannot be anything else"),
		}

		let right_value = self.evaluate(right_expr)?;
		if right_value == Value::Boolean(true) {
			Ok(right_value)
		} else {
			Ok(Value::Boolean(false))
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
			Value::EmptyBoolean => Ok(Value::Boolean(
				if operator.token_type() == TokenType::EqualEqual {
					right_value == Value::EmptyBoolean
				} else {
					right_value != Value::EmptyBoolean
				},
			)),
			Value::EmptyNumber => Ok(Value::Boolean(
				if operator.token_type() == TokenType::EqualEqual {
					right_value == Value::EmptyNumber
				} else {
					right_value != Value::EmptyNumber
				},
			)),
			Value::Empty => unreachable!("should not evaluate equality of empty type"),
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
