use cmake;

fn main () {
    // Builds the project in the directory located in `libfoo`, installing it
    // into $OUT_DIR
    let dst = cmake::build("libgpredict");

    let glib = pkg_config::Config::new().probe("glib-2.0").unwrap();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=gpredict");

    for p in glib.link_paths {
        println!("cargo:rustc-link-search=native={}", p.to_str().unwrap());
    }

    for l in glib.libs {
        println!("cargo:rustc-link-lib=dylib={}", l);
    }
}