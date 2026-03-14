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
		let sub = (spec.command)();
		if sub.get_name() == sub_name {
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

	let suggestion = cmd
		.get_subcommands()
		.flat_map(|sub| {
			let name = sub.get_name().to_string();
			let aliases: Vec<String> =
				sub.get_all_aliases().map(|a| a.to_string()).collect();
			std::iter::once(name).chain(aliases)
		})
		.filter(|candidate| eagle::util::levenshtein(sub_name, candidate) <= 3)
		.min_by_key(|candidate| eagle::util::levenshtein(sub_name, candidate));

	let msg = match suggestion {
		Some(s) => {
			format!("unknown command: {sub_name}\n\n  Did you mean: {s}?")
		}
		None => format!("unknown command: {sub_name}"),
	};

	Err(cmd.error(ErrorKind::InvalidSubcommand, msg).into())
}
