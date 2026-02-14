use clap::{ArgMatches, Command};

use crate::context::Context;

pub struct CommandSpec {
	pub command: fn() -> Command,
	pub run: fn(&ArgMatches, &Context) -> anyhow::Result<()>,
}

inventory::collect!(CommandSpec);

pub fn iter_specs() -> inventory::iter<CommandSpec> {
	inventory::iter::<CommandSpec>
}

pub mod codex;
pub mod create;
pub mod eaglecord;
pub mod help;
pub mod minecraft;
pub mod spicetify;
pub mod uninstall;
pub mod update;
pub mod version;
