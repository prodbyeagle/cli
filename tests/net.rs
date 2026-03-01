use std::path::Path;

use eagle::net::{
	format_bytes, is_retryable_http_error, normalize_sha256, retry_delay,
	temp_download_path,
};

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

#[test]
fn normalize_sha256_rejects_correct_length_but_non_hex() {
	let non_hex =
		"zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
	assert_eq!(non_hex.len(), 64);
	assert!(normalize_sha256(non_hex).is_err());
}

#[test]
fn normalize_sha256_is_case_insensitive() {
	let upper = "sha256:1FC96C67F56BE0E22FCEFF43A111B9C354F051CC1FC858599896C5887BEFC0C3";
	let result = normalize_sha256(upper).unwrap();
	assert_eq!(
		result,
		"1fc96c67f56be0e22fceff43a111b9c354f051cc1fc858599896c5887befc0c3"
	);
}

#[test]
fn retryable_status_codes() {
	assert!(is_retryable_http_error(&ureq::Error::StatusCode(408)));
	assert!(is_retryable_http_error(&ureq::Error::StatusCode(429)));
	assert!(is_retryable_http_error(&ureq::Error::StatusCode(500)));
	assert!(is_retryable_http_error(&ureq::Error::StatusCode(503)));
	assert!(is_retryable_http_error(&ureq::Error::StatusCode(599)));
}

#[test]
fn non_retryable_status_codes() {
	assert!(!is_retryable_http_error(&ureq::Error::StatusCode(400)));
	assert!(!is_retryable_http_error(&ureq::Error::StatusCode(403)));
	assert!(!is_retryable_http_error(&ureq::Error::StatusCode(404)));
	assert!(!is_retryable_http_error(&ureq::Error::StatusCode(200)));
}

#[test]
fn retryable_network_errors() {
	assert!(is_retryable_http_error(&ureq::Error::HostNotFound));
	assert!(is_retryable_http_error(&ureq::Error::ConnectionFailed));
}

#[test]
fn retry_delays_increase_with_attempt() {
	assert!(retry_delay(1) < retry_delay(2));
	assert!(retry_delay(2) < retry_delay(3));
}

#[test]
fn retry_delay_caps_after_third_attempt() {
	assert_eq!(retry_delay(3), retry_delay(4));
	assert_eq!(retry_delay(4), retry_delay(100));
}

#[test]
fn temp_download_path_adds_part_suffix() {
	let p = temp_download_path(Path::new("/tmp/eagle.exe"));
	assert_eq!(p, Path::new("/tmp/eagle.exe.part"));
}

#[test]
fn temp_download_path_preserves_parent() {
	let p = temp_download_path(Path::new("/some/dir/file.jar"));
	assert_eq!(p.parent().unwrap(), Path::new("/some/dir"));
}
