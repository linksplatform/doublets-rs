#!/usr/bin/env rust-script
//! Check that documentation matches the crate version and Rust toolchain.
//!
//! Usage: rust-script scripts/check-toolchain-docs.rs

use std::fs;
use std::path::Path;
use std::process::exit;

struct Violation {
    file: &'static str,
    line: Option<usize>,
    message: String,
}

fn read(path: &'static str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|error| {
        eprintln!("Failed to read {path}: {error}");
        exit(1);
    })
}

fn quoted_value(file: &'static str, contents: &str, key: &str) -> String {
    for line in contents.lines() {
        let line = line.trim();
        let Some(value) = line.strip_prefix(&format!("{key} = ")) else {
            continue;
        };

        if let Some(value) = value.strip_prefix('"').and_then(|value| value.split('"').next()) {
            return value.to_string();
        }
    }

    eprintln!("Failed to find `{key}` in {file}");
    exit(1);
}

fn first_line_containing(contents: &str, needle: &str) -> Option<usize> {
    contents
        .lines()
        .position(|line| line.contains(needle))
        .map(|index| index + 1)
}

fn require_contains(
    violations: &mut Vec<Violation>,
    file: &'static str,
    contents: &str,
    needle: &str,
    message: &str,
) {
    if !contents.contains(needle) {
        violations.push(Violation {
            file,
            line: None,
            message: message.to_string(),
        });
    }
}

fn reject_contains(
    violations: &mut Vec<Violation>,
    file: &'static str,
    contents: &str,
    needle: &str,
    message: &str,
) {
    if let Some(line) = first_line_containing(contents, needle) {
        violations.push(Violation {
            file,
            line: Some(line),
            message: message.to_string(),
        });
    }
}

fn main() {
    println!("\nChecking documented Rust toolchain and crate version...\n");

    let cargo_toml = read("doublets/Cargo.toml");
    let crate_version = quoted_value("doublets/Cargo.toml", &cargo_toml, "version");
    let rust_version = quoted_value("doublets/Cargo.toml", &cargo_toml, "rust-version");

    let rust_toolchain = read("rust-toolchain.toml");
    let toolchain_channel = quoted_value("rust-toolchain.toml", &rust_toolchain, "channel");

    let readme = read("README.md");
    let contributing = read("CONTRIBUTING.md");

    let mut violations = Vec::new();

    require_contains(
        &mut violations,
        "README.md",
        &readme,
        &format!("doublets = \"{crate_version}\""),
        &format!("README installation snippet must use doublets = \"{crate_version}\""),
    );
    require_contains(
        &mut violations,
        "README.md",
        &readme,
        &format!("stable Rust {rust_version} or newer"),
        &format!("README must document stable Rust {rust_version} or newer"),
    );
    require_contains(
        &mut violations,
        "README.md",
        &readme,
        "rustup toolchain install stable",
        "README must show stable toolchain installation",
    );
    reject_contains(
        &mut violations,
        "README.md",
        &readme,
        "0.1.0-pre",
        "README contains the obsolete pre-release dependency version",
    );
    reject_contains(
        &mut violations,
        "README.md",
        &readme,
        "requires nightly Rust",
        "README still says nightly Rust is required",
    );
    reject_contains(
        &mut violations,
        "README.md",
        &readme,
        "rustup default nightly",
        "README still instructs users to switch to nightly Rust",
    );

    require_contains(
        &mut violations,
        "CONTRIBUTING.md",
        &contributing,
        &format!("channel = \"{toolchain_channel}\""),
        &format!("CONTRIBUTING.md must document the `{toolchain_channel}` toolchain channel"),
    );
    require_contains(
        &mut violations,
        "CONTRIBUTING.md",
        &contributing,
        &format!("rust-version = \"{rust_version}\""),
        &format!("CONTRIBUTING.md must document rust-version = \"{rust_version}\""),
    );
    reject_contains(
        &mut violations,
        "CONTRIBUTING.md",
        &contributing,
        "specific nightly toolchain",
        "CONTRIBUTING.md still describes the project as nightly-only",
    );
    reject_contains(
        &mut violations,
        "CONTRIBUTING.md",
        &contributing,
        "nightly-2022-08-22",
        "CONTRIBUTING.md still references the old nightly toolchain",
    );

    if violations.is_empty() {
        println!("Documentation matches doublets/Cargo.toml and rust-toolchain.toml\n");
        return;
    }

    println!("Found stale toolchain documentation:\n");
    for violation in &violations {
        match violation.line {
            Some(line) => println!("  {}:{}: {}", violation.file, line, violation.message),
            None => println!("  {}: {}", violation.file, violation.message),
        }
    }
    println!();

    exit(1);
}
