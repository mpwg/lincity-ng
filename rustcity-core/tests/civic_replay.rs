use std::{fs, path::Path, process::Command};

mod support;

/// Compares every daily observation and complete XML checkpoints on a map with
/// all six residence densities, municipal services, a shanty and a fire.
#[test]
#[ignore = "requires RUSTCITY_CPP_ARCHIVE and RUSTCITY_CPP_REPLAY"]
fn civic_district_matches_the_cpp_simulation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = root.join("tests/fixtures/civic.xml");
    let comparator = support::archived_cpp_core().join("tests/replay/compare.py");
    let dir = std::env::temp_dir().join(format!("rustcity-civic-replay-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    for (runner, mode, name) in [
        (
            std::env::var_os("RUSTCITY_CPP_REPLAY").unwrap(),
            "record",
            "reference",
        ),
        (
            env!("CARGO_BIN_EXE_rustcity-replay").into(),
            "replay",
            "candidate",
        ),
    ] {
        let result = Command::new(runner)
            .current_dir(&dir)
            .args([mode, "decisions"])
            .arg(&fixture)
            .args(["no-traffic", "1201", "-", name])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let compare = Command::new("python3")
        .arg(comparator)
        .current_dir(&dir)
        .args(["reference", "candidate"])
        .output()
        .unwrap();
    assert!(
        compare.status.success(),
        "{}",
        String::from_utf8_lossy(&compare.stderr)
    );
    fs::remove_dir_all(dir).unwrap();
}
