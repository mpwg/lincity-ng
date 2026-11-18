use std::{path::Path, process::Command};

#[test]
fn version_one_reference_cases() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("cases/v1/input.txt");
    let output = Command::new(env!("CARGO_BIN_EXE_lincity-core-slice"))
        .arg(input)
        .output()
        .expect("launch Rust case runner");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("../cases/v1/expected.txt")
    );
}
