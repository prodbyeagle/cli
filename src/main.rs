use clap::error::ErrorKind;
use eagle::context::Context;

fn main() {
	if let Err(err) = run() {
		eagle::ui::error(&format!("{err}"));
		std::process::exit(1);
	}
}

fn run() -> anyhow::Result<()> {
	let ctx = Context::new()?;
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

	let (sub_name, sub_matches) = matches.subcommand().ok_or_else(|| {
		cmd.error(ErrorKind::MissingSubcommand, "missing command")
	})?;

	for spec in eagle::commands::iter_specs() {
		let sub = (spec.command)();
		if sub.get_name() == sub_name {
			return (spec.run)(sub_matches, &ctx);
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
