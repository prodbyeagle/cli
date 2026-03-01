use std::path::Path;

use eagle::commands::minecraft::start::build_java_args;

#[test]
fn args_contain_heap_flags() {
	let args = build_java_args(4096, Path::new("/srv/server.jar"));
	assert!(args.contains(&"-Xmx4096M".to_string()));
	assert!(args.contains(&"-Xms4096M".to_string()));
}

#[test]
fn args_end_with_jar_and_nogui() {
	let args = build_java_args(1024, Path::new("/srv/server.jar"));
	let last = args.last().unwrap();
	let second_last = &args[args.len() - 2];
	assert_eq!(last, "nogui");
	assert!(
		second_last.ends_with("server.jar"),
		"expected jar path, got: {second_last}"
	);
}

#[test]
fn args_include_g1gc() {
	let args = build_java_args(2048, Path::new("server.jar"));
	assert!(args.contains(&"-XX:+UseG1GC".to_string()));
}

#[test]
fn args_include_aikars_flags() {
	let args = build_java_args(2048, Path::new("server.jar"));
	assert!(args.contains(&"-Daikars.new.flags=true".to_string()));
}

#[test]
fn ram_value_is_reflected_in_both_heap_flags() {
	let args = build_java_args(512, Path::new("server.jar"));
	assert!(args.contains(&"-Xmx512M".to_string()));
	assert!(args.contains(&"-Xms512M".to_string()));
}
