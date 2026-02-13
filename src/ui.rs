use std::io::IsTerminal;

use crossterm::style::Stylize;

fn stdout_colors() -> bool {
	std::io::stdout().is_terminal()
}

fn stderr_colors() -> bool {
	std::io::stderr().is_terminal()
}

pub fn info(message: &str) {
	if stdout_colors() {
		println!("{}", message.cyan());
	} else {
		println!("{message}");
	}
}

pub fn success(message: &str) {
	if stdout_colors() {
		println!("{}", message.green());
	} else {
		println!("{message}");
	}
}

pub fn warning(message: &str) {
	if stdout_colors() {
		println!("{}", message.yellow());
	} else {
		println!("{message}");
	}
}

pub fn muted(message: &str) {
	if stdout_colors() {
		println!("{}", message.dark_grey());
	} else {
		println!("{message}");
	}
}

pub fn error(message: &str) {
	if stderr_colors() {
		eprintln!("{}", message.red().bold());
	} else {
		eprintln!("{message}");
	}
}
