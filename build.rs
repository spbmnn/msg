// build.rs
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // fixing a weird issue with my particular gstreamer install on macos
    #[cfg(target_os = "macos")]
    {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,/Library/Frameworks/GStreamer.framework/Versions/Current/lib"
        );
    }
}
