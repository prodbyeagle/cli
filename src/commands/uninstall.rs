use clap::{Arg, ArgMatches, Command};
use dialoguer::Confirm;

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;
use crate::util;

fn build() -> Command {
	Command::new("uninstall")
		.about("Uninstall eagle.exe (Windows)")
		.alias("rem")
		.arg(
			Arg::new("yes")
				.long("yes")
				.short('y')
				.help("Do not prompt")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new("force")
				.long("force")
				.help("Run even if this looks like a dev binary")
				.action(clap::ArgAction::SetTrue),
		)
}

fn run(matches: &ArgMatches, ctx: &Context) -> anyhow::Result<()> {
	let yes = matches.get_flag("yes");
	let force = matches.get_flag("force");

	if is_dev_exe(&ctx.exe_path) && !force {
		anyhow::bail!("Refusing to uninstall a dev binary. Use --force.");
	}

	if !yes {
		let confirmed = Confirm::new()
			.with_prompt("Uninstall eagle?")
			.default(false)
			.interact()?;
		if !confirmed {
			ui::muted("Uninstall canceled.");
			return Ok(());
		}
	}

	let pid = std::process::id();
	let exe_path =
		util::escape_powershell_single_quoted(&ctx.exe_path.to_string_lossy());

	let script = format!(
		"Wait-Process -Id {pid}; \
if (Test-Path '{exe_path}') {{ Remove-Item -Force '{exe_path}' }}; \
if (Test-Path $PROFILE) {{ \
  $c = Get-Content $PROFILE; \
  $c2 = $c | Where-Object {{ $_ -notmatch 'Set-Alias\\s+eagle' }}; \
  Set-Content -Path $PROFILE -Value $c2 \
}}"
	);

	util::spawn_powershell_hidden(&script)?;

	ui::success(
		"Uninstall scheduled. Close this shell if eagle is still in use.",
	);
	Ok(())
}

fn is_dev_exe(path: &std::path::Path) -> bool {
	let s = path.to_string_lossy().to_lowercase();
	s.contains("\\target\\debug\\") || s.contains("\\target\\release\\")
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
