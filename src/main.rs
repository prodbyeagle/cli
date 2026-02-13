mod cli;
mod commands;
mod context;
mod net;
mod ui;
mod util;

use clap::error::ErrorKind;

use crate::context::Context;

fn main() {
	if let Err(err) = run() {
		crate::ui::error(&format!("{err}"));
		std::process::exit(1);
	}
}

fn run() -> anyhow::Result<()> {
	let ctx = Context::new()?;
	let mut cmd = crate::cli::build_cli();

	let matches = match cmd.clone().try_get_matches() {
		Ok(m) => m,
		Err(err) if err.kind() == ErrorKind::DisplayHelp => {
			err.print()?;
			return Ok(());
		}
		Err(err) if err.kind() == ErrorKind::DisplayVersion => {
			err.print()?;
			return Ok(());
		}
		Err(err) => return Err(err.into()),
	};

	let (sub_name, sub_matches) = matches.subcommand().ok_or_else(|| {
		cmd.error(ErrorKind::MissingSubcommand, "missing command")
	})?;

	for spec in crate::commands::iter_specs() {
		let sub = (spec.command)();
		if sub.get_name() == sub_name {
			return (spec.run)(sub_matches, &ctx);
		}
	}

	let mut cmd2 = crate::cli::build_cli();
	Err(cmd2
		.error(
			ErrorKind::InvalidSubcommand,
			format!("unknown command: {sub_name}"),
		)
		.into())
}
