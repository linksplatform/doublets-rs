//! Durable recovery, including subprocesses exiting without running destructors.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    process::Command,
};

use doublets::{
    decorators::{DecoratorsExt, FileTransitionLog, JournalEntry, Transition, TransitionLog},
    mem::{FileMapped, Global},
    unit, Doublets, DoubletsExt, Link,
};
use tempfile::tempdir;

#[test]
fn file_log_round_trips_records_and_repairs_torn_tail() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    let entries = vec![
        JournalEntry::Begin {
            transaction_id: 1,
            snapshot: vec![],
        },
        JournalEntry::Transition(Transition::new(
            1,
            1,
            Link::nothing(),
            Link::new(1u64, 0, 0),
        )),
        JournalEntry::Transition(Transition::new(1, 2, Link::new(1, 0, 0), Link::point(1))),
        JournalEntry::Commit {
            transaction_id: 1,
            snapshot: vec![Link::point(1)],
        },
        JournalEntry::Begin {
            transaction_id: 2,
            snapshot: vec![Link::point(1)],
        },
        JournalEntry::Transition(Transition::new(2, 3, Link::point(1), Link::nothing())),
        JournalEntry::Rollback { transaction_id: 2 },
    ];
    let mut log = FileTransitionLog::open(&path).unwrap();
    for entry in &entries {
        log.append(entry).unwrap();
    }
    drop(log);
    OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"doublets-tx-v1|B|3|\xff")
        .unwrap();
    let mut log = FileTransitionLog::<u64>::open(&path).unwrap();
    assert_eq!(log.read_entries().unwrap(), entries);
    log.append(&JournalEntry::Begin {
        transaction_id: 3,
        snapshot: vec![],
    })
    .unwrap();
    drop(log);
    assert_eq!(
        FileTransitionLog::<u64>::open(&path)
            .unwrap()
            .read_entries()
            .unwrap()
            .len(),
        8
    );
}

#[test]
fn file_log_rejects_complete_corruption_and_address_narrowing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut log = FileTransitionLog::<u64>::open(&path).unwrap();
    log.append(&JournalEntry::Begin {
        transaction_id: 1,
        snapshot: vec![Link::point(u64::from(u32::MAX) + 1)],
    })
    .unwrap();
    drop(log);
    let error = FileTransitionLog::<u32>::open(&path).unwrap_err();
    assert!(error.to_string().contains("exceeds journal address type"));
    let mut bytes = fs::read(&path).unwrap();
    bytes[0] = b'X';
    fs::write(&path, &bytes).unwrap();
    assert!(FileTransitionLog::<u64>::open(&path)
        .unwrap_err()
        .to_string()
        .contains("checksum"));
    // Complete corruption is preserved for investigation.
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn an_entirely_torn_log_can_be_reopened_and_appended() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    fs::write(&path, b"incomplete first record").unwrap();
    let mut log = FileTransitionLog::<usize>::open(&path).unwrap();
    assert_eq!(
        log.read_entries().unwrap(),
        Vec::<JournalEntry<usize>>::new()
    );
    log.append(&JournalEntry::Begin {
        transaction_id: 1,
        snapshot: vec![],
    })
    .unwrap();
    drop(log);
    assert_eq!(
        FileTransitionLog::<usize>::open(&path)
            .unwrap()
            .read_entries()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn recovery_undoes_mutation_even_without_a_transition_callback() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let a = raw.create_point().unwrap();
    let b = raw.create_point().unwrap();
    let mut log = FileTransitionLog::open(&path).unwrap();
    log.append(&JournalEntry::Begin {
        transaction_id: 1,
        snapshot: raw.iter().collect(),
    })
    .unwrap();
    // Model the gap between mutation and callback: no Transition record exists.
    raw.update(a, b, a).unwrap();
    raw.delete(b).unwrap();
    drop(log);
    let store = raw
        .with_transactions(FileTransitionLog::open(&path).unwrap())
        .unwrap();
    assert_eq!(store.get_link(a), Some(Link::point(a)));
    assert_eq!(store.get_link(b), Some(Link::point(b)));
    let (raw, log) = store.into_parts().unwrap();
    drop(log);
    let store = raw
        .with_transactions(FileTransitionLog::open(&path).unwrap())
        .unwrap();
    assert_eq!(store.count(), 2);
}

#[test]
fn committed_snapshots_restore_holes_swapped_pairs_and_missing_store_writes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    for _ in 0..5 {
        raw.create_point().unwrap();
    }
    raw.delete(2).unwrap();
    let mut store = raw
        .with_transactions(FileTransitionLog::open(&path).unwrap())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    tx.update(1, 0, 0).unwrap();
    tx.update(3, 1, 1).unwrap();
    tx.update(1, 3, 3).unwrap();
    tx.commit().unwrap();
    let expected = store.iter().collect::<Vec<_>>();
    let (mut raw, log) = store.into_parts().unwrap();
    drop(log);
    // Readable partial persistence can contain an intermediate state.
    raw.update(1, 0, 0).unwrap();
    raw.update(3, 3, 3).unwrap();
    raw.update(1, 1, 1).unwrap();
    let store = raw
        .with_transactions(FileTransitionLog::open(&path).unwrap())
        .unwrap();
    assert_eq!(store.iter().collect::<Vec<_>>(), expected);
    let (_, log) = store.into_parts().unwrap();
    drop(log);
    // Commit redo also works if none of the data-store writes persisted.
    let fresh = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let store = fresh
        .with_transactions(FileTransitionLog::open(&path).unwrap())
        .unwrap();
    assert_eq!(store.iter().collect::<Vec<_>>(), expected);
    assert!(!store.exist(2));
    assert_eq!(store.search(1, 1), Some(3));
    assert_eq!(store.search(3, 3), Some(1));
}

#[test]
fn recovery_rejects_invalid_marker_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut log = FileTransitionLog::open(&path).unwrap();
    log.append(&JournalEntry::Commit {
        transaction_id: 1,
        snapshot: vec![Link::point(1usize)],
    })
    .unwrap();
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    assert!(raw.with_transactions(log).is_err());
}

#[test]
fn process_exit_recovers_uncommitted_and_committed_file_mapped_writes() {
    for mode in ["pending", "committed"] {
        let dir = tempdir().unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_worker", "--nocapture"])
            .env("DOUBLETS_TX_CRASH_ROOT", dir.path())
            .env("DOUBLETS_TX_CRASH_MODE", mode)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(86),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let db_path = dir.path().join("store");
        if mode == "committed" {
            fs::remove_file(&db_path).unwrap();
        }
        let raw = unit::Store::<usize, _>::new(FileMapped::from_path(&db_path).unwrap()).unwrap();
        let store = raw
            .with_transactions(FileTransitionLog::open(dir.path().join("journal")).unwrap())
            .unwrap();
        if mode == "pending" {
            assert_eq!(
                store.iter().collect::<Vec<_>>(),
                vec![Link::point(1), Link::point(2)]
            );
        } else {
            assert_eq!(
                store.iter().collect::<Vec<_>>(),
                vec![Link::point(1), Link::new(2, 1, 2), Link::point(3)]
            );
        }
    }
}

#[test]
fn crash_worker() {
    let Some(dir) = std::env::var_os("DOUBLETS_TX_CRASH_ROOT") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let raw =
        unit::Store::<usize, _>::new(FileMapped::from_path(dir.join("store")).unwrap()).unwrap();
    let mut store = raw
        .with_transactions(FileTransitionLog::open(dir.join("journal")).unwrap())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    tx.create_point().unwrap();
    tx.create_point().unwrap();
    tx.commit().unwrap();
    let mut tx = store.begin_transaction().unwrap();
    tx.update(2, 1, 2).unwrap();
    tx.create_point().unwrap();
    if std::env::var("DOUBLETS_TX_CRASH_MODE").unwrap() == "committed" {
        tx.commit().unwrap();
    }
    // Neither the transaction guard nor the mapped store gets its Drop here.
    std::process::exit(86);
}
