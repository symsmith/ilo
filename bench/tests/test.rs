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
	assert_eq!("true", ev("true and true"));
	assert_eq!("false", ev("true and false"));
	assert_eq!("false", ev("false and true"));
	assert_eq!("false", ev("false and false"));
	assert_eq!("true", ev("true or true"));
	assert_eq!("true", ev("true or false"));
	assert_eq!("true", ev("false or true"));
	assert_eq!("false", ev("false or false"));
	assert_eq!("false", ev("1 and 2"));
	assert_eq!("true", ev("-1 or true"));
	assert_eq!("false", ev(r#""string" and 123"#));
	assert_eq!("false", ev("false or false and true"));
	assert_eq!("true", ev("true and (false or true)"));
	assert_eq!(
		"true",
		ev("a = true
	a == false or a == true")
	);
	assert_eq!("true", ev("empty == empty"));
	assert_eq!("false", ev("empty != empty"));
	assert_eq!("false", ev("empty == 3"));
	assert_eq!("true", ev("empty != 3"));
	assert_eq!("false", ev("empty == true"));
	assert_eq!("true", ev("empty != false"));
	assert_eq!(
		"true",
		ev("b = empty(boolean)
		empty == b")
	);
	assert_eq!(
		"false",
		ev("b = empty(boolean)
		empty != b")
	);
	assert_eq!(
		"true",
		ev("n = empty(number)
		empty == n")
	);
	assert_eq!(
		"false",
		ev("n = empty(number)
		empty != n")
	);
	assert_eq!("true", ev("(empty == empty)"));
	assert_eq!("false", ev("(empty != empty)"));
	assert_eq!("true", ev("(empty == 3) == (3 == empty)"));
	assert_eq!("true", ev("(empty != 3) == (3 != empty)"));
	assert_eq!("true", ev("(empty == true) == (true == empty)"));
	assert_eq!("true", ev("(empty != false) == (false != empty)"));
	assert_eq!(
		"true",
		ev("b = empty(boolean)
		(empty == b) == (b == empty)")
	);
	assert_eq!(
		"true",
		ev("b = empty(boolean)
		(empty != b) == (b != empty)")
	);
	assert_eq!(
		"true",
		ev("n = empty(number)
		(empty == n) == (n == empty)")
	);
	assert_eq!(
		"true",
		ev("n = empty(number)
		(empty != n) == (n != empty)")
	);
	assert_eq!(
		"false",
		ev("b = empty(boolean)
		d = empty(number)
		d == b")
	);

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
fn native_functions() {
	// Output (`out`)
	assert_eq!("", ev(r#"out("output")"#));
	assert_eq!(
		"1",
		ev(r#"out("output")
	1"#)
	);

	assert!(has_parsing_error(r#"out("something""#));
	assert!(has_parsing_error(r#"out("something") 1 + 1"#));
	assert!(has_parsing_error(
		"out(3
		
		out(4
		out(54)"
	));

	// Size (`size`)
	assert_eq!("11", ev(r#"size("hello world")"#));
	assert_eq!("0", ev("size(3)"));
	assert_eq!("0", ev("size(true)"));

	// Command execution (`cmd`)
	assert_eq!("hello world", ev(r#"cmd("echo -n hello world")"#));
	assert_eq!("", ev(r#"cmd("")"#));
	assert_eq!("", ev("cmd(4)"));

	// Equality
	assert_eq!(
		"true",
		ev("a = ask
			b = ask
			b == a")
	);
	assert_eq!(
		"false",
		ev("a = ask
			b = out
			b == a")
	);
	assert_eq!("true", ev("time != 3"));

	// Display
	assert_eq!("f time(0 arguments) { [native code] }", ev("time"));
	assert_eq!("f out(1 argument) { [native code] }", ev("out"));
}

#[test]
fn functions() {
	assert_eq!(
		"",
		ev("f triple(n) {
				n * 3
			}")
	);
	assert_eq!(
		"",
		ev("n = 2
			f triple(n) {
				n * 3
			}")
	);
	assert_eq!(
		"5",
		ev("n = 5
			f count(n) {
				if (n > 1) {
					count(n - 1)
				}
				out(n)
			}
			count(3)
			n")
	);
	assert_eq!(
		"3",
		ev("n = 5
			a = 0
			f count(n) {
				if (n > 1) {
					count(n - 1)
					a = n
				}
				out(n)
			}
			count(3)
			a")
	);
	assert_eq!(
		"f sum(2 arguments) {}",
		ev("f sum(a, b) {
			}
			sum")
	);

	assert_eq!("err", ev("3()"));
	assert_eq!("err", ev(r#""hello"()"#));
	assert_eq!("err", ev("time(3)"));
	assert_eq!("err", ev("out()"));
	assert_eq!("err", ev("time()()"));
	has_parsing_error(
		"f test(f) {
	}",
	);
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

#[test]
fn empty_variables() {
	assert_eq!(
		"",
		ev("var = empty(number)
			var")
	);
	assert_eq!(
		"",
		ev("var = empty(boolean)
			var")
	);
	assert_eq!(
		"3",
		ev("var = empty(number)
			var = 3
			var")
	);
	assert_eq!(
		"false",
		ev("var = empty(boolean)
			var = true
			var == false")
	);
	assert_eq!(
		"",
		ev("var = empty(boolean)
			var = true
			var = empty
			var")
	);
	assert_eq!(
		"",
		ev("var = empty(number)
			var = 43
			var = empty
			var")
	);
	assert_eq!(
		"false",
		ev("var = empty(number)
			var == 3")
	);
	assert_eq!(
		"false",
		ev("var = empty(boolean)
			var2 = false
			var == var2")
	);
	assert_eq!(
		"true",
		ev("var = empty(number)
			var != 3")
	);
	assert_eq!(
		"true",
		ev("var = empty(boolean)
			var2 = false
			var != var2")
	);

	assert_eq!("err", ev("var = empty"));
	assert!(has_parsing_error("var = empty("));
	assert!(has_parsing_error("var = empty(test)"));
	assert!(has_parsing_error("var = empty(number"));
	assert!(has_parsing_error("var = empty(string)"));
}

#[test]
fn block_statements() {
	assert_eq!(
		"",
		ev("{
		a = 2
	}")
	);
	assert_eq!(
		"8",
		ev("a = 4
	{
		a = a * 2
	}
	a")
	);
	assert_eq!(
		"16",
		ev("a = 2
	{
		a = a* 2
		{
			a = a *2
			{
				a = a*2
			}
		}
	}
	a")
	);
	assert_eq!(
		"err",
		ev("a = 2
	{
		a = a* 2
		{
			b = 3
			a = a *2
			{
				a = a*2
			}
		}
		out(b)
	}
	a")
	);

	assert!(has_parsing_error("{}"));
	assert!(has_parsing_error(
		"{out(a)
	}"
	));
}

#[test]
fn if_statements() {
	assert_eq!(
		"1",
		ev("a = 0
		if true {
			a = 1
		} else {
			a = 2
		}
		a")
	);
	assert_eq!(
		"0",
		ev("a = 2
		b = empty(number)
		if a == 3 {
			b = 1
		} else {
			b = 0
		}
		b")
	);
	assert_eq!(
		"1",
		ev("a = 2
		b = empty(number)


		if a == 2 {

			b = 1
		}
		
		
		else {
			b = 0

		}

		b")
	);
	assert_eq!(
		"-1",
		ev("a = 1
		b = empty(number)
		if a == 3 {
			b = 1
		} else if a == 2 {
			b = 0
		} else {
			b = -1
		}
		b")
	);

	assert_eq!(
		"err",
		ev("a = 2
		b = empty(number)
	if a {
		b = 1
	} else {
		b = 0
	}
	b")
	);
	assert!(has_parsing_error("if true out(4)"));
	assert!(has_parsing_error(
		"if true {
		out(4)
	} else out(5)"
	));
}

#[test]
fn while_statements() {
	assert_eq!(
		"-151",
		ev("i = 0
		while i >= -150 {
			i = i - 1
		}
		i")
	);
	assert_eq!(
		"25",
		ev("a = 0
		i = 0
		while i < 10 {
			temp = a + 1
			a = temp
			if a == 5 {
				a = a * 2
			}
			i =  i+ 1
		}
		a + i")
	);

	assert!(has_parsing_error("while true out(4)"));
}
