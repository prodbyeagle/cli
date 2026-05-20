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

/// Returns the Levenshtein edit distance between two strings.
#[allow(clippy::indexing_slicing)]
// JUSTIFICATION: All indices are mathematically bounded by construction.
// `previous` and `current` are n+1 wide, loops run 1..=m and 1..=n, so
// current[j], previous[j], previous[j-1], current[j-1], a[i-1], and b[j-1]
// are always in range.
pub fn levenshtein(a: &str, b: &str) -> usize {
	let a: Vec<char> = a.chars().collect();
	let b: Vec<char> = b.chars().collect();
	let m = a.len();
	let n = b.len();
	let mut previous: Vec<usize> = (0..=n).collect();
	let mut current = vec![0usize; n + 1];

	for i in 1..=m {
		current[0] = i;
		for j in 1..=n {
			current[j] = if a[i - 1] == b[j - 1] {
				previous[j - 1]
			} else {
				1 + previous[j - 1].min(previous[j]).min(current[j - 1])
			};
		}
		std::mem::swap(&mut previous, &mut current);
	}

	previous[n]
}

/// Escapes a string for use inside a POSIX single-quoted shell string.
/// Single quotes cannot appear inside a single-quoted string, so the string
/// is terminated, a literal `'` is inserted with `'\''`, then reopened.
pub fn escape_sh_single_quoted(value: &str) -> String {
	value.replace('\'', r"'\''")
}

/// Spawns a detached background shell (`sh -c`) process that executes the
/// given command string. stdout/stderr/stdin are all redirected to /dev/null
/// so the process runs silently in the background.
pub fn spawn_shell_background(command: &str) -> anyhow::Result<()> {
	Command::new("sh")
		.args(["-c", command])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.stdin(Stdio::null())
		.spawn()?;
	Ok(())
}
