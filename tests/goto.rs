use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use tempfile::TempDir;

/// Build a fake dev root:
///   <root>/apps/my-app
///   <root>/frontend/my-site
///   <root>/discord/my-bot
///   <root>/empty/           (category with no projects — skipped)
fn make_dev_root() -> TempDir {
	let root = tempfile::tempdir().expect("tempdir");
	let r = root.path();
	fs::create_dir_all(r.join("apps/my-app")).unwrap();
	fs::create_dir_all(r.join("frontend/my-site")).unwrap();
	fs::create_dir_all(r.join("discord/my-bot")).unwrap();
	fs::create_dir_all(r.join("empty")).unwrap();
	root
}

#[test]
fn goto_help_shows_set_location_hint() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["goto", "--help"])
		.assert()
		.success()
		.stdout(contains("Set-Location"));
}

#[test]
fn goto_exits_nonzero_when_no_projects_found() {
	let empty = tempfile::tempdir().expect("tempdir");
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["goto", "--root", empty.path().to_str().unwrap()])
		.assert()
		.failure()
		.stderr(contains("No projects found"));
}

#[test]
fn goto_exits_nonzero_for_missing_root() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["goto", "--root", "/nonexistent/path/xyz"])
		.assert()
		.failure();
}

/// Verify that `collect_projects` finds exactly the three leaf dirs
/// and skips the empty category.
#[test]
fn collect_projects_finds_correct_dirs() {
	let root = make_dev_root();
	let projects = eagle::commands::goto::collect_projects(root.path())
		.expect("collect_projects");

	let labels: Vec<&str> = projects.iter().map(|(l, _)| l.as_str()).collect();

	assert_eq!(labels.len(), 3, "expected 3 projects, got {labels:?}");
	assert!(labels.contains(&"apps/my-app"));
	assert!(labels.contains(&"discord/my-bot"));
	assert!(labels.contains(&"frontend/my-site"));
}

#[test]
fn collect_projects_are_sorted() {
	let root = make_dev_root();
	let projects = eagle::commands::goto::collect_projects(root.path())
		.expect("collect_projects");
	let labels: Vec<&str> = projects.iter().map(|(l, _)| l.as_str()).collect();
	// Alphabetical order.
	assert_eq!(
		labels,
		vec!["apps/my-app", "discord/my-bot", "frontend/my-site"]
	);
}
