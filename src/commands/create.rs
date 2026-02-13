use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};
use dialoguer::{Input, Select};

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;
use crate::util;

fn build() -> Command {
	Command::new("create")
		.about("Create a new project from a template")
		.alias("c")
		.arg(
			Arg::new("name")
				.long("name")
				.short('n')
				.help("Project name")
				.required(false),
		)
		.arg(
			Arg::new("template")
				.long("template")
				.short('t')
				.help("Template: discord | next | typescript")
				.required(false),
		)
		.arg(
			Arg::new("root")
				.long("root")
				.help(
					"Base path for projects (defaults to %EAGLE_CREATE_ROOT% or %USERPROFILE%\\Development\\.YY)",
				)
				.required(false),
		)
}

fn run(matches: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	if which::which("git").is_err() {
		anyhow::bail!("git not found in PATH");
	}
	if which::which("bun").is_err() {
		anyhow::bail!("bun not found in PATH");
	}

	let name = match matches.get_one::<String>("name") {
		Some(v) => v.clone(),
		None => prompt_name()?,
	};
	if name.trim().is_empty() {
		anyhow::bail!("Project name must not be empty");
	}

	let template = match matches.get_one::<String>("template") {
		Some(v) => v.to_string(),
		None => select_template()?,
	};

	let template = template.to_lowercase();

	let year = current_two_digit_year()?;
	let base_root = resolve_base_root(matches, &year)?;

	let target_root = match template.as_str() {
		"discord" => base_root.join("discord"),
		"next" => base_root.join("frontend"),
		"typescript" => base_root.join("typescript"),
		_ => anyhow::bail!("Invalid template: {template}"),
	};
	ui::muted(&format!("Target root: {}", target_root.display()));

	std::fs::create_dir_all(&target_root)?;

	let project_path = target_root.join(&name);
	if project_path.exists() {
		anyhow::bail!("Project already exists: {}", project_path.display());
	}

	let repo_url = match template.as_str() {
		"discord" => "https://github.com/meowlounge/discord-template.git",
		"next" => "https://github.com/meowlounge/next-template.git",
		"typescript" => "https://github.com/meowlounge/typescript-template.git",
		_ => unreachable!(),
	};
	ui::info(&format!("Cloning template: {repo_url}"));

	let status = std::process::Command::new("git")
		.arg("clone")
		.arg(repo_url)
		.arg(&project_path)
		.stdin(std::process::Stdio::inherit())
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()?;
	if !status.success() {
		anyhow::bail!("git clone failed");
	}

	let git_dir = project_path.join(".git");
	if git_dir.exists() {
		std::fs::remove_dir_all(git_dir)?;
	}

	ui::info("Updating dependencies with Bun...");
	let status = util::run_inherit_with_dir(
		"bun",
		&["update", "--latest"],
		&project_path,
	)?;
	if !status.success() {
		anyhow::bail!("bun update failed");
	}

	ui::success(&format!("Project created: {}", project_path.display()));
	Ok(())
}

fn resolve_base_root(
	matches: &ArgMatches,
	year: &str,
) -> anyhow::Result<PathBuf> {
	if let Some(value) = matches.get_one::<String>("root") {
		return Ok(PathBuf::from(value));
	}

	if let Ok(from_env) = std::env::var("EAGLE_CREATE_ROOT") {
		let value = from_env.trim();
		if value.is_empty() {
			anyhow::bail!("EAGLE_CREATE_ROOT is set but empty");
		}
		return Ok(PathBuf::from(value));
	}

	let home = directories::UserDirs::new()
		.map(|u| u.home_dir().to_path_buf())
		.ok_or_else(|| {
			anyhow::anyhow!("Could not resolve user home directory")
		})?;

	Ok(home.join("Development").join(format!(".{year}")))
}

fn prompt_name() -> anyhow::Result<String> {
	Input::<String>::new()
		.with_prompt("Enter project name")
		.interact_text()
		.map_err(|err| anyhow::anyhow!("Failed to read project name: {err}"))
}

fn select_template() -> anyhow::Result<String> {
	let options = ["discord", "next", "typescript"];
	let selection = Select::new()
		.with_prompt("Choose a template")
		.items(&options)
		.default(0)
		.interact()
		.map_err(|err| anyhow::anyhow!("Failed to select template: {err}"))?;
	Ok(options[selection].to_string())
}

fn current_two_digit_year() -> anyhow::Result<String> {
	let now = time::OffsetDateTime::now_local()
		.unwrap_or_else(|_| time::OffsetDateTime::now_utc());
	let year = now.year() % 100;
	Ok(format!("{year:02}"))
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
