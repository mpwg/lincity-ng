use super::*;
use std::{fs, path::Path, process::Command};

const LEGACY: &[u8] = include_bytes!("../../../lincity-core/tests/fixtures/migration-2130.xml");
const CURRENT: &[u8] = include_bytes!("../../../lincity-core/tests/fixtures/statistics-2140.xml");

#[test]
fn round_trip_preserves_building_data_and_encoded_text() {
    for bytes in [LEGACY, CURRENT] {
        let game = SaveGame::read(bytes).unwrap();
        assert!(
            game.globals()
                .child("given_scene")
                .unwrap()
                .text()
                .contains('&')
        );
        let mut compressed = Vec::new();
        game.write_gzip(&mut compressed).unwrap();
        let restored = SaveGame::read(compressed.as_slice()).unwrap();
        assert_eq!(restored, game);
        compressed.pop();
        assert!(SaveGame::read(compressed.as_slice()).is_err());
    }
}

#[test]
fn upgrades_legacy_accounts_and_history() {
    let mut game = SaveGame::read(LEGACY).unwrap();
    game.upgrade().unwrap();
    assert_eq!(game.version(), 2140);
    let globals = game.globals();
    assert_eq!(globals.child("total_money").unwrap().text(), "1501");
    let stats = globals.child("stats").unwrap();
    assert_eq!(
        stats
            .child("population")
            .unwrap()
            .child("unemployed_days_total")
            .unwrap()
            .text(),
        "2417"
    );
    let food = stats.child("history").unwrap().child("food").unwrap();
    assert_eq!(
        food.child("index_1")
            .unwrap()
            .child("amount")
            .unwrap()
            .text(),
        "620"
    );
    assert_eq!(game.map().attr("size"), Some("32"));
    let first = game.map().child("MapTile").unwrap();
    assert_eq!(
        first.child("flags").unwrap().text().parse::<u32>().unwrap(),
        0x22030000
    );
}

#[test]
fn invalid_versions_and_incomplete_xml_are_rejected() {
    for xml in [
        "<lc-game ldsv-version='2129'><globals/><map/></lc-game>",
        "<lc-game ldsv-version='2141'><globals/><map/></lc-game>",
        "<lc-game ldsv-version='2140'><globals/><map size='32'/>",
        "<lc-game ldsv-version='2140'><globals/><map size='-1'/></lc-game>",
        "<lc-game ldsv-version='2140'><globals/><map size='16'><MapTile group='21' map-x='16' map-y='3'/></map></lc-game>",
    ] {
        assert!(SaveGame::read(xml.as_bytes()).is_err(), "{xml}");
    }
}

#[test]
#[ignore = "set RUSTCITY_CPP_REPLAY to the built C++ core_replay executable"]
fn cpp_loads_rust_saves_and_rust_loads_cpp_checkpoints() {
    let replay = std::env::var_os("RUSTCITY_CPP_REPLAY").unwrap();
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lincity-core");
    let output = std::env::temp_dir().join(format!("rustcity-save-{}", std::process::id()));
    fs::create_dir(&output).unwrap();
    for path in [
        "tests/fixtures/migration-2130.xml",
        "tests/fixtures/statistics-2140.xml",
        "tests/fixtures/specialized-buildings-2140.xml",
        "tests/fixtures/blacksmith-2140.xml",
        "opening/Beach.scn.gz",
        "opening/bad_times.scn.gz",
        "opening/extreme_arid.scn.gz",
        "opening/extreme_wetland.scn.gz",
        "opening/good_times.scn.gz",
        "opening/Rocket_98.scn.gz",
    ] {
        let fixture = core.join(path);
        let dir = output.join(fixture.file_name().unwrap());
        fs::create_dir(&dir).unwrap();
        let record = |mode: &str, path: &Path, out: &str| {
            assert!(
                Command::new(&replay)
                    .current_dir(&dir)
                    .args([mode, "tape"])
                    .arg(path)
                    .args(["no-traffic", "1", "-", out])
                    .status()
                    .unwrap()
                    .success()
            );
        };
        record("record", &fixture, "reference");
        let game = crate::City::load(fs::File::open(&fixture).unwrap()).unwrap();
        let candidate = dir.join("candidate.gz");
        game.save(fs::File::create(&candidate).unwrap()).unwrap();
        record("replay", &candidate, "candidate");
        assert!(
            Command::new("python3")
                .arg(core.join("tests/replay/compare.py"))
                .current_dir(&dir)
                .args(["reference", "candidate"])
                .status()
                .unwrap()
                .success()
        );
        let checkpoint = dir.join("reference/day-1.gz");
        assert_eq!(
            SaveGame::read(fs::File::open(checkpoint).unwrap())
                .unwrap()
                .version(),
            2140
        );
    }
    fs::remove_dir_all(&output).unwrap();
}

#[test]
fn unsupported_roots_and_versions_explain_which_format_was_encountered() {
    for (xml, message) in [
        (
            "<SaveGame/>",
            "Detected a load/save format that is too old. See the readme for how to update the load/save format.",
        ),
        (
            "<other/>",
            "invalid save file: expected <lc-game>, found <other>",
        ),
        (
            "<lc-game/>",
            "failed to parse load/save version: missing attribute ldsv-version in <lc-game>",
        ),
        (
            "<lc-game ldsv-version='2129'/>",
            "load/save version too old",
        ),
        (
            "<lc-game ldsv-version='2141'/>",
            "load/save version too new",
        ),
    ] {
        assert_eq!(
            SaveGame::read(xml.as_bytes()).unwrap_err().to_string(),
            message
        );
    }
}
