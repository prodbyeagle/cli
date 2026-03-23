use std::path::Path;

use eagle::commands::update::is_dev_exe;

#[test]
fn dev_debug_path_detected() {
	assert!(is_dev_exe(Path::new("/home/user/project/target/debug/eagle")));
}

#[test]
fn dev_release_path_detected() {
	assert!(is_dev_exe(Path::new(
		"/home/user/project/target/release/eagle"
	)));
}

#[test]
fn installed_path_not_dev() {
	assert!(!is_dev_exe(Path::new("/usr/local/bin/eagle")));
}

#[test]
fn path_is_case_insensitive() {
	assert!(is_dev_exe(Path::new(
		"/home/user/project/TARGET/DEBUG/eagle"
	)));
}

#[test]
fn empty_path_not_dev() {
	assert!(!is_dev_exe(Path::new("")));
}
