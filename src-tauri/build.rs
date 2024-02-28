extern crate pkg_config;
fn main() {
    let mut config = pkg_config::Config::new();
    config.statik(true).probe("x264").unwrap();
    config.statik(true).probe("x265").unwrap();
    config.atleast_version("1.13.1").statik(true).probe("vpx").unwrap();
    tauri_build::build()
}
