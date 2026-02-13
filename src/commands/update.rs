use std::path::Path;

use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::net;
use crate::ui;
use crate::util;

const RELEASE_API_URL: &str =
	"https://api.github.com/repos/prodbyeagle/eaglePowerShell/releases/latest";

#[derive(Debug, Deserialize)]
struct GithubRelease {
	assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
	name: String,
	browser_download_url: String,
	digest: Option<String>,
}

fn latest_eagle_asset() -> anyhow::Result<GithubAsset> {
	let release = net::get_json::<GithubRelease>(RELEASE_API_URL)?;
	release
		.assets
		.into_iter()
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
}

fn run(matches: &ArgMatches, ctx: &Context) -> anyhow::Result<()> {
	let force = matches.get_flag("force");
	if is_dev_exe(&ctx.exe_path) && !force {
		anyhow::bail!("Refusing to self-update a dev binary. Use --force.");
	}

	let asset = latest_eagle_asset()?;
	let digest = asset.digest.as_deref().ok_or_else(|| {
		anyhow::anyhow!("Release asset is missing sha256 digest")
	})?;

	let new_path = ctx.exe_dir.join("eagle.new.exe");
	ui::info(&format!(
		"Downloading update: {}",
		asset.browser_download_url
	));
	net::download_to_file_with_sha256(
		&asset.browser_download_url,
		&new_path,
		digest,
	)?;

	let pid = std::process::id();
	let exe_path =
		util::escape_powershell_single_quoted(&ctx.exe_path.to_string_lossy());
	let new_path_s =
		util::escape_powershell_single_quoted(&new_path.to_string_lossy());

	let script = format!(
		"Wait-Process -Id {pid}; Start-Sleep -Milliseconds 200; \
Move-Item -Force '{new_path_s}' '{exe_path}'"
	);

	util::spawn_powershell_hidden(&script)?;

	ui::success("Update scheduled. Re-run eagle in a new shell.");
	Ok(())
}

fn is_dev_exe(path: &Path) -> bool {
	let s = path.to_string_lossy().to_lowercase();
	s.contains("\\target\\debug\\") || s.contains("\\target\\release\\")
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
