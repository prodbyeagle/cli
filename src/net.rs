//! Small networking helpers used across commands.
//!
//! This module intentionally stays minimal:
//! - blocking IO (fits the CLI model)
//! - no global mutable client state
//! - retries with bounded backoff for transient failures

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::ui;

const USER_AGENT: &str = concat!("eagle/", env!("CARGO_PKG_VERSION"));
const MAX_HTTP_ATTEMPTS: usize = 3;

fn http_agent() -> &'static ureq::Agent {
	static AGENT: OnceLock<ureq::Agent> = OnceLock::new();

	AGENT.get_or_init(|| {
		let config = ureq::Agent::config_builder()
			.timeout_connect(Some(Duration::from_secs(10)))
			.timeout_global(Some(Duration::from_secs(60)))
			.timeout_recv_response(Some(Duration::from_secs(30)))
			.timeout_recv_body(Some(Duration::from_secs(120)))
			.build();

		config.into()
	})
}

fn request_get(
	url: &str,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
	http_agent()
		.get(url)
		.header("User-Agent", USER_AGENT)
		.call()
}

fn is_retryable_http_error(err: &ureq::Error) -> bool {
	match err {
		ureq::Error::StatusCode(code) => {
			*code == 408 || *code == 429 || (500..=599).contains(code)
		}
		ureq::Error::Timeout(_)
		| ureq::Error::Io(_)
		| ureq::Error::HostNotFound
		| ureq::Error::ConnectionFailed => true,
		_ => false,
	}
}

fn retry_delay(attempt: usize) -> Duration {
	match attempt {
		1 => Duration::from_millis(350),
		2 => Duration::from_millis(900),
		_ => Duration::from_millis(1500),
	}
}

fn call_with_retries<F>(
	label: &str,
	mut call: F,
) -> anyhow::Result<ureq::http::Response<ureq::Body>>
where
	F: FnMut() -> Result<ureq::http::Response<ureq::Body>, ureq::Error>,
{
	for attempt in 1..=MAX_HTTP_ATTEMPTS {
		match call() {
			Ok(resp) => return Ok(resp),
			Err(err) => {
				if !is_retryable_http_error(&err)
					|| attempt == MAX_HTTP_ATTEMPTS
				{
					return Err(err.into());
				}

				let delay = retry_delay(attempt);
				ui::warning(&format!(
					"{label} failed ({err}). Retrying in {}ms ({attempt}/{MAX_HTTP_ATTEMPTS})",
					delay.as_millis()
				));
				std::thread::sleep(delay);
			}
		}
	}

	anyhow::bail!("unreachable retry loop state")
}

fn normalize_sha256(value: &str) -> anyhow::Result<String> {
	let trimmed = value.trim();
	let without_prefix = trimmed.strip_prefix("sha256:").unwrap_or(trimmed);
	let normalized = without_prefix.to_ascii_lowercase();

	if normalized.len() != 64
		|| !normalized.chars().all(|c| c.is_ascii_hexdigit())
	{
		anyhow::bail!("invalid sha256 value: {value}");
	}

	Ok(normalized)
}

fn temp_download_path(out_path: &Path) -> PathBuf {
	let file_name = out_path
		.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or("download.bin");
	out_path.with_file_name(format!("{file_name}.part"))
}

/// Performs a blocking HTTP GET and deserializes the response body as JSON.
///
/// Errors if the server response is not `200 OK` or if the body cannot be
/// deserialized.
pub fn get_json<T: DeserializeOwned>(url: &str) -> anyhow::Result<T> {
	let resp = call_with_retries(&format!("GET {url}"), || request_get(url))?;

	let status = resp.status();
	if status != 200 {
		anyhow::bail!("HTTP {status} for {url}");
	}

	let mut reader = resp.into_body().into_reader();
	let mut buf = Vec::new();
	reader.read_to_end(&mut buf)?;

	let json = serde_json::from_slice::<T>(&buf)?;
	Ok(json)
}

/// Performs a blocking HTTP GET and returns response body as UTF-8 text.
pub fn get_text(url: &str) -> anyhow::Result<String> {
	let resp = call_with_retries(&format!("GET {url}"), || request_get(url))?;
	let status = resp.status();
	if status != 200 {
		anyhow::bail!("HTTP {status} for {url}");
	}

	let mut reader = resp.into_body().into_reader();
	let mut buf = Vec::new();
	reader.read_to_end(&mut buf)?;
	let text = String::from_utf8(buf)?;
	Ok(text)
}

/// Downloads a URL to a file, streaming to disk and showing a simple progress
/// bar when `Content-Length` is available.
pub fn download_to_file(url: &str, out_path: &Path) -> anyhow::Result<()> {
	download_to_file_internal(url, out_path, None)
}

/// Downloads a URL to a file and validates SHA-256.
pub fn download_to_file_with_sha256(
	url: &str,
	out_path: &Path,
	expected_sha256: &str,
) -> anyhow::Result<()> {
	let expected = normalize_sha256(expected_sha256)?;
	download_to_file_internal(url, out_path, Some(expected.as_str()))
}

fn download_to_file_internal(
	url: &str,
	out_path: &Path,
	expected_sha256: Option<&str>,
) -> anyhow::Result<()> {
	if let Some(parent) = out_path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let temp_path = temp_download_path(out_path);
	if temp_path.exists() {
		let _ = std::fs::remove_file(&temp_path);
	}

	let resp = call_with_retries(&format!("GET {url}"), || request_get(url))?;

	let status = resp.status();
	if status != 200 {
		anyhow::bail!("Download failed (HTTP {status})");
	}

	let total_bytes = resp
		.headers()
		.get("content-length")
		.and_then(|v| v.to_str().ok())
		.and_then(|s| s.parse::<u64>().ok());

	let mut reader = resp.into_body().into_reader();
	let mut file = std::fs::File::create(&temp_path)?;
	let mut hasher = Sha256::new();

	let mut downloaded: u64 = 0;
	let mut buf = vec![0_u8; 64 * 1024];

	let mut last_draw = Instant::now()
		.checked_sub(Duration::from_secs(10))
		.unwrap_or_else(Instant::now);

	loop {
		let n = reader.read(&mut buf)?;
		if n == 0 {
			break;
		}

		file.write_all(&buf[..n])?;
		hasher.update(&buf[..n]);
		downloaded += n as u64;

		if last_draw.elapsed() >= Duration::from_millis(120) {
			draw_progress(downloaded, total_bytes)?;
			last_draw = Instant::now();
		}
	}

	draw_progress(downloaded, total_bytes)?;
	println!();
	file.flush()?;

	if let Some(expected) = expected_sha256 {
		let actual = format!("{:x}", hasher.finalize());
		if actual != expected {
			let _ = std::fs::remove_file(&temp_path);
			anyhow::bail!(
				"sha256 mismatch for {}: expected {}, got {}",
				out_path.display(),
				expected,
				actual
			);
		}
	}

	if out_path.exists() {
		std::fs::remove_file(out_path)?;
	}
	std::fs::rename(&temp_path, out_path)?;

	Ok(())
}

fn draw_progress(downloaded: u64, total: Option<u64>) -> anyhow::Result<()> {
	let mut out = std::io::stdout();

	match total {
		Some(total) if total > 0 => {
			let pct = (downloaded as f64 / total as f64).min(1.0);
			let width = 28;
			let filled = (pct * width as f64).round() as usize;
			let filled = filled.min(width);
			let empty = width.saturating_sub(filled);

			let bar = format!("[{}{}]", "#".repeat(filled), ".".repeat(empty));

			let pct_s = format!("{:>3}%", (pct * 100.0).round() as u64);
			let cur = format_bytes(downloaded);
			let tot = format_bytes(total);

			print!("\r{bar} {pct_s} {cur}/{tot}");
		}
		_ => {
			let cur = format_bytes(downloaded);
			print!("\rDownloading... {cur}");
		}
	}

	out.flush()?;
	Ok(())
}

fn format_bytes(n: u64) -> String {
	const KIB: f64 = 1024.0;
	const MIB: f64 = KIB * 1024.0;
	const GIB: f64 = MIB * 1024.0;

	let n_f = n as f64;
	if n_f >= GIB {
		format!("{:.1}GiB", n_f / GIB)
	} else if n_f >= MIB {
		format!("{:.1}MiB", n_f / MIB)
	} else if n_f >= KIB {
		format!("{:.1}KiB", n_f / KIB)
	} else {
		format!("{n}B")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn format_bytes_bytes() {
		assert_eq!(format_bytes(0), "0B");
		assert_eq!(format_bytes(999), "999B");
	}

	#[test]
	fn format_bytes_kib() {
		assert_eq!(format_bytes(1024), "1.0KiB");
		assert_eq!(format_bytes(1536), "1.5KiB");
	}

	#[test]
	fn format_bytes_mib() {
		assert_eq!(format_bytes(1024 * 1024), "1.0MiB");
	}

	#[test]
	fn format_bytes_gib() {
		assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GiB");
	}

	#[test]
	fn normalize_sha256_handles_prefix() {
		let input = "sha256:1fc96c67f56be0e22fceff43a111b9c354f051cc1fc858599896c5887befc0c3";
		assert_eq!(
			normalize_sha256(input).unwrap(),
			"1fc96c67f56be0e22fceff43a111b9c354f051cc1fc858599896c5887befc0c3"
		);
	}

	#[test]
	fn normalize_sha256_rejects_bad_input() {
		assert!(normalize_sha256("abc123").is_err());
	}
}
