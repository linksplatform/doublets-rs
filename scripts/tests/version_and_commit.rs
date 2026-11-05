use super::{replace_local_lock_version, replace_readme_dependency};

#[test]
fn updates_only_the_released_crates_installation_snippet() {
    let original = "doublets = \"0.12.0\"\nother = \"0.12.0\"\nRust 1.85\n";
    assert_eq!(
        replace_readme_dependency(original, "doublets", "0.13.0"),
        "doublets = \"0.13.0\"\nother = \"0.12.0\"\nRust 1.85\n"
    );
    assert_eq!(
        replace_readme_dependency(original, "absent", "1.0.0"),
        original
    );
}

#[test]
fn updates_local_package_without_changing_registry_versions_or_lock_format() {
    let original = r#"version = 4

[[package]]
name = "doublets"
version = "0.12.0"
dependencies = ["platform-data"]

[[package]]
name = "doublets"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "unchanged"
"#;
    let updated = replace_local_lock_version(original, "doublets", "0.12.1");
    assert_eq!(updated, original.replacen("0.12.0", "0.12.1", 1));
    assert_eq!(
        replace_local_lock_version(&updated, "doublets", "0.12.1"),
        updated
    );
    assert_eq!(
        replace_local_lock_version(original, "missing", "1.0.0"),
        original
    );
}
