use std::path::Path;

use eagle::commands::update::is_dev_exe;

#[test]
fn dev_debug_path_detected() {
	assert!(is_dev_exe(Path::new(
		"C:\\project\\target\\debug\\eagle.exe"
	)));
}

#[test]
fn dev_release_path_detected() {
	assert!(is_dev_exe(Path::new(
		"C:\\project\\target\\release\\eagle.exe"
	)));
}

#[test]
fn installed_path_not_dev() {
	assert!(!is_dev_exe(Path::new("C:\\Users\\user\\bin\\eagle.exe")));
}

#[test]
fn path_is_case_insensitive() {
	assert!(is_dev_exe(Path::new(
		"C:\\project\\TARGET\\DEBUG\\eagle.exe"
	)));
}

#[test]
fn empty_path_not_dev() {
	assert!(!is_dev_exe(Path::new("")));
}
