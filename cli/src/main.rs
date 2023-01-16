use dialoguer::{theme::Theme, Input};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::{env::args, fmt, fs, path::PathBuf, process::exit};

fn main() {
	let args: Vec<String> = args().collect();
	let args_len = args.len();

	if args_len > 2 || (args_len == 2 && args[1] == "--help") {
		display_usage();
		exit(64);
	} else if args_len == 2 && {
		if let Some(ext) = PathBuf::from(&args[1]).extension() {
			ext != "ilo"
		} else {
			true
		}
	} {
		display_command_error("file name must have .ilo extension.".into());
		display_usage();
		exit(64);
	} else if args_len == 2 {
		run_file(&args[1]);
	} else {
		run_repl();
	}
}

fn display_usage() {
	let executable = args().next().unwrap_or("ilo".into());
	println!(
		"Usage:
    Show this help  {} --help
    Run script      {} <file.ilo>
    Run REPL        {}",
		executable, executable, executable
	);
}

fn run_file(path: &String) {
	match fs::read_to_string(path) {
		Ok(source) => {
			if let Err(()) = run(source) {
				exit(70);
			}
		}
		Err(_) => {
			display_command_error(format!("no file found at path {path}"));
			display_usage();
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

fn run_repl() {
	println!("Type exit to stop the REPL.");

	loop {
		let input: String = Input::with_theme(&PromptTheme)
			.with_prompt("ilo> ")
			.allow_empty(true)
			.interact()
			.unwrap_or(String::new());

		if input == "exit" {
			println!("Exiting...");
			break;
		}

		_ = run(input);
	}
}

fn run(source: String) -> Result<(), ()> {
	let mut lexer = Lexer::new(source);
	let tokens = lexer.scan_tokens();

	if let Err(()) = tokens {
		return Err(());
	}

	let tokens = tokens.unwrap();

	let mut parser = Parser::new(tokens);
	let expr = parser.parse();

	if let Err(()) = expr {
		return Err(());
	}

	let expr = expr.unwrap();

	let interpreter = Interpreter::new();
	interpreter.interpret(expr)
}
