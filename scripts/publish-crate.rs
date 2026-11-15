#!/usr/bin/env rust-script
//! Publish only the repository's public package using the short-lived OIDC token.
//! Usage: rust-script scripts/publish-crate.rs

use std::{env, fs::OpenOptions, io::Write, process::{Command, exit}};

fn set_output(value: &str) {
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "publish_result={value}");
        }
    }
}

fn main() {
    if env::var("CARGO_REGISTRY_TOKEN").unwrap_or_default().is_empty() {
        eprintln!("::error::Missing trusted publishing token. Configure the doublets trusted publisher on crates.io for linksplatform/doublets-rs, workflow release.yml, then authenticate with rust-lang/crates-io-auth-action before publishing.");
        set_output("auth_failed");
        exit(1);
    }

    // Explicit selection prevents new workspace members from being released.
    // Cargo reads the token from the environment; never place it in arguments.
    let status = Command::new("cargo")
        .args(["publish", "--package", "doublets", "--locked", "--allow-dirty"])
        .status()
        .expect("Failed to execute cargo publish");
    if !status.success() {
        set_output("failed");
        exit(status.code().unwrap_or(1));
    }
    set_output("success");
}
