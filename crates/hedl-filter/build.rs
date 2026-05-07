use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("builtin_filters.hedl");

    let hedl_path = Path::new("filters/filters.hedl");
    let content = fs::read_to_string(hedl_path).unwrap_or_default();

    fs::write(&dest_path, content).unwrap();
    println!("cargo:rerun-if-changed=filters/filters.hedl");
}
