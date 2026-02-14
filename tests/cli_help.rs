use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn help_command_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.arg("help").assert().success().stdout(contains("eagle"));
}

#[test]
fn version_command_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.arg("version")
		.assert()
		.success()
		.stdout(contains("eagle"));
}

#[test]
fn minecraft_create_help_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["minecraft", "create", "--help"])
		.assert()
		.success()
		.stdout(contains("--skip-download"));
}

#[test]
fn codex_help_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["help", "codex"])
		.assert()
		.success()
		.stdout(contains("codex --yolo"));
}
