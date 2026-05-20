use clap::{Arg, ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;

fn build() -> Command {
	Command::new("help").about("Show help").alias("h").arg(
		Arg::new("command")
			.help("Command to show help for")
			.required(false),
	)
}

fn run(matches: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	let mut cli = crate::cli::build_cli();

	if let Some(target) = matches.get_one::<String>("command") {
		let target = target.as_str();

		if let Some(mut sub) = find_subcommand(&mut cli, target) {
			println!("{}", sub.render_long_help());
			return Ok(());
		}

		anyhow::bail!("Unknown command: {target}");
	}

	println!("{}", cli.render_long_help());
	Ok(())
}

fn find_subcommand(cli: &mut Command, name_or_alias: &str) -> Option<Command> {
	for sub in cli.get_subcommands_mut() {
		if sub.get_name().eq_ignore_ascii_case(name_or_alias) {
			return Some(sub.clone());
		}

		for alias in sub.get_all_aliases() {
			if alias.eq_ignore_ascii_case(name_or_alias) {
				return Some(sub.clone());
			}
		}
	}

	None
}

inventory::submit! {
	CommandSpec {
		name: "help",
		command: build,
		run,
	}
}
