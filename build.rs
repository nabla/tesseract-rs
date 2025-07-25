#[cfg(feature = "build-tesseract")]
mod build_tesseract {
    use cmake::Config;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    const LEPTONICA_URL: &str =
        "https://github.com/DanBloomberg/leptonica/archive/refs/tags/1.85.0.zip";
    const TESSERACT_URL: &str =
        "https://github.com/tesseract-ocr/tesseract/archive/refs/tags/5.5.1.zip";

    pub fn build() {
        let out_dir_string = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
        let out_dir = Path::new(&out_dir_string);
        let third_party_dir = out_dir.join("third_party");

        let leptonica_dir = if third_party_dir.join("leptonica").exists() {
            third_party_dir.join("leptonica")
        } else {
            fs::create_dir_all(&third_party_dir).expect("Failed to create third_party directory");
            download_and_extract(&third_party_dir, LEPTONICA_URL, "leptonica")
        };

        let tesseract_dir = if third_party_dir.join("tesseract").exists() {
            third_party_dir.join("tesseract")
        } else {
            fs::create_dir_all(&third_party_dir).expect("Failed to create third_party directory");
            download_and_extract(&third_party_dir, TESSERACT_URL, "tesseract")
        };

        let (cmake_cxx_flags, additional_defines) = get_os_specific_config();

        let leptonica_install_dir = out_dir.join("leptonica");

        build_inner("leptonica", &leptonica_install_dir, || {
            let mut leptonica_config = Config::new(&leptonica_dir);

            let leptonica_src_dir = leptonica_dir.join("src");
            let environ_h_path = leptonica_src_dir.join("environ.h");
            let environ_h = std::fs::read_to_string(&environ_h_path)
                .expect("Failed to read environ.h")
                .replace(
                    "#define  HAVE_LIBZ          1",
                    "#define  HAVE_LIBZ          0",
                )
                .replace(
                    "#ifdef  NO_CONSOLE_IO",
                    "#define NO_CONSOLE_IO\n#ifdef  NO_CONSOLE_IO",
                );
            std::fs::write(environ_h_path, environ_h).expect("Failed to write environ.h");

            let makefile_static_path = leptonica_dir.join("prog").join("makefile.static");
            let makefile_static = std::fs::read_to_string(&makefile_static_path)
                .expect("Failed to read makefile.static")
                .replace(
                    "ALL_LIBS =	$(LEPTLIB) -ltiff -ljpeg -lpng -lz -lm",
                    "ALL_LIBS =	$(LEPTLIB) -lm",
                );
            std::fs::write(makefile_static_path, makefile_static)
                .expect("Failed to write makefile.static");

            if env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache" {
                leptonica_config
                    .env("CC", "sccache cc")
                    .env("CXX", "sccache c++");
            }
            leptonica_config
                .define("CMAKE_BUILD_TYPE", "Release")
                .define("BUILD_PROG", "OFF")
                .define("BUILD_SHARED_LIBS", "OFF")
                .define("ENABLE_ZLIB", "OFF")
                .define("ENABLE_PNG", "OFF")
                .define("ENABLE_JPEG", "OFF")
                .define("ENABLE_TIFF", "OFF")
                .define("ENABLE_WEBP", "OFF")
                .define("ENABLE_OPENJPEG", "OFF")
                .define("ENABLE_GIF", "OFF")
                .define("NO_CONSOLE_IO", "ON")
                .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags)
                .define("MINIMUM_SEVERITY", "L_SEVERITY_NONE")
                .define("SW_BUILD", "OFF")
                .define("HAVE_LIBZ", "0")
                .define("ENABLE_LTO", "OFF")
                .define("CMAKE_INSTALL_PREFIX", &leptonica_install_dir);

            for (key, value) in &additional_defines {
                leptonica_config.define(key, value);
            }

            leptonica_config.build();
        });

        let leptonica_include_dir = leptonica_install_dir.join("include");
        let leptonica_lib_dir = leptonica_install_dir.join("lib");
        let tesseract_install_dir = out_dir.join("tesseract");

        build_inner("tesseract", &tesseract_install_dir, || {
            let cmakelists_path = tesseract_dir.join("CMakeLists.txt");
            let cmakelists = std::fs::read_to_string(&cmakelists_path)
                .expect("Failed to read CMakeLists.txt")
                .replace("set(HAVE_TIFFIO_H ON)", "");
            std::fs::write(&cmakelists_path, cmakelists).expect("Failed to write CMakeLists.txt");

            let mut tesseract_config = Config::new(&tesseract_dir);
            if env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache" {
                tesseract_config
                    .env("CC", "sccache cc")
                    .env("CXX", "sccache c++");
            }
            tesseract_config
                .define("CMAKE_BUILD_TYPE", "Release")
                .define("BUILD_TRAINING_TOOLS", "OFF")
                .define("BUILD_SHARED_LIBS", "OFF")
                .define("DISABLE_ARCHIVE", "ON")
                .define("DISABLE_CURL", "ON")
                .define("DISABLE_OPENCL", "ON")
                .define("LEPTONICA_DIR", &leptonica_install_dir)
                .define("LEPTONICA_INCLUDE_DIR", &leptonica_include_dir)
                .define("LEPTONICA_LIBRARY", &leptonica_lib_dir)
                .define("CMAKE_PREFIX_PATH", &leptonica_install_dir)
                .define("CMAKE_INSTALL_PREFIX", &tesseract_install_dir)
                .define("DISABLE_TIFF", "ON")
                .define("DISABLE_PNG", "ON")
                .define("DISABLE_JPEG", "ON")
                .define("DISABLE_WEBP", "ON")
                .define("DISABLE_OPENJPEG", "ON")
                .define("DISABLE_ZLIB", "ON")
                .define("DISABLE_LIBXML2", "ON")
                .define("DISABLE_LIBICU", "ON")
                .define("DISABLE_LZMA", "ON")
                .define("DISABLE_GIF", "ON")
                .define("DISABLE_DEBUG_MESSAGES", "ON")
                .define("debug_file", "/dev/null")
                .define("HAVE_LIBARCHIVE", "OFF")
                .define("HAVE_LIBCURL", "OFF")
                .define("HAVE_TIFFIO_H", "OFF")
                .define("GRAPHICS_DISABLED", "ON")
                .define("DISABLED_LEGACY_ENGINE", "ON")
                .define("USE_OPENCL", "OFF")
                .define("OPENMP_BUILD", "OFF")
                .define("BUILD_TESTS", "OFF")
                .define("ENABLE_LTO", "OFF")
                .define("BUILD_PROG", "OFF")
                .define("SW_BUILD", "OFF")
                .define("LEPT_TIFF_RESULT", "FALSE")
                .define("INSTALL_CONFIGS", "ON")
                .define("USE_SYSTEM_ICU", "ON")
                .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags);

            for (key, value) in &additional_defines {
                tesseract_config.define(key, value);
            }

            tesseract_config.build();
        });

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed={}", third_party_dir.display());
        println!("cargo:rerun-if-changed={}", leptonica_dir.display());
        println!("cargo:rerun-if-changed={}", tesseract_dir.display());

        println!(
            "cargo:rustc-link-search=native={}",
            leptonica_lib_dir.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            tesseract_install_dir.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=leptonica");
        println!("cargo:rustc-link-lib=static=tesseract");

        set_os_specific_link_flags();

        println!(
            "cargo:warning=Leptonica include dir: {:?}",
            leptonica_include_dir
        );
        println!("cargo:warning=Leptonica lib dir: {:?}", leptonica_lib_dir);
        println!(
            "cargo:warning=Tesseract install dir: {:?}",
            tesseract_install_dir
        );
    }

    fn get_os_specific_config() -> (String, Vec<(String, String)>) {
        let mut cmake_cxx_flags = String::new();
        let mut additional_defines = Vec::new();

        if cfg!(target_os = "macos") {
            cmake_cxx_flags.push_str("-stdlib=libc++ ");
            cmake_cxx_flags.push_str("-std=c++11 ");
        } else if cfg!(target_os = "linux") {
            cmake_cxx_flags.push_str("-std=c++11 ");
            // Check if we're on a system using clang
            if cfg!(target_env = "musl")
                || env::var("CC")
                    .map(|cc| cc.contains("clang"))
                    .unwrap_or(false)
            {
                cmake_cxx_flags.push_str("-stdlib=libc++ ");
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "clang++".to_string()));
            } else {
                // Assume GCC
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "g++".to_string()));
            }
        } else if cfg!(target_os = "windows") {
            // Windows-specific MSVC flags
            cmake_cxx_flags.push_str("/EHsc /MP ");
            additional_defines.push(("CMAKE_CXX_FLAGS_RELEASE".to_string(), "/MD".to_string()));
            additional_defines.push(("CMAKE_CXX_FLAGS_DEBUG".to_string(), "/MDd".to_string()));
        }

        // Common flags and defines for all platforms
        cmake_cxx_flags.push_str("-DUSE_STD_NAMESPACE ");
        additional_defines.push((
            "CMAKE_POSITION_INDEPENDENT_CODE".to_string(),
            "ON".to_string(),
        ));

        (cmake_cxx_flags, additional_defines)
    }

    fn set_os_specific_link_flags() {
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=c++");
        } else if cfg!(target_os = "linux") {
            if cfg!(target_env = "musl")
                || env::var("CC")
                    .map(|cc| cc.contains("clang"))
                    .unwrap_or(false)
            {
                println!("cargo:rustc-link-lib=c++");
            } else {
                println!("cargo:rustc-link-lib=stdc++");
            }
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=dl");
        } else if cfg!(target_os = "windows") {
            // Additional linker flags are generally not required for Windows,
            // as MSVC automatically links the necessary libraries.
            // However, for some special cases, additions can be made as follows:
            // println!("cargo:rustc-link-lib=user32");
            // println!("cargo:rustc-link-lib=gdi32");
        }

        println!(
            "cargo:rustc-link-search=native={}",
            env::var("OUT_DIR").unwrap()
        );
    }

    fn download_and_extract(target_dir: &Path, url: &str, name: &str) -> PathBuf {
        use reqwest::blocking::Client;
        use zip::ZipArchive;

        fs::create_dir_all(target_dir).expect("Failed to create target directory");

        let client = Client::new();
        let mut response = client.get(url).send().expect("Failed to download archive");
        let mut content = Vec::new();
        response
            .copy_to(&mut content)
            .expect("Failed to read archive content");

        let temp_file = target_dir.join(format!("{}.zip", name));
        fs::write(&temp_file, content).expect("Failed to write archive to file");

        let extract_dir = target_dir.join(name);
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir).expect("Failed to remove existing directory");
        }
        fs::create_dir_all(&extract_dir).expect("Failed to create extraction directory");

        let mut archive = ZipArchive::new(fs::File::open(&temp_file).unwrap()).unwrap();

        // Extract files, ignoring the top-level directory
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let file_path = file.mangled_name();
            let file_path = file_path.to_str().unwrap();

            // Skip the top-level directory
            let path = Path::new(file_path);
            let path = path
                .strip_prefix(path.components().next().unwrap())
                .unwrap();

            if path.as_os_str().is_empty() {
                continue;
            }

            let target_path = extract_dir.join(path);

            if file.is_dir() {
                fs::create_dir_all(target_path).unwrap();
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let mut outfile = fs::File::create(target_path).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        fs::remove_file(temp_file).expect("Failed to remove temporary zip file");

        extract_dir
    }

    fn build_inner<F>(name: &str, install_dir: &Path, build_fn: F)
    where
        F: FnOnce(),
    {
        let lib_name = if cfg!(target_os = "windows") {
            // .lib for Windows
            format!("{}.lib", name)
        } else {
            // .a for Unix
            format!("lib{}.a", name)
        };

        let out_path = install_dir.join("lib").join(&lib_name);
        fs::create_dir_all(out_path.parent().unwrap()).expect("Failed to create output directory");

        println!("Building {} library", name);
        build_fn();
        rename_versioned_libraries_if_needed(name, install_dir);

        println!(
            "cargo:warning=Expected library not found at: {}",
            out_path.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            install_dir.join("lib").display()
        );

        println!("cargo:rustc-link-lib=static={}", name);
    }

    /// Handle versioned library files on Windows (e.g. `leptonica-1.85.1d.lib`) by copying them to the unversioned name.
    fn rename_versioned_libraries_if_needed(name: &str, install_dir: &Path) {
        if cfg!(target_os = "windows") {
            let lib_dir = install_dir.join("lib");
            let expected_lib_name = format!("{}.lib", name);
            let expected_lib_path = lib_dir.join(&expected_lib_name);

            if !expected_lib_path.exists() {
                if let Ok(entries) = fs::read_dir(&lib_dir) {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        if file_name_str.starts_with(name)
                            && file_name_str.ends_with(".lib")
                            && file_name_str != expected_lib_name
                        {
                            let versioned_lib_path = entry.path();
                            println!("Found versioned library: {}", file_name_str);

                            if let Err(e) = fs::copy(&versioned_lib_path, &expected_lib_path) {
                                println!(
                                    "cargo:warning=Failed to copy versioned library {} to {}: {}",
                                    file_name_str, expected_lib_name, e
                                );
                            } else {
                                println!(
                                    "cargo:warning=Copied {} to {}",
                                    file_name_str, expected_lib_name
                                );
                                break; // Stop after renaming the first match.
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    #[cfg(feature = "build-tesseract")]
    build_tesseract::build();
}
