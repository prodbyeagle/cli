use std::process::{Command, ExitStatus, Stdio};

/// Runs a command inheriting stdin/stdout/stderr.
pub fn run_inherit(program: &str, args: &[&str]) -> anyhow::Result<ExitStatus> {
	let mut cmd = Command::new(program);
	cmd.args(args)
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit());

	Ok(cmd.status()?)
}

/// Runs a command inheriting stdin/stdout/stderr from a specific working dir.
pub fn run_inherit_with_dir(
	program: &str,
	args: &[&str],
	current_dir: &std::path::Path,
) -> anyhow::Result<ExitStatus> {
	let mut cmd = Command::new(program);
	cmd.current_dir(current_dir)
		.args(args)
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit());

	Ok(cmd.status()?)
}

/// Runs a command and returns stdout as UTF-8. If it fails, includes stderr in
/// the error message.
pub fn run_capture(program: &str, args: &[&str]) -> anyhow::Result<String> {
	let out = Command::new(program).args(args).output()?;
	if !out.status.success() {
		let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
		anyhow::bail!(
			"command failed: {} {} ({})",
			program,
			args.join(" "),
			stderr
		);
	}
	Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Escapes a string for use inside a single-quoted PowerShell string literal.
pub fn escape_powershell_single_quoted(value: &str) -> String {
	value.replace('\'', "''")
}

/// Spawns a hidden PowerShell process that executes the given command string.
pub fn spawn_powershell_hidden(command: &str) -> anyhow::Result<()> {
	Command::new("powershell")
		.args([
			"-NoProfile",
			"-ExecutionPolicy",
			"Bypass",
			"-WindowStyle",
			"Hidden",
			"-Command",
			command,
		])
		.spawn()?;

	Ok(())
}
