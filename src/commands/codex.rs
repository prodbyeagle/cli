use std::process::Stdio;

use clap::{ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;

const CODEX_WORKDIR: &str = r"D:\development\.26\eagle";

fn build() -> Command {
	Command::new("codex").about(
		"Launch codex --yolo in a PowerShell terminal at D:\\development\\.26\\eagle",
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
		std::process::Command::new("wt")
			.args([
				"-d",
				CODEX_WORKDIR,
				"powershell",
				"-NoExit",
				"-Command",
				"codex --yolo",
			])
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;
		ui::success("Opened Windows Terminal (PowerShell) with: codex --yolo");
		return Ok(());
	}

	ui::warning("Windows Terminal (wt) not found, running in current shell.");
	let status = std::process::Command::new("codex")
		.current_dir(workdir)
		.arg("--yolo")
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()?;

	if !status.success() {
		anyhow::bail!("codex --yolo failed: {status}");
	}

	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
