use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// Runtime information available to all commands.
pub struct Context {
	pub exe_path: PathBuf,
	pub exe_dir: PathBuf,
	pub version: &'static str,
	pub repo_url: &'static str,
}

impl Context {
	/// Constructs a [`Context`] by discovering the current executable path.
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
		})
	}
}
