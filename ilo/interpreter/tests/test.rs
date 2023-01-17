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
	assert_eq!("3", ev("3 % 5"));
	assert_eq!("2", ev("7  %5"));
	assert_eq!("2", ev("-3 % 5"));
	assert_eq!("8", ev("2^3"));
	assert_eq!("7", ev("1 + 2 * 3"));
	assert_eq!("9", ev("(1 + 2) * 3"));
	assert_eq!("3", ev("9 / 3"));
	assert_eq!("2.5", ev("5/2"));
}
