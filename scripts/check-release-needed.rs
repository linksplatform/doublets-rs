#!/usr/bin/env rust-script
//! Check if a release is needed based on changelog fragments and version state
//!
//! This script checks:
//! 1. If there are changelog fragments to process
//! 2. If the current version has already been published to crates.io
//!
//! IMPORTANT: This script checks crates.io (the source of truth for Rust packages),
//! NOT git tags. This is critical because:
//! - Git tags can exist without the package being published
//! - GitHub releases create tags but don't publish to crates.io
//! - Only crates.io publication means users can actually install the package
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/check-release-needed.rs [--rust-root <path>]
//!
//! Environment variables:
//!   - HAS_FRAGMENTS: 'true' if changelog fragments exist (from get-bump-type.rs)
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - should_release: 'true' if a release should be created
//!   - skip_bump: 'true' if version bump should be skipped (version not yet released)
//!   - max_published_version: the highest non-yanked version on crates.io (for downstream use)
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ureq = "2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use regex::Regex;
use serde::Deserialize;

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn get_rust_root() -> String {
    if let Some(root) = get_arg("rust-root") {
        eprintln!("Using explicitly configured Rust root: {}", root);
        return root;
    }

    if Path::new("./Cargo.toml").exists() {
        eprintln!("Detected single-language repository (Cargo.toml in root)");
        return ".".to_string();
    }

    if Path::new("./rust/Cargo.toml").exists() {
        eprintln!("Detected multi-language repository (Cargo.toml in rust/)");
        return "rust".to_string();
    }

    eprintln!("Error: Could not find Cargo.toml in expected locations");
    exit(1);
}

fn get_cargo_toml_path(rust_root: &str) -> String {
    if rust_root == "." {
        "./Cargo.toml".to_string()
    } else {
        format!("{}/Cargo.toml", rust_root)
    }
}

fn set_output(key: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Err(e) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}={}", key, value)
            })
        {
            eprintln!("Warning: Could not write to GITHUB_OUTPUT: {}", e);
        }
    }
    println!("Output: {}={}", key, value);
}

fn get_current_version(cargo_toml_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(cargo_toml_path)
        .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let re = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();

    if let Some(caps) = re.captures(&content) {
        Ok(caps.get(1).unwrap().as_str().to_string())
    } else {
        Err(format!("Could not find version in {}", cargo_toml_path))
    }
}

fn get_crate_name(cargo_toml_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(cargo_toml_path)
        .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let re = Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).unwrap();

    if let Some(caps) = re.captures(&content) {
        Ok(caps.get(1).unwrap().as_str().to_string())
    } else {
        Err(format!("Could not find name in {}", cargo_toml_path))
    }
}

#[derive(Deserialize)]
struct CratesIoVersion {
    version: Option<CratesIoVersionInfo>,
}

#[derive(Deserialize)]
struct CratesIoVersionInfo {
    #[allow(dead_code)]
    num: String,
}

#[derive(Deserialize)]
struct CratesIoCrate {
    versions: Option<Vec<CratesIoVersionEntry>>,
}

#[derive(Deserialize)]
struct CratesIoVersionEntry {
    num: String,
    yanked: bool,
}

fn check_version_on_crates_io(crate_name: &str, version: &str) -> bool {
    let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, version);

    match ureq::get(&url)
        .set("User-Agent", "rust-script-check-release")
        .call()
    {
        Ok(response) => {
            if response.status() == 200 {
                if let Ok(body) = response.into_string() {
                    if let Ok(data) = serde_json::from_str::<CratesIoVersion>(&body) {
                        return data.version.is_some();
                    }
                }
            }
            false
        }
        Err(ureq::Error::Status(404, _)) => {
            false
        }
        Err(e) => {
            eprintln!("Warning: Could not check crates.io: {}", e);
            false
        }
    }
}

fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('-').next()?.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn get_max_published_version(crate_name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    match ureq::get(&url)
        .set("User-Agent", "rust-script-check-release")
        .call()
    {
        Ok(response) => {
            if response.status() == 200 {
                if let Ok(body) = response.into_string() {
                    if let Ok(data) = serde_json::from_str::<CratesIoCrate>(&body) {
                        if let Some(versions) = data.versions {
                            let mut max_version: Option<(u32, u32, u32, String)> = None;
                            for v in &versions {
                                if v.yanked {
                                    continue;
                                }
                                if let Some(parsed) = parse_semver(&v.num) {
                                    match &max_version {
                                        None => {
                                            max_version = Some((parsed.0, parsed.1, parsed.2, v.num.clone()));
                                        }
                                        Some(current) => {
                                            if parsed > (current.0, current.1, current.2) {
                                                max_version = Some((parsed.0, parsed.1, parsed.2, v.num.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                            return max_version.map(|v| v.3);
                        }
                    }
                }
            }
            None
        }
        Err(ureq::Error::Status(404, _)) => None,
        Err(e) => {
            eprintln!("Warning: Could not query crates.io for versions: {}", e);
            None
        }
    }
}

fn main() {
    let rust_root = get_rust_root();
    let cargo_toml = get_cargo_toml_path(&rust_root);

    let has_fragments = env::var("HAS_FRAGMENTS")
        .map(|v| v == "true")
        .unwrap_or(false);

    let crate_name = match get_crate_name(&cargo_toml) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let current_version = match get_current_version(&cargo_toml) {
        Ok(version) => version,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let max_published = get_max_published_version(&crate_name);
    if let Some(ref max_ver) = max_published {
        println!("Max published version on crates.io: {}", max_ver);
        set_output("max_published_version", max_ver);
    } else {
        println!("No versions published on crates.io yet (or crate not found)");
        set_output("max_published_version", "");
    }

    if !has_fragments {
        let is_published = check_version_on_crates_io(&crate_name, &current_version);

        println!(
            "Crate: {}, Version: {}, Published on crates.io: {}",
            crate_name, current_version, is_published
        );

        if is_published {
            println!(
                "No changelog fragments and v{} already published on crates.io",
                current_version
            );
            set_output("should_release", "false");
        } else {
            println!(
                "No changelog fragments but v{} not yet published to crates.io",
                current_version
            );
            set_output("should_release", "true");
            set_output("skip_bump", "true");
        }
    } else {
        println!("Found changelog fragments, proceeding with release");
        set_output("should_release", "true");
        set_output("skip_bump", "false");
    }
}
