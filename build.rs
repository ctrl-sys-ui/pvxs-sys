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
    
    println!("cargo:warning=INFO: Using EPICS_BASE: {}", epics_base);
    println!("cargo:warning=INFO: Using EPICS_HOST_ARCH: {}", epics_host_arch);
    
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
    
    // Get libevent location (bundled with PVXS)
    let libevent_base = env::var("EPICS_PVXS_LIBEVENT")
        .unwrap_or_else(|_| {
            // Default to bundled libevent within PVXS
            pvxs_base_path.join("bundle").join("usr").join(&epics_host_arch).to_string_lossy().to_string()
        });
    
    let libevent_base_path = PathBuf::from(&libevent_base);
    let libevent_include = libevent_base_path.join("include");
    let libevent_lib = libevent_base_path.join("lib");
    
    println!("cargo:warning=INFO: Using PVXS location: {}", pvxs_base);
    println!("cargo:warning=INFO: Using libevent location: {}", libevent_base);
    
    // Tell cargo to rerun this build script if files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=include/wrapper.h");
    println!("cargo:rerun-if-changed=src/client_wrapper.cpp");
    println!("cargo:rerun-if-changed=src/client_wrapper_async.cpp");
    println!("cargo:rerun-if-changed=src/client_wrapper_monitor.cpp");
    println!("cargo:rerun-if-changed=src/client_wrapper_rpc.cpp");
    println!("cargo:rerun-if-changed=src/server_wrapper.cpp");
    println!("cargo:rerun-if-env-changed=EPICS_BASE");
    println!("cargo:rerun-if-env-changed=EPICS_HOST_ARCH");
    println!("cargo:rerun-if-env-changed=EPICS_PVXS");
    println!("cargo:rerun-if-env-changed=PVXS_DIR");
    println!("cargo:rerun-if-env-changed=EPICS_PVXS_LIBEVENT");
    
    // Copy wrapper.h to cxxbridge include directory so it can be found
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cxxbridge_dir = out_dir.join("cxxbridge");
    let cxxbridge_include_dir = cxxbridge_dir.join("include");
    std::fs::create_dir_all(&cxxbridge_include_dir).ok();
    std::fs::copy("include/wrapper.h", cxxbridge_include_dir.join("wrapper.h")).ok();
    
    // Build the C++ bridge using cxx
    let mut build = cxx_build::bridge("src/bridge.rs");
    
    // Check if async feature is enabled
    if cfg!(feature = "async") {
        build.define("PVXS_ASYNC_ENABLED", "1");
    }
    
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
    
    // Get current directory for wrapper.h
    let include_dir = std::env::current_dir().unwrap().join("include");
    
    build
        .file("src/client_wrapper_async.cpp")
        .file("src/client_wrapper_monitor.cpp")
        .file("src/client_wrapper_rpc.cpp")
        .file("src/client_wrapper.cpp")
        .file("src/server_wrapper.cpp")
        .include(&include_dir)  // Add include directory first so wrapper.h is found
        .include(&epics_include)
        .include(epics_include.join("compiler").join(compiler_dir))
        .include(epics_include.join("os").join(os_dir))
        .include(&pvxs_include)
        .include(&libevent_include)  // Add libevent include path
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
    println!("cargo:rustc-link-search=native={}", libevent_lib.display());
    
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
    
    // Copy required DLLs to target directories for seamless execution
    copy_runtime_dlls(&epics_base_path, &pvxs_base_path, &libevent_base_path, &epics_host_arch);
    
    // Export include paths for dependent crates
    println!("cargo:include={}", pvxs_include.display());
    println!("cargo:include={}", epics_include.display());
    println!("cargo:include={}", libevent_include.display());
}

fn copy_runtime_dlls(epics_base: &PathBuf, pvxs_base: &PathBuf, libevent_base: &PathBuf, host_arch: &str) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Determine target directory (go up from OUT_DIR to find target/debug or target/release)
    let mut target_dir = out_dir.clone();
    while target_dir.file_name() != Some(std::ffi::OsStr::new("target")) {
        if !target_dir.pop() {
            return; // Silently skip if we can't find target directory
        }
    }
    
    // Determine which profile we're building (debug or release)
    let profile = if out_dir.to_string_lossy().contains("release") {
        "release"
    } else {
        "debug"
    };
    
    // Source paths for DLLs
    let pvxs_dll = pvxs_base.join("bin").join(host_arch).join("pvxs.dll");
    let com_dll = epics_base.join("bin").join(host_arch).join("Com.dll");
    let event_dll = libevent_base.join("lib").join("event_core.dll");
    
    // Copy to main profile directory and examples subdirectory
    let directories = [
        target_dir.join(profile),
        target_dir.join(profile).join("examples"),
    ];
    
    let mut copied_dlls = Vec::new();
    
    for dest_dir in &directories {
        // Only process directories that exist or can be created
        if std::fs::create_dir_all(dest_dir).is_err() {
            continue;
        }
        
        // Copy DLLs if they exist
        if pvxs_dll.exists() {
            std::fs::copy(&pvxs_dll, dest_dir.join("pvxs.dll")).ok();
            if !copied_dlls.contains(&"pvxs.dll") {
                copied_dlls.push("pvxs.dll");
            }
        }
        
        if com_dll.exists() {
            std::fs::copy(&com_dll, dest_dir.join("Com.dll")).ok();
            if !copied_dlls.contains(&"Com.dll") {
                copied_dlls.push("Com.dll");
            }
        }
        
        if event_dll.exists() {
            std::fs::copy(&event_dll, dest_dir.join("event_core.dll")).ok();
            if !copied_dlls.contains(&"event_core.dll") {
                copied_dlls.push("event_core.dll");
            }
        }
    }
    
    if !copied_dlls.is_empty() {
        println!("cargo:warning=INFO: Copied {} to {}", copied_dlls.join(", "), profile);
    }
}