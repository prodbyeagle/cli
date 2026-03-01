use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::net;
use crate::ui;

/// Minimal shape of `GET https://fill.papermc.io/v3/projects/paper`.
#[derive(Debug, Clone, Deserialize)]
struct FillProjectIndex {
	versions: HashMap<String, Vec<String>>,
}

pub(super) fn resolve_paper_version(version: &str) -> anyhow::Result<String> {
	let version = version.trim();
	if !looks_like_family_key(version) {
		return Ok(version.to_string());
	}

	let index = net::get_json::<FillProjectIndex>(
		"https://fill.papermc.io/v3/projects/paper",
	)?;

	let versions = index.versions.get(version).ok_or_else(|| {
		anyhow::anyhow!("Unknown Paper version family: {version}")
	})?;

	let best = pick_best_version_for_family(versions).ok_or_else(|| {
		anyhow::anyhow!("No versions found for Paper family: {version}")
	})?;

	Ok(best.to_string())
}

#[doc(hidden)]
pub fn looks_like_family_key(s: &str) -> bool {
	let s = s.trim();
	if s.is_empty() || s.contains('-') {
		return false;
	}

	let parts: Vec<&str> = s.split('.').collect();
	if parts.len() != 2 {
		return false;
	}

	parts
		.iter()
		.all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

#[doc(hidden)]
pub fn pick_best_version_for_family(versions: &[String]) -> Option<&str> {
	let stable_max = versions
		.iter()
		.filter(|v| !v.contains('-'))
		.max_by(|a, b| cmp_numeric_dotted(a, b));

	stable_max.or_else(|| versions.first()).map(|s| s.as_str())
}

fn cmp_numeric_dotted(a: &str, b: &str) -> Ordering {
	let pa = a.split('.').collect::<Vec<_>>();
	let pb = b.split('.').collect::<Vec<_>>();
	let max_len = pa.len().max(pb.len());

	for idx in 0..max_len {
		let av = pa.get(idx).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
		let bv = pb.get(idx).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
		match av.cmp(&bv) {
			Ordering::Equal => continue,
			other => return other,
		}
	}

	Ordering::Equal
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillBuild {
	pub id: u64,
	pub channel: String,
	pub downloads: HashMap<String, FillDownload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillDownload {
	pub name: String,
	pub checksums: FillChecksums,
	pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillChecksums {
	pub sha256: String,
}

pub(super) fn download_paper_server(
	version: &str,
	jar_path: &Path,
) -> anyhow::Result<()> {
	ui::info(&format!("Downloading Paper {version}..."));

	let url = format!(
		"https://fill.papermc.io/v3/projects/paper/versions/{version}/builds"
	);
	let builds = net::get_json::<Vec<FillBuild>>(&url)?;
	if builds.is_empty() {
		anyhow::bail!("No Paper builds found for {version}");
	}

	let best = pick_best_build(&builds).ok_or_else(|| {
		anyhow::anyhow!("No Paper builds found for {version}")
	})?;

	let download = best
		.downloads
		.get("server:default")
		.ok_or_else(|| anyhow::anyhow!("Missing Paper server download"))?;

	ui::muted(&format!(
		"Build {}: {} (sha256 {})",
		best.id, download.name, download.checksums.sha256
	));

	net::download_to_file_with_sha256(
		&download.url,
		jar_path,
		&download.checksums.sha256,
	)?;
	Ok(())
}

#[doc(hidden)]
pub fn pick_best_build(builds: &[FillBuild]) -> Option<&FillBuild> {
	builds
		.iter()
		.filter(|b| b.channel == "STABLE")
		.max_by_key(|b| b.id)
		.or_else(|| builds.iter().max_by_key(|b| b.id))
}
