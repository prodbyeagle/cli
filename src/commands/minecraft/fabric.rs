use std::cmp::Ordering;
use std::path::Path;

use serde::Deserialize;

use crate::net;
use crate::ui;

/// Minimal shape of `GET https://meta.fabricmc.net/v2/versions/loader/{game_version}`.
#[derive(Debug, Clone, Deserialize)]
pub struct LoaderCombo {
	pub loader: LoaderPart,
	pub installer: InstallerPart,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoaderPart {
	pub version: String,
	#[serde(default)]
	pub stable: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallerPart {
	pub version: String,
	#[serde(default)]
	pub stable: Option<bool>,
}

pub(super) fn download_fabric_server(
	version: &str,
	jar_path: &Path,
) -> anyhow::Result<()> {
	ui::info(&format!("Downloading Fabric {version}..."));

	let url = format!("https://meta.fabricmc.net/v2/versions/loader/{version}");
	let combos = net::get_json::<Vec<LoaderCombo>>(&url)?;
	if combos.is_empty() {
		anyhow::bail!("No Fabric loader versions found for {version}");
	}

	let best = pick_best_combo(&combos)
		.ok_or_else(|| anyhow::anyhow!("No Fabric loader versions found"))?;

	let loader = &best.loader.version;
	let installer = &best.installer.version;

	let url = format!(
		"https://meta.fabricmc.net/v2/versions/loader/{version}/{loader}/{installer}/server/jar"
	);

	if let Some(sha256) = fetch_optional_sha256_for_url(&url) {
		net::download_to_file_with_sha256(&url, jar_path, &sha256)?;
	} else {
		ui::warning(
			"No checksum endpoint found for this Fabric artifact; downloading without digest verification.",
		);
		net::download_to_file(&url, jar_path)?;
	}
	Ok(())
}

fn fetch_optional_sha256_for_url(url: &str) -> Option<String> {
	let checksum_url = format!("{url}.sha256");
	let text = net::get_text(&checksum_url).ok()?;
	parse_sha256_token(&text)
}

#[doc(hidden)]
pub fn parse_sha256_token(text: &str) -> Option<String> {
	text.split_whitespace()
		.find(|token| {
			token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit())
		})
		.map(|token| token.to_ascii_lowercase())
}

#[doc(hidden)]
pub fn pick_best_combo(combos: &[LoaderCombo]) -> Option<&LoaderCombo> {
	let stable_max = combos
		.iter()
		.filter(|c| {
			c.loader.stable.unwrap_or(true)
				&& c.installer.stable.unwrap_or(true)
		})
		.max_by(|a, b| {
			cmp_numeric_dotted(&a.loader.version, &b.loader.version).then_with(
				|| {
					cmp_numeric_dotted(
						&a.installer.version,
						&b.installer.version,
					)
				},
			)
		});

	stable_max.or_else(|| combos.first())
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
