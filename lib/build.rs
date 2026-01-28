use std::path::Path;

fn main(){
    let cmdir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&cmdir).join("../build/Debug/");
    // let _ = write!(std::io::stdout(), "path is = {}", path.to_string_lossy()).unwrap();
    println!("cargo:rustc-link-search=native={}", path.to_string_lossy());
    println!("cargo:rustc-link-lib=static=httpcpp");
}