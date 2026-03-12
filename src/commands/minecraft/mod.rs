use std::cmp::Ordering;

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

pub(crate) fn cmp_numeric_dotted(a: &str, b: &str) -> Ordering {
	let pa = a.split('.').collect::<Vec<_>>();
	let pb = b.split('.').collect::<Vec<_>>();
	let max_len = pa.len().max(pb.len());

	for idx in 0..max_len {
		let av = pa.get(idx).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
		let bv = pb.get(idx).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
		match av.cmp(&bv) {
			Ordering::Equal => continue,
			other => return other,
		}
	}

	Ordering::Equal
}
