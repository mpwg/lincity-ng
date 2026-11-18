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
