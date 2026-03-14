#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Command, ExitStatus, Stdio};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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

/// Returns the Levenshtein edit distance between two strings.
#[allow(clippy::indexing_slicing)]
// JUSTIFICATION: All indices are mathematically bounded by construction.
// `dp` is (m+1)×(n+1), loops run 1..=m and 1..=n, so dp[i][j], dp[i-1][j-1],
// dp[i-1][j], dp[i][j-1] are always in range. `a[i-1]` and `b[j-1]` are safe
// because i ∈ 1..=m=a.len() and j ∈ 1..=n=b.len().
pub fn levenshtein(a: &str, b: &str) -> usize {
	let a: Vec<char> = a.chars().collect();
	let b: Vec<char> = b.chars().collect();
	let m = a.len();
	let n = b.len();
	let mut dp = vec![vec![0usize; n + 1]; m + 1];
	for (i, row) in dp.iter_mut().enumerate() {
		row[0] = i;
	}
	for (j, cell) in dp[0].iter_mut().enumerate() {
		*cell = j;
	}
	for i in 1..=m {
		for j in 1..=n {
			dp[i][j] = if a[i - 1] == b[j - 1] {
				dp[i - 1][j - 1]
			} else {
				1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1])
			};
		}
	}
	dp[m][n]
}

/// Escapes a string for use inside a single-quoted PowerShell string literal.
pub fn escape_powershell_single_quoted(value: &str) -> String {
	value.replace('\'', "''")
}

/// Spawns a hidden PowerShell process that executes the given command string.
///
/// Uses `CREATE_NO_WINDOW` on Windows so no console window is created and
/// the current terminal's focus is never disturbed.
pub fn spawn_powershell_hidden(command: &str) -> anyhow::Result<()> {
	let mut cmd = Command::new("powershell");
	cmd.args([
		"-NoProfile",
		"-NonInteractive",
		"-ExecutionPolicy",
		"Bypass",
		"-Command",
		command,
	])
	.stdout(Stdio::null())
	.stderr(Stdio::null());

	#[cfg(target_os = "windows")]
	cmd.creation_flags(CREATE_NO_WINDOW);

	cmd.spawn()?;
	Ok(())
}
