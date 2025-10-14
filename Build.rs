use std::env;
use std::path::PathBuf;

fn main() {
    // Get EPICS_BASE from environment
    let epics_base = env::var("EPICS_BASE")
        .expect("EPICS_BASE environment variable not set. Please set it to your EPICS base installation path.");
    
    let epics_base_path = PathBuf::from(&epics_base);
    
    // Determine EPICS host architecture
    let epics_host_arch = env::var("EPICS_HOST_ARCH")
        .unwrap_or_else(|_| {
            // Try to determine from common patterns
            if cfg!(target_os = "windows") {
                if cfg!(target_pointer_width = "64") {
                    "windows-x64".to_string()
                } else {
                    "win32-x86".to_string()
                }
            } else if cfg!(target_os = "linux") {
                if cfg!(target_pointer_width = "64") {
                    "linux-x86_64".to_string()
                } else {
                    "linux-x86".to_string()
                }
            } else if cfg!(target_os = "macos") {
                "darwin-x86".to_string()
            } else {
                panic!("Unable to determine EPICS_HOST_ARCH. Please set it manually.")
            }
        });
    
    println!("cargo:warning=Using EPICS_BASE: {}", epics_base);
    println!("cargo:warning=Using EPICS_HOST_ARCH: {}", epics_host_arch);
    
    // EPICS Base paths
    let epics_include = epics_base_path.join("include");
    let epics_lib = epics_base_path.join("lib").join(&epics_host_arch);
    
    // Get PVXS location (could be within EPICS base or separate)
    let pvxs_base = env::var("EPICS_PVXS")
        .or_else(|_| env::var("PVXS_DIR"))
        .or_else(|_| env::var("PVXS_BASE"))
        .unwrap_or_else(|_| {
            // Assume PVXS is built as an EPICS module within base
            epics_base.clone()
        });
    
    let pvxs_base_path = PathBuf::from(&pvxs_base);
    let pvxs_include = pvxs_base_path.join("include");
    let pvxs_lib = pvxs_base_path.join("lib").join(&epics_host_arch);
    
    println!("cargo:warning=Using PVXS location: {}", pvxs_base);
    
    // Tell cargo to rerun this build script if files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=include/adapter.h");
    println!("cargo:rerun-if-changed=src/adapter.cpp");
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-env-changed=EPICS_BASE");
    println!("cargo:rerun-if-env-changed=EPICS_HOST_ARCH");
    println!("cargo:rerun-if-env-changed=EPICS_PVXS");
    println!("cargo:rerun-if-env-changed=PVXS_DIR");
    
    // Copy adapter.h to cxxbridge include directory so it can be found
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cxxbridge_dir = out_dir.join("cxxbridge");
    let cxxbridge_include_dir = cxxbridge_dir.join("include");
    std::fs::create_dir_all(&cxxbridge_include_dir).ok();
    std::fs::copy("include/adapter.h", cxxbridge_include_dir.join("adapter.h")).ok();
    
    // Build the C++ bridge using cxx
    let mut build = cxx_build::bridge("src/bridge.rs");
    
    // Platform-specific compiler and OS includes
    let (compiler_dir, os_dir) = if cfg!(target_os = "windows") {
        ("msvc", "WIN32")
    } else if cfg!(target_os = "linux") {
        ("gcc", "Linux")
    } else if cfg!(target_os = "macos") {
        ("clang", "Darwin")
    } else {
        ("gcc", "default")
    };
    
    // Get current directory for adapter.h
    let include_dir = std::env::current_dir().unwrap().join("include");
    
    build
        .file("src/adapter.cpp")
        .include(&include_dir)  // Add include directory first so adapter.h is found
        .include(&epics_include)
        .include(epics_include.join("compiler").join(compiler_dir))
        .include(epics_include.join("os").join(os_dir))
        .include(&pvxs_include)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("/std:c++11");  // MSVC
    
    // Platform-specific flags
    if cfg!(target_os = "windows") {
        build.flag_if_supported("/EHsc"); // Enable C++ exceptions on MSVC
    } else {
        build.flag_if_supported("-fexceptions");
        build.flag_if_supported("-pthread");
    }
    
    build.compile("epics_pvxs_sys");
    
    // Link to PVXS and EPICS libraries
    println!("cargo:rustc-link-search=native={}", pvxs_lib.display());
    println!("cargo:rustc-link-search=native={}", epics_lib.display());
    
    // Link required libraries
    println!("cargo:rustc-link-lib=pvxs");
    println!("cargo:rustc-link-lib=Com");  // EPICS Base Com library
    
    // Platform-specific system libraries
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=rt");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=advapi32");
    }
    
    // Export include paths for dependent crates
    println!("cargo:include={}", pvxs_include.display());
    println!("cargo:include={}", epics_include.display());
}