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
		println!("{} {}", "→".cyan(), message.cyan());
	} else {
		println!("{message}");
	}
}

pub fn success(message: &str) {
	if stdout_colors() {
		println!("{} {}", "✓".green(), message.green());
	} else {
		println!("{message}");
	}
}

pub fn warning(message: &str) {
	if stdout_colors() {
		println!("{} {}", "!".yellow(), message.yellow());
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
		eprintln!("{} {}", "✗".red().bold(), message.red().bold());
	} else {
		eprintln!("{message}");
	}
}

pub fn debug(message: &str) {
	if stderr_colors() {
		eprintln!("{} {}", "dbg".dark_grey(), message.dark_grey());
	} else {
		eprintln!("[dbg] {message}");
	}
}
