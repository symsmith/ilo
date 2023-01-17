use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn run(source: String) -> Result<String, ()> {
	let mut lexer = Lexer::new(source);
	let tokens = lexer.scan_tokens()?;

	let mut parser = Parser::new(tokens);
	let statements = parser.parse()?;

	let interpreter = Interpreter::new();

	interpreter.interpret(statements)
}

fn ev(source: &str) -> String {
	if let Ok(result) = run(String::from(source)) {
		return result;
	}
	String::from("err")
}

#[test]
fn math_expressions() {
	assert_eq!("2", ev("1+1"));
	assert_eq!("6", ev("2*3"));
	assert_eq!("6", ev("3*2"));
	assert_eq!("2", ev("1 +            1"));
	assert_eq!("0", ev("1+ -1"));
	assert_eq!("0", ev("1  -1"));
	assert_eq!("0", ev("-1 - -1"));
	assert_eq!("3", ev("3 % 5"));
	assert_eq!("2", ev("7  %5"));
	assert_eq!("2", ev("-3 % 5"));
	assert_eq!("8", ev("2^3"));
	assert_eq!("7", ev("1 + 2 * 3"));
	assert_eq!("9", ev("(1 + 2) * 3"));
	assert_eq!("3", ev("9 / 3"));
	assert_eq!("2.5", ev("5/2"));
	assert_eq!("3", ev("((((((((3))))))))"));
	assert_eq!("4", ev("((((((((3)))))) + 1))"));

	assert_eq!("err", ev("((((((((3)))))))"));
	assert_eq!("err", ev("5/"));
	assert_eq!("err", ev(r#"5/"string""#));
	assert_eq!("err", ev("-true"));
	assert_eq!("err", ev("1 + true"));
	assert_eq!("err", ev("true - 3"));
	assert_eq!("err", ev("false * 4"));
	assert_eq!("err", ev("true / 9"));
	assert_eq!("err", ev("8 % false"));
	assert_eq!("err", ev("-3 ^ true"));
	assert_eq!("err", ev("-true"));
	assert_eq!("err", ev("-true"));
	assert_eq!("err", ev("-true"));
}

#[test]
fn boolean_expressions() {
	assert_eq!("true", ev("true"));
	assert_eq!("false", ev("false"));
	assert_eq!("false", ev("!true"));
	assert_eq!("false", ev("true == false"));
	assert_eq!("false", ev("true != true"));
	assert_eq!("false", ev("1 == 2"));
	assert_eq!("false", ev("1 != 1"));
	assert_eq!("false", ev(r#""string" == "other string""#));
	assert_eq!("false", ev(r#""string" != "string""#));
	assert_eq!("false", ev("1 < -1"));
	assert_eq!("false", ev("-1 > 1"));
	assert_eq!("false", ev("2 ^ (2 * 1) < -0"));
	assert_eq!("false", ev("-10 >= -9"));
	assert_eq!("true", ev("-10 >= -10"));
	assert_eq!("true", ev("-10 <= -9"));
	assert_eq!("true", ev("-10 <= -10"));

	assert_eq!("err", ev("1 == true"));
	assert_eq!("err", ev("true == 3"));
	assert_eq!("err", ev(r#""test"== true"#));
	assert_eq!("err", ev("1 != true"));
	assert_eq!("err", ev("true != 3"));
	assert_eq!("err", ev(r#""test"!= true"#));
	assert_eq!("err", ev("!4"));
	assert_eq!("err", ev("3 < true"));
	assert_eq!("err", ev(r#"true >= "test""#));
	assert_eq!("err", ev(r#"true * true"#));
}

#[test]
fn string_expressions() {
	assert_eq!("hello world", ev(r#""hello world""#));
	assert_eq!("hello world", ev(r#""hello " + "world""#));
	assert_eq!("hello hello hello ", ev(r#""hello " * 3"#));
	assert_eq!("hello ", ev(r#""hello " * 1"#));
	assert_eq!("", ev(r#""hello " * 0"#));

	assert_eq!("err", ev(r#""hello " * "world""#));
	assert_eq!("err", ev(r#""hello " * 3.1"#));
	assert_eq!("err", ev(r#""hello " * -2"#));
	assert_eq!("err", ev(r#""hello " * true"#));
}

#[test]
fn output_statement() {
	assert_eq!("", ev(r#"out("output")"#))
}
