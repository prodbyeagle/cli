use eagle::util::escape_powershell_single_quoted;

#[test]
fn escape_no_special_chars() {
	assert_eq!(escape_powershell_single_quoted("hello"), "hello");
}

#[test]
fn escape_single_quote_doubled() {
	assert_eq!(escape_powershell_single_quoted("it's"), "it''s");
}

#[test]
fn escape_multiple_quotes() {
	assert_eq!(escape_powershell_single_quoted("a'b'c"), "a''b''c");
}

#[test]
fn escape_only_quotes() {
	assert_eq!(escape_powershell_single_quoted("'''"), "''''''");
}

#[test]
fn escape_empty_string() {
	assert_eq!(escape_powershell_single_quoted(""), "");
}

#[test]
fn escape_path_with_apostrophe() {
	assert_eq!(
		escape_powershell_single_quoted("C:\\Users\\user's\\eagle.exe"),
		"C:\\Users\\user''s\\eagle.exe"
	);
}
