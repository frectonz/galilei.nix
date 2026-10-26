fn main() {
    topcoat::icon::iconify::BuildConfig::new()
        .cache_dir("icons")
        .icon_set("feather")
        .icon_set("brands")
        .stage()
        .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons");
}
