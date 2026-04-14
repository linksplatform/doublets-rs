#!/usr/bin/env rust-script
//! Detect code changes for CI/CD pipeline
//!
//! This script detects what types of files have changed between two commits
//! and outputs the results for use in GitHub Actions workflow conditions.
//!
//! Key behavior:
//! - For PRs: compares PR head against base branch
//! - For pushes: compares HEAD against HEAD^
//! - Excludes certain folders and file types from "code changes" detection
//!
//! Excluded from code changes (don't require changelog fragments):
//! - Markdown files (*.md) in any folder
//! - changelog.d/ folder (changelog fragments)
//! - docs/ folder (documentation)
//! - experiments/ folder (experimental scripts)
//! - examples/ folder (example scripts)
//!
//! Usage: rust-script scripts/detect-code-changes.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - GITHUB_EVENT_NAME: 'pull_request' or 'push'
//!   - GITHUB_BASE_SHA: Base commit SHA for PR
//!   - GITHUB_HEAD_SHA: Head commit SHA for PR
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - rs-changed: 'true' if any .rs files changed
//!   - toml-changed: 'true' if any .toml files changed
//!   - mjs-changed: 'true' if any .mjs files changed
//!   - docs-changed: 'true' if any .md files changed
//!   - workflow-changed: 'true' if any .github/workflows/ files changed
//!   - any-code-changed: 'true' if any code files changed (excludes docs, changelog.d, experiments, examples)
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use regex::Regex;

fn exec(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                eprintln!("Error executing {} {:?}", command, args);
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute {} {:?}: {}", command, args, e);
            String::new()
        }
    }
}

fn exec_silent(command: &str, args: &[&str]) {
    let _ = Command::new(command)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn set_output(name: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&output_file) {
            let _ = writeln!(file, "{}={}", name, value);
        }
    }
    println!("{}={}", name, value);
}

fn get_changed_files() -> Vec<String> {
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_else(|_| "local".to_string());

    if event_name == "pull_request" {
        let base_sha = env::var("GITHUB_BASE_SHA").ok();
        let head_sha = env::var("GITHUB_HEAD_SHA").ok();

        if let (Some(base), Some(head)) = (base_sha, head_sha) {
            println!("Comparing PR: {}...{}", base, head);

            // Ensure we have the base commit
            exec_silent("git", &["fetch", "origin", &base]);

            let output = exec("git", &["diff", "--name-only", &base, &head]);
            if !output.is_empty() {
                return output.lines().filter(|s| !s.is_empty()).map(String::from).collect();
            }
        }
    }

    // For push events or fallback
    println!("Comparing HEAD^ to HEAD");
    let output = exec("git", &["diff", "--name-only", "HEAD^", "HEAD"]);

    if output.is_empty() {
        // If HEAD^ doesn't exist (first commit), list all files in HEAD
        println!("HEAD^ not available, listing all files in HEAD");
        let output = exec("git", &["ls-tree", "--name-only", "-r", "HEAD"]);
        return output.lines().filter(|s| !s.is_empty()).map(String::from).collect();
    }

    output.lines().filter(|s| !s.is_empty()).map(String::from).collect()
}

fn is_excluded_from_code_changes(file_path: &str) -> bool {
    // Exclude markdown files in any folder
    if file_path.ends_with(".md") {
        return true;
    }

    // Exclude specific folders from code changes
    let excluded_folders = ["changelog.d/", "docs/", "experiments/", "examples/"];

    for folder in &excluded_folders {
        if file_path.starts_with(folder) {
            return true;
        }
    }

    false
}

fn main() {
    println!("Detecting file changes for CI/CD...\n");

    let changed_files = get_changed_files();

    println!("Changed files:");
    if changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &changed_files {
            println!("  {}", file);
        }
    }
    println!();

    // Detect .rs file changes (Rust source)
    let rs_changed = changed_files.iter().any(|f| f.ends_with(".rs"));
    set_output("rs-changed", if rs_changed { "true" } else { "false" });

    // Detect .toml file changes (Cargo.toml, Cargo.lock, etc.)
    let toml_changed = changed_files.iter().any(|f| f.ends_with(".toml"));
    set_output("toml-changed", if toml_changed { "true" } else { "false" });

    // Detect .mjs file changes (scripts)
    let mjs_changed = changed_files.iter().any(|f| f.ends_with(".mjs"));
    set_output("mjs-changed", if mjs_changed { "true" } else { "false" });

    // Detect documentation changes (any .md file)
    let docs_changed = changed_files.iter().any(|f| f.ends_with(".md"));
    set_output("docs-changed", if docs_changed { "true" } else { "false" });

    // Detect workflow changes
    let workflow_changed = changed_files.iter().any(|f| f.starts_with(".github/workflows/"));
    set_output("workflow-changed", if workflow_changed { "true" } else { "false" });

    // Detect code changes (excluding docs, changelog.d, experiments, examples folders, and markdown files)
    let code_changed_files: Vec<&String> = changed_files
        .iter()
        .filter(|f| !is_excluded_from_code_changes(f))
        .collect();

    println!("\nFiles considered as code changes:");
    if code_changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &code_changed_files {
            println!("  {}", file);
        }
    }
    println!();

    // Check if any code files changed (.rs, .toml, .mjs, .yml, .yaml, or workflow files)
    let code_pattern = Regex::new(r"\.(rs|toml|mjs|js|yml|yaml)$|\.github/workflows/").unwrap();
    let code_changed = code_changed_files.iter().any(|f| code_pattern.is_match(f));
    set_output("any-code-changed", if code_changed { "true" } else { "false" });

    println!("\nChange detection completed.");
}
