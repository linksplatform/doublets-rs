#!/usr/bin/env rust-script
//! Check documentation workflow badges.
//!
//! This prevents stale GitHub Actions badges from pointing at deleted workflows
//! or showing pull request status when the README is meant to show `main`.
//!
//! Usage: rust-script scripts/check-readme-badges.rs

use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

const REPO_PREFIX: &str = "https://github.com/linksplatform/doublets-rs/";
const LEGACY_WORKFLOW_PREFIX: &str = "https://github.com/linksplatform/doublets-rs/workflows/";
const WORKFLOW_FILE_PREFIX: &str =
    "https://github.com/linksplatform/doublets-rs/actions/workflows/";
const EXCLUDED_DIRS: &[&str] = &[".git", "target", "docs/case-studies"];

struct Violation {
    file: String,
    line: usize,
    message: String,
}

fn github_urls(line: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = line;

    while let Some(start) = rest.find(REPO_PREFIX) {
        let candidate = &rest[start..];
        let end = candidate
            .find(|c: char| c == ')' || c == ']' || c == '"' || c == '\'' || c.is_whitespace())
            .unwrap_or(candidate.len());

        urls.push(candidate[..end].to_string());
        rest = &candidate[end..];
    }

    urls
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches("./")
        .replace('\\', "/")
}

fn should_skip_dir(path: &Path) -> bool {
    let path = normalized_path(path);
    EXCLUDED_DIRS
        .iter()
        .any(|excluded| path == *excluded || path.starts_with(&format!("{excluded}/")))
}

fn collect_markdown_files(dir: &Path, files: &mut Vec<String>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        eprintln!("Failed to read {}: {}", dir.display(), error);
        exit(1);
    });

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            eprintln!("Failed to read directory entry: {error}");
            exit(1);
        });
        let path = entry.path();

        if path.is_dir() {
            if !should_skip_dir(&path) {
                collect_markdown_files(&path, files);
            }
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "md") {
            files.push(normalized_path(&path));
        }
    }
}

fn validate_badge_url(file: &str, line: usize, url: &str, violations: &mut Vec<Violation>) {
    if !url.contains("badge.svg") {
        return;
    }

    if url.starts_with(LEGACY_WORKFLOW_PREFIX) {
        violations.push(Violation {
            file: file.to_string(),
            line,
            message: format!(
                "legacy workflow-name badge URL `{url}` should use `actions/workflows/<workflow-file>/badge.svg?branch=main`"
            ),
        });
        return;
    }

    if let Some(workflow_url) = url.strip_prefix(WORKFLOW_FILE_PREFIX) {
        let Some((workflow_file, _)) = workflow_url.split_once("/badge.svg") else {
            violations.push(Violation {
                file: file.to_string(),
                line,
                message: format!("workflow badge URL `{url}` is missing `/badge.svg`"),
            });
            return;
        };

        let workflow_path = Path::new(".github/workflows").join(workflow_file);
        if !workflow_path.is_file() {
            violations.push(Violation {
                file: file.to_string(),
                line,
                message: format!(
                    "workflow badge URL `{url}` points to missing workflow file `{}`",
                    workflow_path.display()
                ),
            });
        }

        if !url.contains("/badge.svg?branch=main") {
            violations.push(Violation {
                file: file.to_string(),
                line,
                message: format!("workflow badge URL `{url}` must be scoped with `?branch=main`"),
            });
        }
    }
}

fn validate_line(file: &str, line_number: usize, line: &str, violations: &mut Vec<Violation>) {
    if line.contains("actions?query=workflow%3A") {
        violations.push(Violation {
            file: file.to_string(),
            line: line_number,
            message:
                "legacy workflow query link should point to `actions/workflows/<workflow-file>`"
                    .to_string(),
        });
    }

    for url in github_urls(line) {
        validate_badge_url(file, line_number, &url, violations);
    }
}

fn main() {
    println!("\nChecking documentation workflow badges...\n");

    let mut violations = Vec::new();

    let docs_to_check: Vec<String> = match env::args().skip(1).collect::<Vec<_>>() {
        args if args.is_empty() => {
            let mut files = Vec::new();
            collect_markdown_files(Path::new("."), &mut files);
            files
        }
        args => args,
    };

    for file in docs_to_check {
        let path = Path::new(&file);
        if !path.exists() {
            continue;
        }

        let content = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("Failed to read {}: {}", path.display(), error);
            exit(1);
        });

        for (index, line) in content.lines().enumerate() {
            validate_line(&file, index + 1, line, &mut violations);
        }
    }

    if violations.is_empty() {
        println!("Documentation workflow badges are current and branch-scoped\n");
        return;
    }

    println!("Found stale documentation workflow badges:\n");
    for violation in &violations {
        println!(
            "  {}:{}: {}",
            violation.file, violation.line, violation.message
        );
    }
    println!();

    exit(1);
}
