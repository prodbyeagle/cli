use std::path::Path;

use clap::{Arg, ArgMatches, Command};
use dialoguer::{Input, Select};

use super::fabric;
use super::fs;
use super::paper;
use crate::ui;

pub(super) fn build_command() -> Command {
	Command::new("create")
		.about("Create a new Minecraft server")
		.arg(
			Arg::new("name")
				.long("name")
				.short('n')
				.help("Server name (folder name)")
				.required(false),
		)
		.arg(
			Arg::new("type")
				.long("type")
				.short('t')
				.help("Server type: paper | fabric")
				.value_parser(["paper", "fabric"])
				.required(false),
		)
		.arg(
			Arg::new("version")
				.long("version")
				.short('v')
				.help("Minecraft version (e.g. 1.21.11 or 1.21)")
				.required(false),
		)
		.arg(
			Arg::new("port")
				.long("port")
				.help("Server port")
				.value_parser(clap::value_parser!(u16))
				.default_value("22222"),
		)
		.arg(
			Arg::new("motd")
				.long("motd")
				.help("Server motd")
				.default_value("eagle minecraft server"),
		)
		.arg(
			Arg::new("force")
				.long("force")
				.help("Overwrite if the folder already exists")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new("skip_download")
				.long("skip-download")
				.help("Only create config files (no jar download)")
				.action(clap::ArgAction::SetTrue),
		)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerType {
	Paper,
	Fabric,
}

impl ServerType {
	fn as_str(self) -> &'static str {
		match self {
			Self::Paper => "paper",
			Self::Fabric => "fabric",
		}
	}
}

pub(super) fn run_create(matches: &ArgMatches) -> anyhow::Result<()> {
	let name = matches
		.get_one::<String>("name")
		.map(|s| s.to_string())
		.unwrap_or_else(prompt_server_name);

	validate_server_name(&name)?;

	let server_type = matches
		.get_one::<String>("type")
		.map(|s| s.as_str())
		.map(parse_server_type)
		.transpose()?
		.unwrap_or_else(select_server_type);

	let version_input = matches
		.get_one::<String>("version")
		.map(|s| s.to_string())
		.unwrap_or_else(prompt_version);

	let version = match server_type {
		ServerType::Paper => paper::resolve_paper_version(&version_input)?,
		ServerType::Fabric => version_input.clone(),
	};

	let port = *matches.get_one::<u16>("port").unwrap_or(&22222);
	let motd = matches
		.get_one::<String>("motd")
		.map(|s| s.to_string())
		.unwrap_or_else(|| "eagle minecraft server".to_string());

	let force = matches.get_flag("force");
	let skip_download = matches.get_flag("skip_download");

	let root = fs::servers_root()?;
	std::fs::create_dir_all(&root)?;

	let server_dir = root.join(&name);
	if server_dir.exists() {
		if !force {
			anyhow::bail!(
				"Folder already exists: {} (use --force)",
				server_dir.display()
			);
		}
		std::fs::remove_dir_all(&server_dir)?;
	}

	std::fs::create_dir_all(&server_dir)?;
	let mut guard = fs::DirGuard::new(server_dir.clone());

	write_eula(&server_dir)?;
	write_server_properties(&server_dir, port, &motd)?;

	if !skip_download {
		let jar_path = server_dir.join("server.jar");
		match server_type {
			ServerType::Paper => {
				paper::download_paper_server(&version, &jar_path)?
			}
			ServerType::Fabric => {
				fabric::download_fabric_server(&version, &jar_path)?
			}
		}
	} else {
		ui::warning(
			"Skipping jar download. This server will not start until server.jar exists.",
		);
	}

	ui::success(&format!(
		"Created server: {} ({}, {})",
		server_dir.display(),
		server_type.as_str(),
		format_version_label(&version_input, &version),
	));
	ui::muted(&format!("Port: {port}"));
	ui::muted(&format!("Motd: {motd}"));

	guard.commit();
	Ok(())
}

fn format_version_label(input: &str, resolved: &str) -> String {
	if input == resolved {
		input.to_string()
	} else {
		format!("{input} -> {resolved}")
	}
}

fn prompt_server_name() -> String {
	Input::<String>::new()
		.with_prompt("Server name")
		.interact_text()
		.unwrap_or_else(|_| "mc-server".to_string())
}

fn prompt_version() -> String {
	Input::<String>::new()
		.with_prompt("Minecraft version (e.g. 1.21.11 or 1.21)")
		.interact_text()
		.unwrap_or_else(|_| "1.21.11".to_string())
}

fn select_server_type() -> ServerType {
	let options = ["paper", "fabric"];
	let selection = Select::new()
		.with_prompt("Server type")
		.items(&options)
		.default(0)
		.interact()
		.unwrap_or(0);

	if options[selection] == "fabric" {
		ServerType::Fabric
	} else {
		ServerType::Paper
	}
}

fn parse_server_type(s: &str) -> anyhow::Result<ServerType> {
	match s.to_lowercase().as_str() {
		"paper" => Ok(ServerType::Paper),
		"fabric" => Ok(ServerType::Fabric),
		_ => anyhow::bail!("Invalid type: {s} (expected: paper | fabric)"),
	}
}

fn validate_server_name(name: &str) -> anyhow::Result<()> {
	if name.trim().is_empty() {
		anyhow::bail!("Name must not be empty");
	}

	let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
	if name.chars().any(|c| invalid.contains(&c)) {
		anyhow::bail!(
			"Invalid name. Windows folder names cannot contain: <>:\"/\\|?*"
		);
	}

	if name.contains("..") {
		anyhow::bail!("Invalid name: '..' not allowed");
	}

	Ok(())
}

fn write_eula(server_dir: &Path) -> anyhow::Result<()> {
	let content = "# By changing the setting below to TRUE you are indicating your\n# agreement to our EULA (https://aka.ms/MinecraftEULA).\n"
			.to_string()
		+ "eula=true\n";

	std::fs::write(server_dir.join("eula.txt"), content)?;
	Ok(())
}

fn write_server_properties(
	server_dir: &Path,
	port: u16,
	motd: &str,
) -> anyhow::Result<()> {
	let mut lines = Vec::new();
	lines.push("enable-jmx-monitoring=false".to_string());
	lines.push(format!("server-port={port}"));
	lines.push("server-ip=".to_string());
	lines.push(format!("motd={motd}"));
	lines.push("enable-command-block=false".to_string());
	lines.push("online-mode=true".to_string());
	lines.push("level-name=world".to_string());
	lines.push("gamemode=survival".to_string());
	lines.push("difficulty=easy".to_string());
	lines.push("max-players=20".to_string());
	lines.push("view-distance=10".to_string());
	lines.push("simulation-distance=10".to_string());
	lines.push("spawn-protection=16".to_string());
	lines.push("sync-chunk-writes=true".to_string());
	lines.push("enable-rcon=false".to_string());
	lines.push("enable-query=false".to_string());
	lines.push("enforce-secure-profile=true".to_string());
	lines.push("white-list=false".to_string());
	lines.push("pvp=true".to_string());
	lines.push("allow-flight=false".to_string());
	lines.push("generate-structures=true".to_string());
	lines.push("level-seed=".to_string());
	lines.push("allow-nether=true".to_string());
	lines.push("spawn-animals=true".to_string());
	lines.push("spawn-monsters=true".to_string());
	lines.push("spawn-npcs=true".to_string());
	lines.push("use-native-transport=true".to_string());

	std::fs::write(
		server_dir.join("server.properties"),
		format!("{}\n", lines.join("\n")),
	)?;

	Ok(())
}
