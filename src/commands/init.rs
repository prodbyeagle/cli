use clap::{ArgMatches, Command};

use crate::commands::CommandSpec;
use crate::context::Context;

const GOTO_SNIPPET: &str = r#"
# eagle goto shell integration
g() { local p; p="$(eagle goto "$@")"; [ -n "$p" ] && cd "$p"; }
"#;

fn build() -> Command {
	Command::new("init")
		.about("Install shell integrations (zsh profile)")
		.long_about(
			"Appends the `g` function to your zsh profile so that\n\
			 `g <query>` changes your shell directory via `eagle goto`.\n\n\
			 Profile path: ~/.zshrc",
		)
}

fn run(_: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	let profile_path = zsh_profile()?;

	// Read existing profile (empty string if it doesn't exist yet)
	let existing = if profile_path.exists() {
		std::fs::read_to_string(&profile_path)?
	} else {
		String::new()
	};

	if existing.contains("eagle goto") {
		println!(
			"Shell integration already present in {}",
			profile_path.display()
		);
		return Ok(());
	}

	// Ensure parent directory exists
	if let Some(parent) = profile_path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let mut content = existing;
	content.push_str(GOTO_SNIPPET);
	std::fs::write(&profile_path, content)?;

	println!(
		"Added `g` function to {}\nRestart your shell or run `source ~/.zshrc` to apply.",
		profile_path.display()
	);
	Ok(())
}

fn zsh_profile() -> anyhow::Result<std::path::PathBuf> {
	let home = directories::UserDirs::new()
		.map(|u| u.home_dir().to_path_buf())
		.ok_or_else(|| anyhow::anyhow!("Could not resolve home directory"))?;

	Ok(home.join(".zshrc"))
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
