use std::path::Path;

fn main(){
    let cmdir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_path = Path::new(&cmdir).join("../build/Debug/");
    let src_path = Path::new(&cmdir).join("../cxx");
    // let _ = write!(std::io::stdout(), "path is = {}", path.to_string_lossy()).unwrap();
    println!("cargo:rustc-link-search=native={}", build_path.to_string_lossy());
    println!("cargo:rustc-link-lib=static=httpcpp");
    // println!("cargo::waring=httpcpp path is {}", path.to_string_lossy());

    println!("cargo:rerun-if-changed={}", src_path.to_string_lossy());
}