use clap::Parser as CLIParser;
use dialoguer::{theme::Theme, Input};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::{fmt, fs, path::PathBuf, process::exit};

#[derive(CLIParser)]
struct Args {
	/// Path to the file to run, ending in .ilo. If this is not provided, the REPL will be
	/// executed instead.
	file: Option<String>,
	#[clap(short, long)]
	/// Display the lexed tokens before running the script
	tokens: bool,
	#[clap(short, long)]
	/// Display the parsed Abstract Syntax Tree (AST) before running the script
	ast: bool,
}

fn main() {
	let args = Args::parse();

	if let Some(path) = args.file {
		if let Some(ext) = PathBuf::from(path.clone()).extension() {
			if ext != "ilo" {
				display_command_error("file name must have `.ilo` extension.".to_string());
				exit(64);
			}
			run_file(&path, args.tokens, args.ast);
		} else {
			display_command_error("file name must have `.ilo` extension.".to_string());
			exit(64);
		}
	} else {
		run_repl(args.tokens, args.ast);
	}
}

fn run_file(path: &String, show_tokens: bool, show_ast: bool) {
	match fs::read_to_string(path) {
		Ok(source) => {
			if let Err(()) = run(source, show_tokens, show_ast) {
				exit(70);
			}
		}
		Err(_) => {
			display_command_error(format!("no file found at path `{path}`"));
		}
	}
}

fn display_command_error(description: String) {
	println!("Error: {description}");
}

struct PromptTheme;

impl Theme for PromptTheme {
	fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
		write!(f, "{}", prompt)
	}

	fn format_input_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		default: Option<&str>,
	) -> fmt::Result {
		match default {
			Some(default) if prompt.is_empty() => write!(f, "[{}]", default),
			Some(default) => write!(f, "{} [{}]", prompt, default),
			None => write!(f, "{}", prompt),
		}
	}

	fn format_input_prompt_selection(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		sel: &str,
	) -> fmt::Result {
		write!(f, "{}{}", prompt, sel)
	}
}

fn run_repl(show_tokens: bool, show_ast: bool) {
	println!("Type exit to stop the REPL.");

	loop {
		let input: String = Input::with_theme(&PromptTheme)
			.with_prompt("ilo> ")
			.allow_empty(true)
			.interact()
			.unwrap_or_default();

		if input == "exit" {
			println!("Exiting...");
			break;
		}

		let result = run(input, show_tokens, show_ast);
		if let Ok(result) = result {
			if !result.is_empty() {
				println!("{result}");
			}
		}
	}
}

fn run(source: String, show_tokens: bool, show_ast: bool) -> Result<String, ()> {
	let mut lexer = Lexer::new(source);
	let tokens = lexer.scan_tokens();

	if let Err(()) = tokens {
		return Err(());
	}

	let tokens = tokens.unwrap();

	let separator = "----------------------------------";

	if show_tokens {
		println!("{separator}");
		println!("Tokens:");
		println!("{:#?}", tokens);
		println!("{separator}");
	}

	let mut parser = Parser::new(tokens);
	let expr = parser.parse();

	if let Err(()) = expr {
		return Err(());
	}

	let statements = expr.unwrap();

	if show_ast {
		println!("AST:");
		println!("{:#?}", statements);
		println!("{separator}");
	}

	let mut interpreter = Interpreter::new();

	interpreter.interpret(statements)
}
