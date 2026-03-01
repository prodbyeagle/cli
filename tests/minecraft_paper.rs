use std::collections::HashMap;

use eagle::commands::minecraft::paper::{
	FillBuild, looks_like_family_key, pick_best_build,
	pick_best_version_for_family,
};

#[test]
fn family_key_detection() {
	assert!(looks_like_family_key("1.21"));
	assert!(looks_like_family_key(" 1.21 "));
	assert!(!looks_like_family_key("1.21.11"));
	assert!(!looks_like_family_key("1.21-rc1"));
	assert!(!looks_like_family_key("paper"));
	assert!(!looks_like_family_key(""));
}

#[test]
fn pick_best_version_prefers_non_prerelease() {
	let versions = vec![
		"1.21.11-rc3".to_string(),
		"1.21.10".to_string(),
		"1.21.11".to_string(),
	];
	assert_eq!(pick_best_version_for_family(&versions), Some("1.21.11"));
}

#[test]
fn pick_best_version_chooses_highest_stable() {
	let versions = vec![
		"1.21.2".to_string(),
		"1.21.12".to_string(),
		"1.21.9".to_string(),
	];
	assert_eq!(pick_best_version_for_family(&versions), Some("1.21.12"));
}

#[test]
fn pick_best_version_falls_back_to_first() {
	let versions = vec!["1.21.11-rc3".to_string()];
	assert_eq!(pick_best_version_for_family(&versions), Some("1.21.11-rc3"));
}

#[test]
fn pick_best_build_prefers_stable_highest_id() {
	let builds = vec![
		FillBuild {
			id: 1,
			channel: "STABLE".to_string(),
			downloads: HashMap::new(),
		},
		FillBuild {
			id: 10,
			channel: "BETA".to_string(),
			downloads: HashMap::new(),
		},
		FillBuild {
			id: 5,
			channel: "STABLE".to_string(),
			downloads: HashMap::new(),
		},
	];

	assert_eq!(pick_best_build(&builds).map(|b| b.id), Some(5));
}
