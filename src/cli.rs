use clap::{Arg, ArgAction, Command};

use crate::commands;

pub fn build_cli() -> Command {
	let mut cmd = Command::new("eagle")
		.about("eagle - native CLI toolbox")
		.disable_help_subcommand(true)
		.disable_version_flag(true)
		.arg(
			Arg::new("version")
				.short('v')
				.long("version")
				.action(ArgAction::Version)
				.help("Print version"),
		)
		.version(env!("CARGO_PKG_VERSION"))
		.arg_required_else_help(true);

	for spec in commands::iter_specs() {
		cmd = cmd.subcommand((spec.command)());
	}

	cmd
}
