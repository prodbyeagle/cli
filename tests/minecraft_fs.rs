use std::fs;
use std::path::PathBuf;

use eagle::commands::minecraft::fs::{DirGuard, find_servers};
use tempfile::TempDir;

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
