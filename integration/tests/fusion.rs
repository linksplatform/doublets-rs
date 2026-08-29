//! Asserts that a decorator stack compiles away entirely.
//!
//! `src/bins/fusion-probe.rs` exports, for each intercepted operation, a `bare_*`
//! function driving a plain store and a `composed_*` function driving a deep decorator
//! stack that forwards that operation untouched. This test builds the probe in release
//! mode and compares the two disassembled bodies instruction by instruction: if any
//! decorator layer survived as a real call, the bodies differ.
//!
//! The test skips itself — loudly — when it cannot do its job (no disassembler, an
//! unparsable object format, a failed build). Set `DOUBLETS_SKIP_FUSION_TEST=1` to skip
//! it unconditionally.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

/// Operations whose `bare_*` and `composed_*` bodies must be identical.
const OPERATIONS: &[&str] = &["create", "count", "each"];

/// Compiler flags the probe build must *not* inherit from whatever harness is running
/// this test.
///
/// `cargo llvm-cov` exports `-C instrument-coverage` through `CARGO_ENCODED_RUSTFLAGS`,
/// and a coverage counter is emitted per source region, so an inlined decorator layer
/// still leaves a `lock incq` behind even though it produced no call. The probe wants a
/// plain release build, so the flags are stripped rather than worked around.
const INHERITED_FLAGS: &[&str] = &[
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "LLVM_PROFILE_FILE",
];

macro_rules! skip {
    ($($arg:tt)*) => {{
        println!("note: fusion check skipped: {}", format_args!($($arg)*));
        return;
    }};
}

#[test]
fn a_decorator_stack_emits_the_same_code_as_the_bare_store() {
    if std::env::var_os("DOUBLETS_SKIP_FUSION_TEST").is_some() {
        skip!("DOUBLETS_SKIP_FUSION_TEST is set");
    }

    let Some(disassembler) = find_disassembler() else {
        skip!("neither `llvm-objdump` nor `objdump` is available");
    };

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the integration crate always has a parent directory")
        .to_path_buf();
    let target_dir = workspace.join("target").join("fusion-probe");

    let cargo = std::env::var_os("CARGO").map_or_else(|| PathBuf::from("cargo"), PathBuf::from);
    let mut build = Command::new(cargo);
    build
        .current_dir(&workspace)
        .args([
            "build",
            "--release",
            "-p",
            "integration",
            "--bin",
            "fusion-probe",
        ])
        .arg("--target-dir")
        .arg(&target_dir);
    for variable in INHERITED_FLAGS {
        build.env_remove(variable);
    }
    let build = build
        .output()
        .expect("`cargo build` must be spawnable from a test");

    assert!(
        build.status.success(),
        "building the fusion probe failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let probe = ["fusion-probe", "fusion-probe.exe"]
        .into_iter()
        .map(|name| target_dir.join("release").join(name))
        .find(|path| path.exists());
    let Some(probe) = probe else {
        skip!(
            "the probe binary is not where it was expected: {}",
            target_dir.display()
        );
    };

    let dump = Command::new(&disassembler)
        .args(["-d", "--demangle", "--no-show-raw-insn"])
        .arg(&probe)
        .output()
        .expect("the disassembler must be spawnable once it has been found");
    if !dump.status.success() {
        skip!(
            "`{}` could not disassemble the probe:\n{}",
            disassembler.display(),
            String::from_utf8_lossy(&dump.stderr)
        );
    }

    let functions = disassemble(&String::from_utf8_lossy(&dump.stdout));
    if functions.is_empty() {
        skip!(
            "`{}` produced no recognisable functions",
            disassembler.display()
        );
    }

    let mut compared = 0_usize;
    for operation in OPERATIONS {
        let bare_name = format!("doublets_fusion_bare_{operation}");
        let composed_name = format!("doublets_fusion_composed_{operation}");

        let (Some(bare), Some(composed)) =
            (functions.get(&bare_name), functions.get(&composed_name))
        else {
            if functions.contains_key(&bare_name) || functions.contains_key(&composed_name) {
                // Identical code folding merged the two bodies into a single symbol,
                // which is the strongest possible evidence that the stack fused.
                compared += 1;
                continue;
            }
            skip!("neither `{bare_name}` nor `{composed_name}` is in the disassembly");
        };

        let bare = normalize(bare, &bare_name);
        let composed = normalize(composed, &composed_name);
        assert_eq!(
            bare, composed,
            "the composed `{operation}` did not fuse into the bare one; \
             a decorator layer survived as a real instruction"
        );
        compared += 1;
    }

    assert_eq!(
        compared,
        OPERATIONS.len(),
        "every probed operation must be accounted for"
    );
}

/// Returns the first disassembler that answers `--version`.
fn find_disassembler() -> Option<PathBuf> {
    ["llvm-objdump", "objdump"].into_iter().find_map(|name| {
        Command::new(name)
            .arg("--version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|_| PathBuf::from(name))
    })
}

/// Splits `objdump -d` output into `symbol name -> instruction lines`.
///
/// Both GNU `objdump` and `llvm-objdump` introduce a function with a line of the form
/// `<address> <<name>>:`; macOS prefixes exported symbols with an underscore.
fn disassemble(dump: &str) -> HashMap<String, Vec<String>> {
    let mut functions = HashMap::new();
    let mut current: Option<(String, Vec<String>)> = None;

    for line in dump.lines() {
        if let Some(name) = function_header(line) {
            if let Some((name, body)) = current.take() {
                functions.insert(name, body);
            }
            current = Some((name, Vec::new()));
        } else if let Some((_, body)) = current.as_mut() {
            match instruction(line) {
                Some(instruction) => body.push(instruction),
                None if line.trim().is_empty() => {
                    let (name, body) = current.take().expect("just matched as `Some`");
                    functions.insert(name, body);
                }
                None => {}
            }
        }
    }
    if let Some((name, body)) = current {
        functions.insert(name, body);
    }
    functions
}

/// Recognises `0000000000001234 <symbol>:` and returns `symbol`.
fn function_header(line: &str) -> Option<String> {
    let (address, rest) = line.split_once(' ')?;
    if address.is_empty() || !address.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let name = rest.trim().strip_prefix('<')?.strip_suffix(">:")?;
    Some(name.strip_prefix('_').unwrap_or(name).to_owned())
}

/// Recognises `  1234:\tmov %rsp,%rbp` and returns the instruction text.
fn instruction(line: &str) -> Option<String> {
    let (address, rest) = line.trim_start().split_once(':')?;
    if address.is_empty() || !address.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let rest = rest.trim();
    (!rest.is_empty()).then(|| rest.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Erases everything that legitimately differs between two copies of the same code: the
/// absolute addresses, the assembler comments they appear in, the trailing alignment
/// padding, and the function's own name in its relative jump targets.
fn normalize(body: &[String], name: &str) -> Vec<String> {
    let mut normalized: Vec<_> = body
        .iter()
        .map(|instruction| {
            let instruction = instruction.split('#').next().unwrap_or_default().trim();
            mask_addresses(instruction).replace(name, "SELF")
        })
        .filter(|instruction| !instruction.is_empty())
        .collect();

    while normalized.last().is_some_and(|last| is_padding(last)) {
        normalized.pop();
    }
    normalized
}

/// Replaces every hexadecimal run of four digits or more — `0x40c35`, `18e9b` — with a
/// placeholder. Short literals such as `$0x30` are real operands and are kept.
fn mask_addresses(text: &str) -> String {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let chars: Vec<char> = text.chars().collect();
    let mut masked = String::with_capacity(text.len());
    let mut at = 0;

    while at < chars.len() {
        let run_start = if chars[at] == '0' && chars.get(at + 1) == Some(&'x') {
            at + 2
        } else if chars[at].is_ascii_hexdigit() && (at == 0 || !is_word(chars[at - 1])) {
            at
        } else {
            masked.push(chars[at]);
            at += 1;
            continue;
        };

        let mut run_end = run_start;
        while chars.get(run_end).is_some_and(char::is_ascii_hexdigit) {
            run_end += 1;
        }
        if run_end - run_start >= 4 && !chars.get(run_end).copied().is_some_and(is_word) {
            masked.push_str("ADDR");
            at = run_end;
        } else {
            masked.push(chars[at]);
            at += 1;
        }
    }
    masked
}

/// `int3` / `nop` filler the assembler emits to align the next function.
fn is_padding(instruction: &str) -> bool {
    let mnemonic = instruction.split_whitespace().next().unwrap_or_default();
    mnemonic.starts_with("nop") || mnemonic == "int3" || mnemonic == "ud2"
}
