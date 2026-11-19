fn main() {
    let sources = ["src/lib.rs", "src/view.rs"];
    cxx_build::bridges(sources)
        .std("c++20")
        .compile("rustcity-godot-bridge");
    for source in sources {
        println!("cargo:rerun-if-changed={source}");
    }
}
