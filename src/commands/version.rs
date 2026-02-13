use clap::{ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;

fn build() -> Command {
	Command::new("version")
		.about("Show the current version")
		.alias("v")
}

fn run(_: &ArgMatches, ctx: &Context) -> anyhow::Result<()> {
	ui::success(&format!("eagle {}", ctx.version));
	ui::muted(ctx.repo_url);
	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
