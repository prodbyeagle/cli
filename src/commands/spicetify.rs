use clap::{ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;
use crate::util;

fn build() -> Command {
	Command::new("spicetify")
		.about("Install or update Spicetify")
		.alias("s")
}

fn run(_: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	if which::which("winget").is_err() {
		anyhow::bail!("winget not found. Install/upgrade Spicetify manually.");
	}

	ui::info("Installing Spicetify with winget...");
	let install_status = util::run_inherit(
		"winget",
		&[
			"install",
			"--id",
			"Spicetify.Spicetify",
			"--exact",
			"--accept-package-agreements",
			"--accept-source-agreements",
			"--disable-interactivity",
		],
	)?;
	if install_status.success() {
		ui::success("Spicetify installed.");
		return Ok(());
	}

	ui::warning("Install did not succeed, trying winget upgrade...");
	let upgrade_status = util::run_inherit(
		"winget",
		&[
			"upgrade",
			"--id",
			"Spicetify.Spicetify",
			"--exact",
			"--accept-package-agreements",
			"--accept-source-agreements",
			"--disable-interactivity",
		],
	)?;
	if !upgrade_status.success() {
		anyhow::bail!(
			"Spicetify install/upgrade failed (install: {install_status}, upgrade: {upgrade_status})"
		);
	}

	ui::success("Spicetify updated.");
	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
