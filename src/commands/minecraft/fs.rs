use std::path::{Path, PathBuf};

pub(super) fn documents_dir() -> Option<PathBuf> {
	directories::UserDirs::new()
		.and_then(|u| u.document_dir().map(|p| p.to_path_buf()))
}

pub(super) fn servers_root() -> anyhow::Result<PathBuf> {
	let root = documents_dir()
		.ok_or_else(|| anyhow::anyhow!("Could not resolve Documents dir"))?
		.join("mc-servers");

	Ok(root)
}

pub(super) fn find_servers(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
	if !root.exists() {
		return Ok(Vec::new());
	}

	let mut out = Vec::new();
	for entry in std::fs::read_dir(root)? {
		let entry = entry?;
		let path = entry.path();
		if !path.is_dir() {
			continue;
		}

		let has_jar = path.join("server.jar").exists();
		let has_config = path.join("server.properties").exists()
			|| path.join("eula.txt").exists();
		if has_jar || has_config {
			out.push(path);
		}
	}

	out.sort();
	Ok(out)
}

pub(super) struct DirGuard {
	path: PathBuf,
	committed: bool,
}

impl DirGuard {
	pub(super) fn new(path: PathBuf) -> Self {
		Self {
			path,
			committed: false,
		}
	}

	pub(super) fn commit(&mut self) {
		self.committed = true;
	}
}

impl Drop for DirGuard {
	fn drop(&mut self) {
		if self.committed {
			return;
		}

		let _ = std::fs::remove_dir_all(&self.path);
	}
}
