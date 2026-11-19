use std::path::PathBuf;

pub fn archived_cpp_core() -> PathBuf {
    let archive = std::env::var_os("RUSTCITY_CPP_ARCHIVE")
        .expect("set RUSTCITY_CPP_ARCHIVE to the root of the archived legacy checkout");
    let core = PathBuf::from(archive).join("lincity-core");
    core.canonicalize()
        .unwrap_or_else(|error| panic!("cannot locate archived C++ core at {core:?}: {error}"))
}
