#!/usr/bin/env rust-script
//! Publish package to crates.io
//!
//! This script publishes the Rust package to crates.io and handles
//! the case where the version already exists.
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/publish-crate.rs [--token <token>] [--rust-root <path>]
//!
//! Environment variables (checked in order of priority):
//!   - CARGO_REGISTRY_TOKEN: Cargo's native crates.io token (preferred)
//!   - CARGO_TOKEN: Alternative token name for backwards compatibility
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - publish_result: 'success', 'already_exists', or 'failed'
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, exit};
use regex::Regex;

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    None
}

fn get_rust_root() -> String {
    if let Some(root) = get_arg("rust-root") {
        eprintln!("Using explicitly configured Rust root: {}", root);
        return root;
    }

    if let Ok(root) = env::var("RUST_ROOT") {
        if !root.is_empty() {
            eprintln!("Using environment configured Rust root: {}", root);
            return root;
        }
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

fn needs_cd(rust_root: &str) -> bool {
    rust_root != "."
}

fn set_output(key: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&output_file) {
            let _ = writeln!(file, "{}={}", key, value);
        }
    }
    println!("Output: {}={}", key, value);
}

fn get_package_info(cargo_toml_path: &str) -> Result<(String, String), String> {
    let content = fs::read_to_string(cargo_toml_path)
        .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let name_re = Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).unwrap();
    let version_re = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();

    let name = name_re
        .captures(&content)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .ok_or_else(|| format!("Could not find name in {}", cargo_toml_path))?;

    let version = version_re
        .captures(&content)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .ok_or_else(|| format!("Could not find version in {}", cargo_toml_path))?;

    Ok((name, version))
}

fn main() {
    let rust_root = get_rust_root();
    let cargo_toml = get_cargo_toml_path(&rust_root);

    // Get token from CLI arg, then env vars
    let token = get_arg("token")
        .or_else(|| env::var("CARGO_REGISTRY_TOKEN").ok().filter(|s| !s.is_empty()))
        .or_else(|| env::var("CARGO_TOKEN").ok().filter(|s| !s.is_empty()));

    let (name, version) = match get_package_info(&cargo_toml) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    println!("Package: {}@{}", name, version);

    if name == "example-sum-package-name" {
        println!("Skipping publish: package name is the template default 'example-sum-package-name'");
        println!("Rename the package in Cargo.toml before publishing to crates.io");
        set_output("publish_result", "skipped");
        return;
    }

    println!();
    println!("=== Attempting to publish to crates.io ===");

    if token.is_none() {
        println!("::warning::Neither CARGO_REGISTRY_TOKEN nor CARGO_TOKEN is set, attempting publish without explicit token");
        println!();
        println!("To fix this, ensure one of the following secrets is configured:");
        println!("  - CARGO_REGISTRY_TOKEN (Cargo's native env var, preferred)");
        println!("  - CARGO_TOKEN (alternative for backwards compatibility)");
        println!();
        println!("For organization secrets, you may need to map the secret name in your workflow:");
        println!("  env:");
        println!("    CARGO_REGISTRY_TOKEN: ${{{{ secrets.CARGO_TOKEN }}}}");
        println!();
    } else {
        println!("Using provided authentication token");
    }

    // Build the cargo publish command
    let mut cmd = Command::new("cargo");
    cmd.arg("publish").arg("--allow-dirty");

    if let Some(t) = &token {
        cmd.arg("--token").arg(t);
    }

    // For multi-language repos, change to the rust directory
    if needs_cd(&rust_root) {
        cmd.current_dir(&rust_root);
    }

    let output = cmd.output().expect("Failed to execute cargo publish");

    if output.status.success() {
        println!("Successfully published {}@{} to crates.io", name, version);
        set_output("publish_result", "success");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        if combined.contains("already uploaded") || combined.contains("already exists") {
            eprintln!();
            eprintln!("=== VERSION ALREADY PUBLISHED ===");
            eprintln!();
            eprintln!("Version {} already exists on crates.io.", version);
            eprintln!("The release pipeline must always publish a version greater than what is already published.");
            eprintln!("This indicates a bug in version bumping: the pipeline should have computed a new, unpublished version.");
            eprintln!();
            set_output("publish_result", "already_exists");
            exit(1);
        } else if combined.contains("non-empty token")
            || combined.contains("please provide a")
            || combined.contains("unauthorized")
            || combined.contains("authentication")
        {
            eprintln!();
            eprintln!("=== AUTHENTICATION FAILURE ===");
            eprintln!();
            eprintln!("Failed to publish due to missing or invalid authentication token.");
            eprintln!();
            eprintln!("SOLUTION: Configure one of these secrets in your repository or organization:");
            eprintln!("  1. CARGO_REGISTRY_TOKEN - Cargo's native environment variable (preferred)");
            eprintln!("  2. CARGO_TOKEN - Alternative name for backwards compatibility");
            eprintln!();
            eprintln!("If using organization secrets with a different name, map it in your workflow:");
            eprintln!("  - name: Publish to Crates.io");
            eprintln!("    env:");
            eprintln!("      CARGO_REGISTRY_TOKEN: ${{{{ secrets.YOUR_SECRET_NAME }}}}");
            eprintln!();
            eprintln!("See: https://doc.rust-lang.org/cargo/reference/publishing.html");
            eprintln!();
            set_output("publish_result", "auth_failed");
            exit(1);
        } else {
            eprintln!("Failed to publish for unknown reason");
            eprintln!("{}", combined);
            set_output("publish_result", "failed");
            exit(1);
        }
    }
}
