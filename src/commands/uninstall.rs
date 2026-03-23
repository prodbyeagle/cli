use clap::{Arg, ArgMatches, Command};
use dialoguer::Confirm;

use crate::commands::CommandSpec;
use crate::context::Context;
use crate::ui;
use crate::util;

fn build() -> Command {
	Command::new("uninstall")
		.about("Uninstall eagle (macOS)")
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

	if crate::commands::update::is_dev_exe(&ctx.exe_path) && !force {
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
		util::escape_sh_single_quoted(&ctx.exe_path.to_string_lossy());

	// Wait for the current process to exit, then remove the binary and strip
	// the shell integration from ~/.zshrc.
	let script = format!(
		"while kill -0 {pid} 2>/dev/null; do sleep 0.1; done; \
rm -f '{exe_path}'; \
if [ -f ~/.zshrc ]; then grep -v 'eagle goto' ~/.zshrc > /tmp/eagle_zshrc_tmp && mv /tmp/eagle_zshrc_tmp ~/.zshrc; fi"
	);

	util::spawn_shell_background(&script)?;

	ui::success(
		"Uninstall scheduled. Close this shell if eagle is still in use.",
	);
	Ok(())
}

inventory::submit! {
	CommandSpec {
		command: build,
		run,
	}
}
