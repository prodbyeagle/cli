use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// Runtime information available to all commands.
pub struct Context {
	pub exe_path: PathBuf,
	pub exe_dir: PathBuf,
	pub version: &'static str,
	pub repo_url: &'static str,
	/// Whether the CLI was invoked with the global `--dev` flag.
	pub dev_mode: bool,
}

impl Context {
	/// Constructs a [`Context`] by discovering the current executable path.
	/// Dev mode is enabled automatically for debug builds.
	pub fn new() -> anyhow::Result<Self> {
		let exe_path = std::env::current_exe()?;
		let exe_dir = exe_path
			.parent()
			.map(Path::to_path_buf)
			.unwrap_or_else(|| PathBuf::from("."));

		Ok(Self {
			exe_path,
			exe_dir,
			version: env!("CARGO_PKG_VERSION"),
			repo_url: "https://github.com/prodbyeagle/cli",
			dev_mode: cfg!(debug_assertions),
		})
	}

	/// Returns the version string, appending `-dev` when in dev mode.
	pub fn version_string(&self) -> String {
		if self.dev_mode {
			format!("{}-dev", self.version)
		} else {
			self.version.to_owned()
		}
	}
}
