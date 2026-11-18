use std::{fs, path::Path, process::Command};

#[test]
fn replays_the_versioned_smithy_with_actions_and_checkpoint_days() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lincity-core/tests");
    let target = std::env::temp_dir().join(format!("rustcity-smithy-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_rustcity-replay"))
        .arg("replay")
        .arg(src.join("replay/decisions-v1.txt"))
        .arg(src.join("fixtures/blacksmith-2140.xml"))
        .args(["no-traffic", "5"])
        .arg(src.join("replay/actions.txt"))
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = fs::read_to_string(target.join("state.tsv")).unwrap();
    let expected = fs::read_to_string(src.join("replay/state-v1.tsv")).unwrap();
    assert_eq!(actual, expected);
    for d in [0, 1, 5] {
        assert!(target.join(format!("day-{d}.gz")).exists());
    }
    fs::remove_dir_all(target).unwrap();
}

#[test]
fn rejects_invalid_cli_inputs_with_the_reference_diagnostics() {
    check_invalid_inputs(Path::new(env!("CARGO_BIN_EXE_rustcity-replay")));
}

#[test]
#[ignore = "set RUSTCITY_CPP_REPLAY to the C++ core_replay executable"]
fn cpp_confirms_the_invalid_input_diagnostics() {
    check_invalid_inputs(Path::new(&std::env::var_os("RUSTCITY_CPP_REPLAY").unwrap()));
}

fn check_invalid_inputs(runner: &Path) {
    let name = runner.file_stem().unwrap().to_string_lossy();
    let root = std::env::temp_dir().join(format!("rustcity-invalid-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../lincity-core/tests/fixtures/blacksmith-2140.xml");
    let usage = Command::new(runner).output().unwrap();
    assert_eq!(usage.status.code(), Some(2));
    for (mode, days, input, diagnostic) in [
        ("invalid", "1", "", "invalid mode"),
        ("record", "+1", "", "invalid day count"),
        ("record", "2147483648", "", "invalid day count"),
        (
            "record",
            "1",
            "2 query 1 1",
            "invalid action or day at line 1",
        ),
        ("record", "1", "1 unknown 1 1", "unknown action: unknown"),
        ("record", "1", "1 build 1 1 51", "invalid group at line 1"),
        (
            "record",
            "1",
            "1 build 1 1 32",
            "unregistered building group",
        ),
        ("record", "1", "1 query 1 1 extra", "extra input at line 1"),
        ("replay", "1", "", "cannot open decision tape"),
    ] {
        fs::write(root.join("actions"), input).unwrap();
        let result = Command::new(runner)
            .current_dir(&root)
            .args([mode, "test-tape"])
            .arg(&fixture)
            .args(["no-traffic", days, "actions", "output"])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&result.stderr).trim_end(),
            diagnostic,
            "{input}"
        );
        let _ = fs::remove_file(root.join("test-tape"));
    }
    for (arguments, diagnostic) in [
        (
            [
                "record",
                "test-tape",
                "absent.city",
                "no-traffic",
                "0",
                "-",
                "output",
            ],
            "failed to open file: absent.city",
        ),
        (
            [
                "record",
                "test-tape",
                "absent.city",
                "no-traffic",
                "0",
                "absent.actions",
                "output",
            ],
            "cannot open actions: absent.actions",
        ),
    ] {
        let result = Command::new(runner)
            .current_dir(&root)
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&result.stderr).trim_end(),
            diagnostic
        );
    }
    fs::remove_dir_all(root).unwrap();
}
