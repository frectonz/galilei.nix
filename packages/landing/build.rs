fn main() {
    topcoat::tailwind::BuildConfig::new()
        .executable_env("TAILWIND_CLI")
        .cwd("src")
        .render()
        .unwrap();
    topcoat::icon::iconify::BuildConfig::new()
        .cache_dir("icons")
        .icon_set("feather")
        .icon_set("brands")
        .stage()
        .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-env-changed=TAILWIND_CLI");
}
