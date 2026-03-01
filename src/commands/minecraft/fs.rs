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

#[cfg(test)]
mod tests {
	use std::fs;

	use tempfile::TempDir;

	use super::*;

	fn make_subdir(root: &std::path::Path, name: &str) -> PathBuf {
		let dir = root.join(name);
		fs::create_dir_all(&dir).unwrap();
		dir
	}

	#[test]
	fn empty_root_returns_empty() {
		let tmp = TempDir::new().unwrap();
		let result = find_servers(tmp.path()).unwrap();
		assert!(result.is_empty());
	}

	#[test]
	fn non_existent_root_returns_empty() {
		let result =
			find_servers(std::path::Path::new("/does/not/exist/eagle_test"))
				.unwrap();
		assert!(result.is_empty());
	}

	#[test]
	fn detects_server_by_jar() {
		let tmp = TempDir::new().unwrap();
		let dir = make_subdir(tmp.path(), "survival");
		fs::write(dir.join("server.jar"), b"").unwrap();

		let result = find_servers(tmp.path()).unwrap();
		assert_eq!(result.len(), 1);
		assert_eq!(result[0].file_name().unwrap(), "survival");
	}

	#[test]
	fn detects_server_by_eula_txt() {
		let tmp = TempDir::new().unwrap();
		let dir = make_subdir(tmp.path(), "creative");
		fs::write(dir.join("eula.txt"), b"").unwrap();

		let result = find_servers(tmp.path()).unwrap();
		assert_eq!(result.len(), 1);
	}

	#[test]
	fn detects_server_by_server_properties() {
		let tmp = TempDir::new().unwrap();
		let dir = make_subdir(tmp.path(), "hardcore");
		fs::write(dir.join("server.properties"), b"").unwrap();

		let result = find_servers(tmp.path()).unwrap();
		assert_eq!(result.len(), 1);
	}

	#[test]
	fn ignores_dirs_without_marker_files() {
		let tmp = TempDir::new().unwrap();
		make_subdir(tmp.path(), "not_a_server");

		let result = find_servers(tmp.path()).unwrap();
		assert!(result.is_empty());
	}

	#[test]
	fn results_are_sorted() {
		let tmp = TempDir::new().unwrap();
		for name in ["zebra", "alpha", "middle"] {
			let dir = make_subdir(tmp.path(), name);
			fs::write(dir.join("server.jar"), b"").unwrap();
		}

		let result = find_servers(tmp.path()).unwrap();
		let names: Vec<_> = result
			.iter()
			.map(|p| p.file_name().unwrap().to_str().unwrap())
			.collect();
		assert_eq!(names, ["alpha", "middle", "zebra"]);
	}

	#[test]
	fn dir_guard_removes_dir_on_drop() {
		let tmp = TempDir::new().unwrap();
		let dir = tmp.path().join("transient");
		fs::create_dir_all(&dir).unwrap();

		{
			let _guard = DirGuard::new(dir.clone());
		}

		assert!(!dir.exists());
	}

	#[test]
	fn dir_guard_keeps_dir_when_committed() {
		let tmp = TempDir::new().unwrap();
		let dir = tmp.path().join("kept");
		fs::create_dir_all(&dir).unwrap();

		{
			let mut guard = DirGuard::new(dir.clone());
			guard.commit();
		}

		assert!(dir.exists());
	}
}
