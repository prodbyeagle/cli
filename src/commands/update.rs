use std::path::Path;

use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::net;
use crate::ui;
use crate::util;

const RELEASE_API_URL: &str =
	"https://api.github.com/repos/prodbyeagle/cli/releases/latest";

#[derive(Debug, Deserialize)]
struct GithubRelease {
	tag_name: String,
	assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
	name: String,
	browser_download_url: String,
	digest: Option<String>,
}

fn latest_release() -> anyhow::Result<GithubRelease> {
	net::get_json::<GithubRelease>(RELEASE_API_URL)
}

fn latest_eagle_asset(release: &GithubRelease) -> anyhow::Result<&GithubAsset> {
	release
		.assets
		.iter()
		.find(|asset| asset.name.eq_ignore_ascii_case("eagle.exe"))
		.ok_or_else(|| {
			anyhow::anyhow!("Latest release does not include eagle.exe")
		})
}

fn build() -> Command {
	Command::new("update")
		.about("Update eagle.exe in place (Windows)")
		.alias("u")
		.arg(
			Arg::new("force")
				.long("force")
				.help("Run even if this looks like a dev binary")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new("dev")
				.long("dev")
				.help(
					"Install a local dev build instead of pulling from GitHub.\n\
					 Optionally pass the path to the binary; defaults to\n\
					 .\\target\\debug\\eagle.exe (relative to the current directory).",
				)
				.num_args(0..=1)
				.value_name("PATH")
				.default_missing_value("target/debug/eagle.exe"),
		)
}

fn run(matches: &ArgMatches, ctx: &Context) -> anyhow::Result<()> {
	let force = matches.get_flag("force");

	if let Some(dev_path_str) = matches.get_one::<String>("dev") {
		return run_dev_install(dev_path_str, ctx);
	}

	if is_dev_exe(&ctx.exe_path) && !force {
		anyhow::bail!("Refusing to self-update a dev binary. Use --force.");
	}

	let release = latest_release()?;
	let latest_version = release.tag_name.trim_start_matches('v');

	if latest_version == ctx.version {
		ui::success(&format!("Already up to date (v{})", ctx.version));
		return Ok(());
	}

	let asset = latest_eagle_asset(&release)?;
	let digest = asset.digest.as_deref().ok_or_else(|| {
		anyhow::anyhow!("Release asset is missing sha256 digest")
	})?;

	let new_path = ctx.exe_dir.join("eagle.new.exe");
	ui::info(&format!(
		"Updating eagle v{} → v{latest_version}",
		ctx.version
	));
	net::download_to_file_with_sha256(
		&asset.browser_download_url,
		&new_path,
		digest,
	)?;

	schedule_replace(&new_path, ctx)?;
	ui::success("Update scheduled. Re-run eagle in a new shell.");
	Ok(())
}

fn run_dev_install(dev_path_str: &str, ctx: &Context) -> anyhow::Result<()> {
	let dev_path = {
		let p = std::path::PathBuf::from(dev_path_str);
		if p.is_absolute() {
			p
		} else {
			std::env::current_dir()?.join(p)
		}
	};

	if !dev_path.exists() {
		anyhow::bail!(
			"Dev binary not found: {}\n\
			 Build it first with `cargo build --release`, or pass the path explicitly:\n\
			 eagle update --dev <PATH>",
			dev_path.display()
		);
	}

	let new_path = ctx.exe_dir.join("eagle.new.exe");
	ui::info(&format!(
		"Installing dev build: {} → {}",
		dev_path.display(),
		ctx.exe_path.display()
	));
	std::fs::copy(&dev_path, &new_path)?;

	schedule_replace(&new_path, ctx)?;
	ui::success("Dev build installed. Re-run eagle in a new shell.");
	Ok(())
}

fn schedule_replace(
	new_path: &std::path::Path,
	ctx: &Context,
) -> anyhow::Result<()> {
	let pid = std::process::id();
	let exe_path =
		util::escape_powershell_single_quoted(&ctx.exe_path.to_string_lossy());
	let new_path_s =
		util::escape_powershell_single_quoted(&new_path.to_string_lossy());

	let script = format!(
		"Wait-Process -Id {pid} -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 200; \
Move-Item -Force '{new_path_s}' '{exe_path}'"
	);

	util::spawn_powershell_hidden(&script)
}

#[doc(hidden)]
pub fn is_dev_exe(path: &Path) -> bool {
	let s = path.to_string_lossy().to_lowercase();
	s.contains("\\target\\debug\\") || s.contains("\\target\\release\\")
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
