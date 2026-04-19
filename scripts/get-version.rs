#!/usr/bin/env rust-script
//! Get the current version from Cargo.toml
//!
//! This script reads the version from Cargo.toml and outputs it
//! for use in GitHub Actions.
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/get-version.rs [--rust-root <path>]
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - version: The current version from Cargo.toml
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use regex::Regex;

fn get_rust_root() -> String {
    // Check CLI arguments
    let args: Vec<String> = env::args().collect();
    if let Some(idx) = args.iter().position(|a| a == "--rust-root") {
        if let Some(root) = args.get(idx + 1) {
            return root.clone();
        }
    }

    // Check environment variable
    if let Ok(root) = env::var("RUST_ROOT") {
        if !root.is_empty() {
            return root;
        }
    }

    // Auto-detect
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

fn is_workspace_toml(cargo_toml_path: &str) -> bool {
    fs::read_to_string(cargo_toml_path)
        .map(|c| c.contains("[workspace]"))
        .unwrap_or(false)
}

fn has_publish_false(cargo_toml_path: &str) -> bool {
    fs::read_to_string(cargo_toml_path)
        .map(|c| {
            let re = Regex::new(r#"(?m)^publish\s*=\s*false"#).unwrap();
            re.is_match(&c)
        })
        .unwrap_or(false)
}

fn parse_workspace_members(cargo_toml_path: &str) -> Vec<String> {
    let content = match fs::read_to_string(cargo_toml_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let re = Regex::new(r#"(?s)members\s*=\s*\[(.*?)\]"#).unwrap();
    if let Some(caps) = re.captures(&content) {
        let member_re = Regex::new(r#""([^"]+)""#).unwrap();
        member_re.captures_iter(caps.get(1).unwrap().as_str())
            .map(|c| c.get(1).unwrap().as_str().to_string())
            .collect()
    } else {
        vec![]
    }
}

fn resolve_package_cargo_toml(rust_root: &str) -> String {
    let root_toml = get_cargo_toml_path(rust_root);
    if !is_workspace_toml(&root_toml) {
        return root_toml;
    }

    eprintln!("Detected workspace Cargo.toml, searching for publishable member...");
    let members = parse_workspace_members(&root_toml);
    let base = if rust_root == "." { String::new() } else { format!("{}/", rust_root) };

    for member in &members {
        let member_toml = format!("{}{}/Cargo.toml", base, member);
        if Path::new(&member_toml).exists() && !has_publish_false(&member_toml) {
            eprintln!("Using publishable workspace member: {} ({})", member, member_toml);
            return member_toml;
        }
    }

    eprintln!("Warning: No publishable workspace member found, falling back to root Cargo.toml");
    root_toml
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

fn main() {
    let rust_root = get_rust_root();
    let cargo_toml = resolve_package_cargo_toml(&rust_root);

    match get_current_version(&cargo_toml) {
        Ok(version) => {
            println!("Current version: {}", version);
            set_output("version", &version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
