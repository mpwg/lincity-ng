use rustcity_core::{City, Climate, GenerationOptions, Point, decisions::Decisions};
use std::{fs, io::BufReader, path::Path, process::Command};

#[test]
fn generation_validates_size_before_drawing_and_builds_a_seeded_village() {
    let settings = GenerationOptions {
        climate: Climate::River,
        village: true,
        trees: false,
    };
    assert!(City::generate_with_decisions(25, settings, Decisions::native(448)).is_err());
    let city = City::generate_with_decisions(26, settings, Decisions::native(448)).unwrap();
    assert_eq!(city.building_inventories().count(), 54);
    assert!(city.terrain(Point { x: 25, y: 25 }).is_some());
}

fn compare_generated(
    generator: &std::ffi::OsStr,
    scratch: &Path,
    id: usize,
    settings: GenerationOptions,
    size: i32,
    seed: u64,
) {
    let case = scratch.join(format!(
        "{id}-{}-{}-{size}-{seed}",
        settings.village, settings.trees
    ));
    fs::create_dir_all(&case).unwrap();
    let tape = case.join("decisions");
    let cpp = case.join("cpp.gz");
    let rust = case.join("rust.gz");
    let cpp_run = |mode: &str| {
        let result = Command::new(generator)
            .args([
                id.to_string(),
                size.to_string(),
                seed.to_string(),
                u8::from(settings.village).to_string(),
                u8::from(!settings.trees).to_string(),
            ])
            .arg(&tape)
            .arg(&cpp)
            .arg(mode)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{case:?} {mode}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    for cpp_records in [true, false] {
        let decisions = if cpp_records {
            cpp_run("record");
            Decisions::replay(BufReader::new(fs::File::open(&tape).unwrap())).unwrap()
        } else {
            Decisions::record(seed, fs::File::create(&tape).unwrap()).unwrap()
        };
        let city = City::generate_with_decisions(size, settings, decisions)
            .unwrap_or_else(|e| panic!("{case:?}: {e}"));
        city.save(fs::File::create(&rust).unwrap()).unwrap();
        city.finish_decisions().unwrap();
        if !cpp_records {
            cpp_run("replay");
        }
        let check = Command::new("python3")
            .arg("-B")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../lincity-core/tests/replay/compare.py"),
            )
            .arg("--saves")
            .arg(&cpp)
            .arg(&rust)
            .output()
            .unwrap();
        assert!(
            check.status.success(),
            "{case:?}, C++ recording {cpp_records}: {}",
            String::from_utf8_lossy(&check.stderr)
        );
    }
    fs::remove_dir_all(case).unwrap();
}

#[test]
#[ignore = "set RUSTCITY_CPP_GENERATOR to the built core_generation_reference"]
fn cpp_and_rust_generate_matching_saves_for_every_climate_tree_and_village_setting() {
    let generator = std::env::var_os("RUSTCITY_CPP_GENERATOR").unwrap();
    let scratch = std::env::temp_dir().join(format!("rustcity-gen-{}", std::process::id()));
    for (id, climate) in [
        Climate::River,
        Climate::Desert,
        Climate::Temperate,
        Climate::Swamp,
    ]
    .into_iter()
    .enumerate()
    {
        for village in [false, true] {
            for trees in [false, true] {
                let settings = GenerationOptions {
                    climate,
                    village,
                    trees,
                };
                compare_generated(&generator, &scratch, id, settings, 50, 615);
                compare_generated(&generator, &scratch, id, settings, 100, 615);
            }
        }
    }
    for (size, seed, village) in [
        (13, 615, false),
        (26, 448, true),
        (50, 4, false),
        (50, 448, false),
    ] {
        compare_generated(
            &generator,
            &scratch,
            2,
            GenerationOptions {
                climate: Climate::Temperate,
                village,
                trees: false,
            },
            size,
            seed,
        );
    }
    fs::remove_dir_all(scratch).unwrap();
}
