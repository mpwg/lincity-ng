use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "cargo godot-build [--export] [--debug] [--target TRIPLE]\n\
    --export         Bibliothek für eine Godot-Releasevorlage bereitstellen\n\
    --debug          Unoptimierten Cargo-Build mit Debugsymbolen erzeugen\n\
    --target TRIPLE  Für ein anderes Rust-Ziel bauen";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mut target = None;
    let mut export = false;
    let mut debug = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--export" => export = true,
            "--debug" => debug = true,
            "--target" if target.is_none() => {
                target = Some(
                    args.next()
                        .filter(|arg| !arg.starts_with('-'))
                        .ok_or(USAGE)?,
                );
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => return Err(format!("Unbekanntes Argument {arg}.\n{USAGE}").into()),
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(&root).args([
        "build",
        "--locked",
        "--package",
        "rustcity-godot",
        "--lib",
        "--message-format=json-render-diagnostics",
    ]);
    if !debug {
        command.arg("--release");
    }
    if let Some(target) = target {
        command.args(["--target", &target]);
    }
    let output = command.stderr(std::process::Stdio::inherit()).output()?;
    if !output.status.success() {
        return Err(format!("Cargo-Build fehlgeschlagen ({})", output.status).into());
    }

    let source = library_artifact(&output.stdout)?;
    let directory = root.join("godot/bin");
    fs::create_dir_all(&directory)?;
    let filename_suffix = if export { ".release" } else { "" };
    let mut destination = directory.join(format!("liblincity_godot{filename_suffix}"));
    // Append the extension because `.release` is part of the filename stem.
    if let Some(extension) = source.extension() {
        destination.add_extension(extension);
    }
    let staging = tempfile::NamedTempFile::new_in(&directory)?;
    fs::copy(&source, staging.path())?;
    staging.persist(&destination)?;
    println!("{}", destination.canonicalize()?.display());
    Ok(())
}

fn library_artifact(output: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
    for line in std::str::from_utf8(output)?.lines() {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if message["reason"] != "compiler-artifact" || message["target"]["name"] != "lincity_godot"
        {
            continue;
        }
        if let Some(filenames) = message["filenames"].as_array() {
            for file in filenames.iter().filter_map(Value::as_str).map(Path::new) {
                if matches!(
                    file.extension().and_then(|suffix| suffix.to_str()),
                    Some("so" | "dll" | "dylib")
                ) {
                    return Ok(file.to_owned());
                }
            }
        }
    }
    Err("Cargo hat keine ladbare Godot-Bibliothek gemeldet.".into())
}
