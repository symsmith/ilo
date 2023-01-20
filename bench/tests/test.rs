use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn run(source: String) -> Result<String, ()> {
	let mut lexer = Lexer::new(source);
	let tokens = lexer.scan_tokens()?;

	let mut parser = Parser::new(tokens);
	let statements = parser.parse()?;

	let mut interpreter = Interpreter::new();

	interpreter.interpret(statements)
}

fn ev(source: &str) -> String {
	if let Ok(result) = run(String::from(source)) {
		return result;
	}
	String::from("err")
}

fn has_lexical_error(source: &str) -> bool {
	let mut lexer = Lexer::new(String::from(source));
	if let Ok(_) = lexer.scan_tokens() {
		return false;
	}
	true
}

fn has_parsing_error(source: &str) -> bool {
	let mut lexer = Lexer::new(String::from(source));
	let tokens = lexer.scan_tokens().unwrap();

	let mut parser = Parser::new(tokens);

	if let Ok(_) = parser.parse() {
		return false;
	}
	true
}

#[test]
fn lexical_error() {
	assert!(has_lexical_error("something;"));
}

#[test]
fn comments() {
	assert_eq!("", ev("// this is a comment"));
	assert_eq!("", ev("/* this is a comment */"));
	assert_eq!(
		"",
		ev("/*
	now multiline
	*/")
	);

	assert!(has_lexical_error("/* this is an unterminated comment"));
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
	assert_eq!("false", ev("1 == true"));
	assert_eq!("false", ev("true == 3"));
	assert_eq!("false", ev(r#""test"== true"#));
	assert_eq!("true", ev("1 != true"));
	assert_eq!("true", ev("true != 3"));
	assert_eq!("true", ev(r#""test"!= true"#));
	assert_eq!("false", ev("1 < -1"));
	assert_eq!("false", ev("-1 > 1"));
	assert_eq!("false", ev("2 ^ (2 * 1) < -0"));
	assert_eq!("false", ev("-10 >= -9"));
	assert_eq!("true", ev("-10 >= -10"));
	assert_eq!("true", ev("-10 <= -9"));
	assert_eq!("true", ev("-10 <= -10"));

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
	assert_eq!(
		"multiline
	string",
		ev(r#""multiline
	string""#)
	);

	assert_eq!("err", ev(r#""hello " * "world""#));
	assert_eq!("err", ev(r#""hello " * 3.1"#));
	assert_eq!("err", ev(r#""hello " * -2"#));
	assert_eq!("err", ev(r#""hello " * true"#));
	assert_eq!("err", ev(r#""hello " + true"#));
	assert_eq!("err", ev(r#""hello " + 3"#));
	assert_eq!(true, has_lexical_error(r#""unterminated string"#));
}

#[test]
fn output_statement() {
	assert_eq!("", ev(r#"out("output")"#));
	assert_eq!(
		"1",
		ev(r#"out("output")
	1"#)
	);

	assert!(has_parsing_error(r#"out("something""#));
	assert!(has_parsing_error(r#"out("something") 1 + 1"#));
}

#[test]
fn global_variables() {
	assert_eq!(
		"hello world",
		ev(r#"var1 = "hello "
			secnd_var = "world"
			var1 + secnd_var"#)
	);
	assert_eq!(
		"hello world",
		ev(r#"var1 = "hello "
			secnd_var = "world"
			var3 = var1 + secnd_var
			var3"#)
	);
	assert_eq!(
		"redefinition",
		ev(r#"var1 = "hello "
			secnd_var = "world"
			var1 = "redefinition"
			var1"#)
	);
	assert_eq!(
		"8",
		ev("firstVar = 3
			2^ firstVar")
	);
	assert_eq!(
		"2",
		ev("var = 3
			2^((var + 1) / 4)")
	);
	assert_eq!(
		"true",
		ev("firstVar = false
			!firstVar != firstVar")
	);
	assert_eq!(
		"hello world",
		ev(r#"
		
		var1 = "hello "

		varTWO="world"

		var1+varTWO
		
		"#)
	);

	assert_eq!(
		"err",
		ev(r#"var1 = "hello "
			var3"#)
	);
	assert_eq!(
		"err",
		ev(r#"var1 = "hello "
			secnd_var = "world"
			var1 = true"#)
	);
}
