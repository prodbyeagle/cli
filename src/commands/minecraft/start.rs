use std::path::Path;

use clap::ArgMatches;
use dialoguer::Select;

use super::fs;
use crate::ui;

pub(super) fn run_start(matches: &ArgMatches) -> anyhow::Result<()> {
	if which::which("java").is_err() {
		anyhow::bail!("java not found in PATH");
	}

	let ram_mb = *matches.get_one::<u32>("ram_mb").unwrap_or(&8192);

	let root = fs::servers_root()?;
	let servers = fs::find_servers(&root)?;
	if servers.is_empty() {
		anyhow::bail!("No servers found in: {}", root.display());
	}

	let items: Vec<String> = servers
		.iter()
		.map(|p| {
			let mut name = p
				.file_name()
				.and_then(|s| s.to_str())
				.unwrap_or("server")
				.to_string();
			if !p.join("server.jar").exists() {
				name.push_str(" (missing server.jar)");
			}
			name
		})
		.collect();

	let selection = Select::new()
		.with_prompt("Select a Minecraft server")
		.items(&items)
		.default(0)
		.interact()?;

	let server_path = &servers[selection];
	let jar_path = server_path.join("server.jar");
	if !jar_path.exists() {
		anyhow::bail!(
			"server.jar not found for '{}'. Recreate without --skip-download or place a jar manually.",
			items[selection]
		);
	}

	crossterm::execute!(
		std::io::stdout(),
		crossterm::terminal::SetTitle(format!(
			"MC-SERVER: {}",
			items[selection]
		))
	)?;

	let java_args = build_java_args(ram_mb, &jar_path);
	let status = std::process::Command::new("java")
		.args(java_args)
		.current_dir(server_path)
		.stdin(std::process::Stdio::inherit())
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()?;

	if !status.success() {
		anyhow::bail!("java exited with: {status}");
	}

	ui::success("Server stopped.");
	Ok(())
}

fn build_java_args(ram_mb: u32, jar_path: &Path) -> Vec<String> {
	let ram = format!("-Xmx{ram_mb}M");
	let ram2 = format!("-Xms{ram_mb}M");

	let args = vec![
		ram,
		ram2,
		"-XX:+UseG1GC".to_string(),
		"-XX:+ParallelRefProcEnabled".to_string(),
		"-XX:MaxGCPauseMillis=200".to_string(),
		"-XX:+UnlockExperimentalVMOptions".to_string(),
		"-XX:+DisableExplicitGC".to_string(),
		"-XX:+AlwaysPreTouch".to_string(),
		"-XX:G1NewSizePercent=30".to_string(),
		"-XX:G1MaxNewSizePercent=40".to_string(),
		"-XX:G1HeapRegionSize=8M".to_string(),
		"-XX:G1ReservePercent=20".to_string(),
		"-XX:G1HeapWastePercent=5".to_string(),
		"-XX:G1MixedGCCountTarget=4".to_string(),
		"-XX:InitiatingHeapOccupancyPercent=15".to_string(),
		"-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
		"-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
		"-XX:SurvivorRatio=32".to_string(),
		"-XX:+PerfDisableSharedMem".to_string(),
		"-XX:MaxTenuringThreshold=1".to_string(),
		"-Daikars.new.flags=true".to_string(),
		"-Dusing.aikars.flags=https://mcutils.com".to_string(),
		"-jar".to_string(),
		jar_path.to_string_lossy().to_string(),
		"nogui".to_string(),
	];

	args
}
