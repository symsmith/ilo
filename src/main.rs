use dialoguer::{theme::Theme, Input};
use ilo::lexer::Lexer;
use std::{env::args, fmt, fs, process::exit};
use substring::Substring;

fn main() {
	let args: Vec<String> = args().collect();
	let args_len = args.len();

	if args_len > 2 || (args_len == 2 && args[1] == "--help") {
		display_usage();
		exit(64);
	} else if args_len == 2 && {
		let script_len = args[1].chars().count();
		args[1].substring(script_len - 4, script_len) != ".ilo"
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
		Ok(source) => run(source),
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
	fn format_prompt(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
	) -> fmt::Result {
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

		run(input);
	}
}

fn run(source: String) {
	let mut lexer = Lexer::new(source);
	let tokens = lexer.scan_tokens();

	if let Err(()) = tokens {
		return;
	}

	// `tokens` has to be `Ok(...)`
	let tokens = tokens.unwrap();

	for token in tokens {
		println!("{}", token);
	}
}
