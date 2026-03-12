use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};
use dialoguer::FuzzySelect;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;

use crate::commands::CommandSpec;
use crate::context::Context;

fn build() -> Command {
	Command::new("goto")
		.about("Jump to a development project folder")
		.long_about(
			"Scans your development root for projects and lets you pick one.\n\
			 Prints the selected path to stdout — no other output.\n\n\
			 To make `eagle goto` change your shell directory, add this to\n\
			 your PowerShell profile (~\\Documents\\PowerShell\\profile.ps1):\n\n\
			 \tfunction g { Set-Location (eagle goto $args) }\n\n\
			 Development root resolution order:\n\
			 \t1. --root flag\n\
			 \t2. EAGLE_DEV_ROOT environment variable\n\
			 \t3. ~/Development",
		)
		.alias("g")
		.arg(
			Arg::new("root")
				.long("root")
				.help(
					"Development root (overrides EAGLE_DEV_ROOT and ~/Development)",
				)
				.required(false),
		)
}

fn run(matches: &ArgMatches, _: &Context) -> anyhow::Result<()> {
	let dev_root = resolve_dev_root(matches)?;
	let projects = collect_projects(&dev_root)?;

	if projects.is_empty() {
		anyhow::bail!(
			"No projects found under {}.\n\
			 Expected structure: <root>/.NN/<category>/<project>/\n\
			 Set EAGLE_DEV_ROOT or pass --root to change the root.",
			dev_root.display()
		);
	}

	let labels: Vec<String> = projects
		.iter()
		.map(|(label, _)| {
			let parts: Vec<&str> = label.splitn(3, '/').collect();
			match parts.as_slice() {
				[year, cat, proj] => format!("{year}  ›  {cat}  ›  {proj}"),
				_ => label.clone(),
			}
		})
		.collect();

	let idx = FuzzySelect::with_theme(&ColorfulTheme::default())
		.with_prompt("  goto")
		.items(&labels)
		.interact_on(&Term::stderr())?;

	println!("{}", projects[idx].1.display());
	Ok(())
}

/// Walks `root` and collects all project directories three levels deep:
/// `<root>/<year>/<category>/<project>` where `<year>` matches `.NN`.
///
/// Returns `(label, absolute_path)` pairs sorted newest year first, then alphabetically.
pub fn collect_projects(
	root: &std::path::Path,
) -> anyhow::Result<Vec<(String, PathBuf)>> {
	let mut projects: Vec<(String, PathBuf)> = Vec::new();

	let year_entries = read_dir_sorted(root)?;
	for year_path in year_entries {
		let year_name = dir_name(&year_path);
		if !is_year_dir(&year_name) {
			continue;
		}

		let cat_entries = match read_dir_sorted(&year_path) {
			Ok(entries) => entries,
			Err(_) => continue,
		};

		for cat_path in cat_entries {
			let cat_name = dir_name(&cat_path);

			let proj_entries = match read_dir_sorted(&cat_path) {
				Ok(entries) => entries,
				Err(_) => continue,
			};

			for proj_path in proj_entries {
				let proj_name = dir_name(&proj_path);
				let label = format!("{year_name}/{cat_name}/{proj_name}");
				projects.push((label, proj_path));
			}
		}
	}

	projects.sort_by(|a, b| {
		let a_year = a.0.split('/').next().unwrap_or("");
		let b_year = b.0.split('/').next().unwrap_or("");
		b_year.cmp(a_year).then_with(|| a.0.cmp(&b.0))
	});
	Ok(projects)
}

/// Returns the sorted list of sub-directories directly inside `dir`.
fn read_dir_sorted(dir: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
	let mut entries: Vec<PathBuf> = Vec::new();

	for entry in std::fs::read_dir(dir)? {
		let path = entry?.path();
		if path.is_dir() {
			entries.push(path);
		}
	}

	entries.sort();
	Ok(entries)
}

/// Returns the last component of a path as a `String`, or `""`.
fn dir_name(path: &std::path::Path) -> String {
	path.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or("")
		.to_owned()
}

/// Returns `true` for names like `.25`, `.26` — a dot followed by
/// exactly two ASCII digits.
fn is_year_dir(name: &str) -> bool {
	let Some(rest) = name.strip_prefix('.') else {
		return false;
	};
	rest.len() == 2 && rest.bytes().all(|b| b.is_ascii_digit())
}

fn resolve_dev_root(matches: &ArgMatches) -> anyhow::Result<PathBuf> {
	if let Some(value) = matches.get_one::<String>("root") {
		return Ok(PathBuf::from(value));
	}

	if let Ok(from_env) = std::env::var("EAGLE_DEV_ROOT") {
		let trimmed = from_env.trim().to_owned();
		if trimmed.is_empty() {
			anyhow::bail!("EAGLE_DEV_ROOT is set but empty");
		}
		return Ok(PathBuf::from(trimmed));
	}

	let home = directories::UserDirs::new()
		.map(|u| u.home_dir().to_path_buf())
		.ok_or_else(|| anyhow::anyhow!("Could not resolve home directory"))?;

	Ok(home.join("Development"))
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
