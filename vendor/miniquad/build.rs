use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));

    if target.contains("darwin") || target.contains("ios") {
        println!("cargo:rustc-link-lib=framework=MetalKit");
    }

    // PS5: compile PS5_VulkanTemplate's platform.c (klog, pads, audio, exit). It includes
    // <ps5platform/klog.h> from mihawk-99's payload SDK fork; point PS5_PLATFORM_SDK at that
    // SDK (the folder holding target/include). Without it the C part is skipped: the crate
    // still type-checks, but a PS5 link would miss its symbols.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("freebsd") {
        println!("cargo:rerun-if-env-changed=PS5_PLATFORM_SDK");
        println!("cargo:rerun-if-changed=src/native/ps5c/platform.c");
        match env::var("PS5_PLATFORM_SDK") {
            Ok(sdk) => {
                cc::Build::new()
                    .file("src/native/ps5c/platform.c")
                    .include(format!("{sdk}/target/include"))
                    .include(format!("{sdk}/target/include_common"))
                    .flag_if_supported("-std=c11")
                    .compile("ps5_title_platform");
            }
            Err(_) => println!("cargo:warning=PS5_PLATFORM_SDK not set: platform.c not compiled (type-check only)"),
        }
    }
}
