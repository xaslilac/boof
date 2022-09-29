use std::process::Command;

mod setup;

const EXE: &str = "./build/release/boof";

#[test]
fn example_hello() {
	setup::before();

	let result = Command::new(EXE)
		.arg("./examples/hello.b")
		.output()
		.unwrap();
	let stdout = String::from_utf8_lossy(&result.stdout);

	assert_eq!(stdout, "hello, computer!\n");
}

#[test]
fn example_echo() {
	setup::before();

	let result = Command::new(EXE).arg("./examples/echo.b").output().unwrap();
	let stderr = String::from_utf8_lossy(&result.stderr);

	assert!(stderr.contains("no byte to read"));
}

#[test]
fn no_file() {
	setup::before();

	let result = Command::new(EXE).output().unwrap();
	let stderr = String::from_utf8_lossy(&result.stderr);

	assert!(stderr.contains("must provide an input file"));
}
