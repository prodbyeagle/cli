use eagle::commands::minecraft::fabric::{
	InstallerPart, LoaderCombo, LoaderPart, parse_sha256_token, pick_best_combo,
};

#[test]
fn picks_stable_combo_when_available() {
	let combos = vec![
		LoaderCombo {
			loader: LoaderPart {
				version: "0.16.0".to_string(),
				stable: Some(false),
			},
			installer: InstallerPart {
				version: "1.0.0".to_string(),
				stable: Some(true),
			},
		},
		LoaderCombo {
			loader: LoaderPart {
				version: "0.15.0".to_string(),
				stable: Some(true),
			},
			installer: InstallerPart {
				version: "1.0.0".to_string(),
				stable: Some(true),
			},
		},
	];

	let best = pick_best_combo(&combos).unwrap();
	assert_eq!(best.loader.version, "0.15.0");
}

#[test]
fn falls_back_to_first_combo() {
	let combos = vec![LoaderCombo {
		loader: LoaderPart {
			version: "0.16.0".to_string(),
			stable: Some(false),
		},
		installer: InstallerPart {
			version: "1.0.0".to_string(),
			stable: Some(false),
		},
	}];

	let best = pick_best_combo(&combos).unwrap();
	assert_eq!(best.loader.version, "0.16.0");
}

#[test]
fn picks_highest_stable_combo() {
	let combos = vec![
		LoaderCombo {
			loader: LoaderPart {
				version: "0.15.0".to_string(),
				stable: Some(true),
			},
			installer: InstallerPart {
				version: "1.0.0".to_string(),
				stable: Some(true),
			},
		},
		LoaderCombo {
			loader: LoaderPart {
				version: "0.18.4".to_string(),
				stable: Some(true),
			},
			installer: InstallerPart {
				version: "1.1.1".to_string(),
				stable: Some(true),
			},
		},
	];

	let best = pick_best_combo(&combos).unwrap();
	assert_eq!(best.loader.version, "0.18.4");
	assert_eq!(best.installer.version, "1.1.1");
}

#[test]
fn checksum_parser_accepts_hex_token() {
	let txt = "1fc96c67f56be0e22fceff43a111b9c354f051cc1fc858599896c5887befc0c3  server.jar";
	let parsed = parse_sha256_token(txt);
	assert_eq!(
		parsed.as_deref(),
		Some(
			"1fc96c67f56be0e22fceff43a111b9c354f051cc1fc858599896c5887befc0c3"
		)
	);
}
