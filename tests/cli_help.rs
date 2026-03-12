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
fn create_help_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["create", "--help"])
		.assert()
		.success()
		.stdout(contains("--template"));
}

#[test]
fn update_help_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["update", "--help"])
		.assert()
		.success()
		.stdout(contains("--force"));
}

#[test]
fn minecraft_help_shows_ram_flag() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.args(["minecraft", "--help"])
		.assert()
		.success()
		.stdout(contains("--ram-mb"));
}

#[test]
fn help_flag_succeeds() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.arg("--help")
		.assert()
		.success()
		.stdout(contains("eagle"));
}

#[test]
fn unknown_subcommand_fails() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.arg("notacommand").assert().failure();
}

#[test]
fn version_output_contains_version_number() {
	let mut cmd = cargo_bin_cmd!("eagle");
	cmd.arg("version")
		.assert()
		.success()
		.stdout(contains(env!("CARGO_PKG_VERSION")));
}
