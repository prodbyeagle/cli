use clap::error::ErrorKind;
use eagle::context::Context;

fn main() {
	if let Err(err) = run() {
		eagle::ui::error(&format!("{err}"));
		std::process::exit(1);
	}
}

fn run() -> anyhow::Result<()> {
	let mut cmd = eagle::cli::build_cli();

	let matches = match cmd.clone().try_get_matches() {
		Ok(m) => m,
		Err(err) if err.kind() == ErrorKind::DisplayHelp => {
			err.print()?;
			return Ok(());
		}
		Err(err) if err.kind() == ErrorKind::DisplayVersion => {
			err.print()?;
			return Ok(());
		}
		Err(err) => return Err(err.into()),
	};

	let ctx = Context::new()?;

	if ctx.dev_mode {
		eagle::ui::debug(&format!("eagle v{}", ctx.version_string()));
		eagle::ui::debug(&format!("exe: {}", ctx.exe_path.display()));
	}

	let (sub_name, sub_matches) = matches.subcommand().ok_or_else(|| {
		cmd.error(ErrorKind::MissingSubcommand, "missing command")
	})?;

	if ctx.dev_mode {
		eagle::ui::debug(&format!("dispatch → {sub_name}"));
	}

	for spec in eagle::commands::iter_specs() {
		if spec.name == sub_name {
			let t0 = std::time::Instant::now();
			let result = (spec.run)(sub_matches, &ctx);
			if ctx.dev_mode {
				eagle::ui::debug(&format!(
					"finished in {:.1}ms",
					t0.elapsed().as_secs_f64() * 1000.0
				));
			}
			return result;
		}
	}

	let suggestion = closest_command_name(&cmd, sub_name);

	let msg = match suggestion {
		Some(s) => {
			format!("unknown command: {sub_name}\n\n  Did you mean: {s}?")
		}
		None => format!("unknown command: {sub_name}"),
	};

	Err(cmd.error(ErrorKind::InvalidSubcommand, msg).into())
}

fn closest_command_name<'a>(
	cmd: &'a clap::Command,
	sub_name: &str,
) -> Option<&'a str> {
	let mut closest = None;
	let mut closest_distance = usize::MAX;

	for sub in cmd.get_subcommands() {
		track_candidate(
			sub_name,
			sub.get_name(),
			&mut closest,
			&mut closest_distance,
		);
		for alias in sub.get_all_aliases() {
			track_candidate(
				sub_name,
				alias,
				&mut closest,
				&mut closest_distance,
			);
		}
	}

	if closest_distance <= 3 { closest } else { None }
}

fn track_candidate<'a>(
	sub_name: &str,
	candidate: &'a str,
	closest: &mut Option<&'a str>,
	closest_distance: &mut usize,
) {
	let distance = eagle::util::levenshtein(sub_name, candidate);
	if distance < *closest_distance {
		*closest = Some(candidate);
		*closest_distance = distance;
	}
}
