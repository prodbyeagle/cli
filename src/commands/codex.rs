use std::process::Stdio;

use clap::{ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;

const CODEX_WORKDIR: &str = r"D:\development\.26\eagle";

fn build() -> Command {
	Command::new("codex").about(
		"Launch codex --yolo in a PowerShell 7 terminal at D:\\development\\.26\\eagle",
	)
}

fn run(_: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	if which::which("codex").is_err() {
		anyhow::bail!("codex not found in PATH");
	}

	let workdir = std::path::Path::new(CODEX_WORKDIR);
	if !workdir.exists() {
		anyhow::bail!("Codex workspace not found: {}", workdir.display());
	}

	if which::which("wt").is_ok() {
		if which::which("pwsh").is_err() {
			anyhow::bail!("pwsh (PowerShell 7) not found in PATH");
		}

		std::process::Command::new("wt")
			.args([
				"-d",
				CODEX_WORKDIR,
				"pwsh",
				"-NoExit",
				"-Command",
				"codex --yolo",
			])
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;
		ui::success(
			"Opened Windows Terminal (PowerShell 7) with: codex --yolo",
		);
		return Ok(());
	}

	if which::which("pwsh").is_err() {
		anyhow::bail!("pwsh (PowerShell 7) not found in PATH");
	}

	ui::warning(
		"Windows Terminal (wt) not found, launching PowerShell 7 here.",
	);
	let status = std::process::Command::new("pwsh")
		.current_dir(workdir)
		.args(["-NoExit", "-Command", "codex --yolo"])
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()?;

	if !status.success() {
		anyhow::bail!("pwsh codex --yolo failed: {status}");
	}

	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
