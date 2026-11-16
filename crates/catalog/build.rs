//! Compiles the install prefix into `INSTALLED_DIR`. A release build reads
//! no runtime variable, so the prefix is fixed when the binary is built.

fn main() {
    println!("cargo:rerun-if-env-changed=GARUDA_TOOLBOX_PREFIX");
    let prefix = std::env::var("GARUDA_TOOLBOX_PREFIX").unwrap_or_else(|_| "/usr".to_owned());
    println!("cargo:rustc-env=GARUDA_TOOLBOX_INSTALLED_DIR={prefix}/share/garuda-toolbox/catalog");
}
