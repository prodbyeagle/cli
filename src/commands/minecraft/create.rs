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
	let motd = matches.get_one::<String>("motd").cloned().unwrap();

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

	if selection == 1 {
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
	std::fs::write(
		server_dir.join("eula.txt"),
		"# By changing the setting below to TRUE you are indicating your\n\
		 # agreement to our EULA (https://aka.ms/MinecraftEULA).\n\
		 eula=true\n",
	)?;
	Ok(())
}

fn write_server_properties(
	server_dir: &Path,
	port: u16,
	motd: &str,
) -> anyhow::Result<()> {
	let content = format!(
		"enable-jmx-monitoring=false\n\
		 server-port={port}\n\
		 server-ip=\n\
		 motd={motd}\n\
		 enable-command-block=false\n\
		 online-mode=true\n\
		 level-name=world\n\
		 gamemode=survival\n\
		 difficulty=easy\n\
		 max-players=5\n\
		 view-distance=32\n\
		 simulation-distance=10\n\
		 spawn-protection=16\n\
		 sync-chunk-writes=true\n\
		 enable-rcon=false\n\
		 enable-query=false\n\
		 enforce-secure-profile=true\n\
		 white-list=false\n\
		 pvp=true\n\
		 allow-flight=false\n\
		 generate-structures=true\n\
		 level-seed=\n\
		 allow-nether=true\n\
		 spawn-animals=true\n\
		 spawn-monsters=true\n\
		 spawn-npcs=true\n\
		 use-native-transport=true\n"
	);

	std::fs::write(server_dir.join("server.properties"), content)?;

	Ok(())
}
