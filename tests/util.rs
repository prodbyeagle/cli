use eagle::util::escape_sh_single_quoted;

#[test]
fn escape_no_special_chars() {
	assert_eq!(escape_sh_single_quoted("hello"), "hello");
}

#[test]
fn escape_single_quote() {
	assert_eq!(escape_sh_single_quoted("it's"), r"it'\''s");
}

#[test]
fn escape_multiple_quotes() {
	assert_eq!(escape_sh_single_quoted("a'b'c"), r"a'\''b'\''c");
}

#[test]
fn escape_only_quotes() {
	assert_eq!(escape_sh_single_quoted("'''"), r"'\'''\'''\''");
}

#[test]
fn escape_empty_string() {
	assert_eq!(escape_sh_single_quoted(""), "");
}

#[test]
fn escape_path_with_apostrophe() {
	assert_eq!(
		escape_sh_single_quoted("/Users/user's/eagle"),
		r"/Users/user'\''s/eagle"
	);
}
