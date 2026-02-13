use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;
use crate::util;

fn build() -> Command {
	Command::new("eaglecord")
		.about("Install or update EagleCord (Vencord fork)")
		.alias("e")
		.arg(
			Arg::new("reinstall")
				.long("reinstall")
				.help("Delete the local clone and reinstall")
				.required(false)
				.action(clap::ArgAction::SetTrue),
		)
}

fn build_dev() -> Command {
	Command::new("eaglecord-dev")
		.hide(true)
		.about("EagleCord dev mode (reinstall)")
		.alias("e:dev")
}

fn run(matches: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	let reinstall = matches.get_flag("reinstall");
	run_impl(reinstall)
}

fn run_dev(_: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	run_impl(true)
}

fn run_impl(reinstall: bool) -> anyhow::Result<()> {
	ensure_tool("git")?;
	let bun = ensure_bun()?;

	let repo_url = "https://github.com/prodbyeagle/cord";
	let repo_name = "Vencord";

	let appdata = std::env::var("APPDATA")
		.map_err(|_| anyhow::anyhow!("APPDATA not set"))?;
	let temp_root = PathBuf::from(appdata).join("EagleCord");
	let clone_dir = temp_root.join(repo_name);

	std::fs::create_dir_all(&temp_root)?;

	if reinstall && clone_dir.exists() {
		ui::warning(&format!("Reinstall: removing {}", clone_dir.display()));
		std::fs::remove_dir_all(&clone_dir)?;
	}

	if clone_dir.exists() {
		ensure_repo_clean(&clone_dir)?;
		update_repo(repo_url, &clone_dir)?;
	} else {
		ui::info("Cloning repo...");
		let status = std::process::Command::new("git")
			.arg("clone")
			.arg(repo_url)
			.arg(&clone_dir)
			.stdin(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.status()?;
		if !status.success() {
			anyhow::bail!("git clone failed");
		}
	}

	let dist = clone_dir.join("dist");
	if dist.exists() {
		std::fs::remove_dir_all(dist)?;
	}

	let discord_types = clone_dir.join(r"packages\discord-types");
	if discord_types.exists() {
		ui::info("Linking @vencord/discord-types...");
		run_bun_in_dir(&bun, &discord_types, &["link"])?;
	}

	ui::info("Installing dependencies...");
	run_bun_in_dir(&bun, &clone_dir, &["install"])?;

	ui::info("Building...");
	run_bun_in_dir(&bun, &clone_dir, &["run", "build"])?;

	ui::info("Injecting...");
	run_bun_in_dir(&bun, &clone_dir, &["inject"])?;

	ui::success("EagleCord complete.");
	Ok(())
}

fn ensure_tool(name: &str) -> anyhow::Result<()> {
	which::which(name)
		.map(|_| ())
		.map_err(|_| anyhow::anyhow!("Required tool not found: {name}"))
}

fn ensure_bun() -> anyhow::Result<PathBuf> {
	if let Ok(path) = which::which("bun") {
		return Ok(path);
	}

	if which::which("winget").is_err() {
		anyhow::bail!(
			"bun not found and winget is unavailable. Install Bun manually."
		);
	}

	ui::info("Bun not found. Installing with winget...");
	let install_status = util::run_inherit(
		"winget",
		&[
			"install",
			"--id",
			"Oven-sh.Bun",
			"--exact",
			"--accept-package-agreements",
			"--accept-source-agreements",
			"--disable-interactivity",
		],
	)?;
	if !install_status.success() {
		ui::warning("Bun install did not succeed, trying winget upgrade...");
		let upgrade_status = util::run_inherit(
			"winget",
			&[
				"upgrade",
				"--id",
				"Oven-sh.Bun",
				"--exact",
				"--accept-package-agreements",
				"--accept-source-agreements",
				"--disable-interactivity",
			],
		)?;
		if !upgrade_status.success() {
			anyhow::bail!(
				"Bun install/upgrade failed (install: {install_status}, upgrade: {upgrade_status})"
			);
		}
	}

	if let Ok(path) = which::which("bun") {
		return Ok(path);
	}

	let home = directories::UserDirs::new()
		.map(|u| u.home_dir().to_path_buf())
		.ok_or_else(|| anyhow::anyhow!("Could not resolve home dir"))?;

	let fallback = home.join(r".bun\bin\bun.exe");
	if fallback.exists() {
		return Ok(fallback);
	}

	anyhow::bail!("bun still not found after install")
}

fn ensure_repo_clean(dir: &Path) -> anyhow::Result<()> {
	let dir_s = dir.to_string_lossy().to_string();
	let dirty =
		util::run_capture("git", &["-C", &dir_s, "status", "--porcelain"])?;
	if dirty.trim().is_empty() {
		return Ok(());
	}

	anyhow::bail!(
		"Repo has local changes at {}. Re-run with --reinstall to replace it.",
		dir.display()
	);
}

fn update_repo(repo_url: &str, dir: &Path) -> anyhow::Result<()> {
	let dir_s = dir.to_string_lossy().to_string();
	let local = util::run_capture("git", &["-C", &dir_s, "rev-parse", "HEAD"])?;
	let remote = util::run_capture("git", &["ls-remote", repo_url, "HEAD"])?;

	let remote_hash = remote.split('\t').next().unwrap_or("").trim();
	if local.trim() == remote_hash {
		ui::muted(&format!("Repo is up-to-date ({})", local.trim()));
		return Ok(());
	}

	ui::info("Updating repo...");
	let status = util::run_inherit("git", &["-C", &dir_s, "fetch", "origin"])?;
	if !status.success() {
		anyhow::bail!("git fetch failed");
	}

	let status =
		util::run_inherit("git", &["-C", &dir_s, "pull", "--ff-only"])?;
	if !status.success() {
		anyhow::bail!("git pull --ff-only failed");
	}

	Ok(())
}

fn run_bun_in_dir(bun: &Path, dir: &Path, args: &[&str]) -> anyhow::Result<()> {
	let status = std::process::Command::new(bun)
		.args(args)
		.current_dir(dir)
		.stdin(std::process::Stdio::inherit())
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()?;

	if !status.success() {
		anyhow::bail!("bun {:?} failed: {status}", args);
	}

	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}

inventory::submit! {
	CommandSpec {
		command: build_dev,
		run: run_dev,
	}
}
