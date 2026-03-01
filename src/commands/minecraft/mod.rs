use clap::{Arg, ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;

mod create;
pub mod fabric;
pub mod fs;
pub mod paper;
pub mod start;

fn build() -> Command {
	Command::new("minecraft")
		.about("Minecraft server tools (start, create)")
		.alias("m")
		.arg(
			Arg::new("ram_mb")
				.long("ram-mb")
				.help("RAM in MB")
				.value_parser(clap::value_parser!(u32))
				.required(false),
		)
		.subcommand(create::build_command())
}

fn run(matches: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	match matches.subcommand() {
		Some(("create", sub)) => create::run_create(sub),
		Some((other, _)) => anyhow::bail!("Unknown subcommand: {other}"),
		None => start::run_start(matches),
	}
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
