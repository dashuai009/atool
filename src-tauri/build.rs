extern crate pkg_config;
fn main() {
    println!("cargo:rustc-link-lib=static=snappy");
    println!("cargo:rustc-link-search=/opt/homebrew/Cellar/snappy/1.1.10/lib");
    let mut config = pkg_config::Config::new();
    config.statik(true).probe("x264").unwrap();

    config.statik(true).probe("dav1d").unwrap();
    config.statik(true).probe("aom").unwrap();
    config.statik(true).probe("opencore-amrwb").unwrap();
    //
    // // config.statik(true).probe("snappy").unwrap();
    // config.statik(true).probe("libavcodec").unwrap();
    // config.statik(true).probe("libavutil").unwrap();
    // config.statik(true).probe("libavdevice").unwrap();
    // config.statik(true).probe("libswscale").unwrap();
    // config.statik(true).probe("libswresample").unwrap();
    // config.statik(true).probe("libavformat").unwrap();
    config.statik(true).probe("x265").unwrap();
    config.atleast_version("1.13.1").statik(true).probe("vpx").unwrap();
    tauri_build::build()
}
