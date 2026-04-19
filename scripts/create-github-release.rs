#!/usr/bin/env rust-script
//! Create GitHub Release from CHANGELOG.md
//!
//! Automatically includes crates.io and docs.rs badges in release notes
//! when the crate name can be detected from Cargo.toml.
//!
//! Usage: rust-script scripts/create-github-release.rs --release-version <version> --repository <repository> [--tag-prefix <prefix>] [--release-label <label>]
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio, exit};
use regex::Regex;
use serde::Serialize;

fn get_rust_root() -> String {
    if let Some(root) = get_arg("rust-root") {
        return root;
    }

    if Path::new("./Cargo.toml").exists() {
        return ".".to_string();
    }

    if Path::new("./rust/Cargo.toml").exists() {
        return "rust".to_string();
    }

    ".".to_string()
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

fn get_crate_name_from_toml(cargo_toml_path: &str) -> Option<String> {
    let content = fs::read_to_string(cargo_toml_path).ok()?;
    let re = Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).ok()?;
    re.captures(&content).map(|c| c.get(1).unwrap().as_str().to_string())
}

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn get_changelog_for_version(version: &str) -> String {
    let changelog_path = "CHANGELOG.md";

    if !Path::new(changelog_path).exists() {
        return format!("Release v{}", version);
    }

    let content = match fs::read_to_string(changelog_path) {
        Ok(c) => c,
        Err(_) => return format!("Release v{}", version),
    };

    // Find the section for this version
    let escaped_version = regex::escape(version);
    let header_pattern = format!(r"(?m)^## \[{}\]", escaped_version);
    let header_re = Regex::new(&header_pattern).unwrap();

    if let Some(m) = header_re.find(&content) {
        let after_header = &content[m.end()..];
        let body_start = after_header.find('\n').map_or(after_header.len(), |i| i + 1);
        let body = &after_header[body_start..];

        let next_section_re = Regex::new(r"(?m)^## \[").unwrap();
        let section_body = if let Some(next) = next_section_re.find(body) {
            &body[..next.start()]
        } else {
            body
        };

        let trimmed = section_body.trim();
        if trimmed.is_empty() {
            format!("Release v{}", version)
        } else {
            trimmed.to_string()
        }
    } else {
        format!("Release v{}", version)
    }
}

#[derive(Serialize)]
struct ReleasePayload {
    tag_name: String,
    name: String,
    body: String,
}

fn main() {
    let version = match get_arg("release-version") {
        Some(v) => v,
        None => {
            eprintln!("Error: Missing required argument --release-version");
            eprintln!("Usage: rust-script scripts/create-github-release.rs --release-version <version> --repository <repository>");
            exit(1);
        }
    };

    let repository = match get_arg("repository") {
        Some(r) => r,
        None => {
            eprintln!("Error: Missing required argument --repository");
            eprintln!("Usage: rust-script scripts/create-github-release.rs --release-version <version> --repository <repository>");
            exit(1);
        }
    };

    let tag_prefix = get_arg("tag-prefix").unwrap_or_else(|| "v".to_string());
    let release_label = get_arg("release-label");
    let crates_io_url = get_arg("crates-io-url");

    let rust_root = get_rust_root();
    let cargo_toml = resolve_package_cargo_toml(&rust_root);

    if let Some(ref crate_name) = get_crate_name_from_toml(&cargo_toml) {
        if crate_name == "example-sum-package-name" {
            println!("Skipping GitHub release: package name is the template default 'example-sum-package-name'");
            println!("Rename the package in Cargo.toml before creating releases");
            return;
        }
    }

    let tag = format!("{}{}", tag_prefix, version);
    println!("Creating GitHub release for {}...", tag);

    let mut release_notes = get_changelog_for_version(&version);
    if let Some(crate_name) = get_crate_name_from_toml(&cargo_toml) {
        let badges = format!(
            "[![Crates.io](https://img.shields.io/crates/v/{}?label=crates.io)](https://crates.io/crates/{}/{}) [![Docs.rs](https://docs.rs/{}/badge.svg)](https://docs.rs/{}/{})",
            crate_name, crate_name, version, crate_name, crate_name, version
        );
        release_notes = format!("{}\n\n{}", badges, release_notes);
    }

    // Add explicit crates.io link if provided (overrides auto-detected)
    if let Some(url) = crates_io_url {
        release_notes = format!("{}\n\n{}", url, release_notes);
    }

    // Build release name with optional label for multi-language repos
    let release_name = match &release_label {
        Some(label) => format!("{}{} ({})", tag_prefix, version, label),
        None => format!("{}{}", tag_prefix, version),
    };

    // Create release using GitHub API with JSON input
    let payload = ReleasePayload {
        tag_name: tag.clone(),
        name: release_name,
        body: release_notes,
    };

    let payload_json = serde_json::to_string(&payload).expect("Failed to serialize payload");

    let mut child = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/releases", repository),
            "-X",
            "POST",
            "--input",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute gh command");

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(payload_json.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on gh command");

    if output.status.success() {
        println!("Created GitHub release: {}", tag);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stderr, stdout);
        if combined.contains("already exists") || combined.contains("already_exists")
            || combined.contains("Validation Failed")
        {
            println!("Release {} already exists, skipping", tag);
        } else {
            eprintln!("Error creating release: {}", stderr);
            exit(1);
        }
    }
}
