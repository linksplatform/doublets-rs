//! Memory and durable file journals.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use data::LinkReference;

use super::{JournalEntry, Transition, TransitionKind};
use crate::Link;

/// Storage for ordered transaction records.
///
/// A durable implementation must persist a complete record before `append`
/// succeeds, preserve append order, and reject corruption on `read_entries`.
/// An incomplete final append may be discarded. After an append error the
/// decorator blocks further writes until reopened, since durability is uncertain.
/// Use one exclusively owned journal for one exclusively owned store.
pub trait TransitionLog<T: LinkReference>: Send + Sync {
    /// Reads all complete records in append order.
    fn read_entries(&mut self) -> io::Result<Vec<JournalEntry<T>>>;
    /// Appends and, for durable implementations, synchronizes one record.
    fn append(&mut self, entry: &JournalEntry<T>) -> io::Result<()>;
}

/// An in-memory journal for transactions that do not require crash recovery.
#[derive(Debug, Clone, Default)]
pub struct MemoryTransitionLog<T: LinkReference> {
    entries: Vec<JournalEntry<T>>,
}

impl<T: LinkReference> MemoryTransitionLog<T> {
    /// Returns the complete journal, including rolled-back writes.
    #[must_use]
    pub fn entries(&self) -> &[JournalEntry<T>] {
        &self.entries
    }
}

impl<T: LinkReference> TransitionLog<T> for MemoryTransitionLog<T> {
    fn read_entries(&mut self) -> io::Result<Vec<JournalEntry<T>>> {
        Ok(self.entries.clone())
    }
    fn append(&mut self, entry: &JournalEntry<T>) -> io::Result<()> {
        self.entries.push(entry.clone());
        Ok(())
    }
}

/// A versioned, checksummed append-only file journal.
///
/// Every append uses `sync_all`. Opening validates every complete record and
/// removes a torn final line before further appends. Corrupt complete records
/// and addresses too wide for the requested `T` are errors, never skipped.
/// Creating a new file requires its parent directory to exist; synchronize that
/// directory before relying on a newly created path surviving a system crash.
/// File locking and retention are the caller's responsibility.
#[derive(Debug)]
pub struct FileTransitionLog<T: LinkReference> {
    file: File,
    marker: std::marker::PhantomData<T>,
}

impl<T: LinkReference> FileTransitionLog<T> {
    /// Opens or creates a journal and validates/repairs its final append.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let mut log = Self {
            file,
            marker: std::marker::PhantomData,
        };
        log.read_entries()?;
        Ok(log)
    }
}

impl<T: LinkReference> TransitionLog<T> for FileTransitionLog<T> {
    fn read_entries(&mut self) -> io::Result<Vec<JournalEntry<T>>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        self.file.read_to_end(&mut bytes)?;
        let complete = bytes
            .iter()
            .rposition(|&byte| byte == b'\n')
            .map_or(0, |i| i + 1);
        let contents = std::str::from_utf8(&bytes[..complete]).map_err(invalid)?;
        let entries = contents
            .lines()
            .map(decode)
            .collect::<io::Result<Vec<_>>>()?;
        if complete != bytes.len() {
            self.file
                .set_len(u64::try_from(complete).map_err(invalid)?)?;
            self.file.sync_all()?;
        }
        self.file.seek(SeekFrom::End(0))?;
        Ok(entries)
    }

    fn append(&mut self, entry: &JournalEntry<T>) -> io::Result<()> {
        let payload = encode(entry);
        writeln!(self.file, "{payload}#{:016x}", checksum(payload.as_bytes()))?;
        self.file.sync_all()
    }
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn link_text<T: LinkReference>(link: &Link<T>) -> String {
    format!("{},{},{}", link.index, link.source, link.target)
}

fn encode<T: LinkReference>(entry: &JournalEntry<T>) -> String {
    let body = match entry {
        JournalEntry::Begin {
            transaction_id,
            snapshot,
        }
        | JournalEntry::Commit {
            transaction_id,
            snapshot,
        } => {
            let kind = if matches!(entry, JournalEntry::Begin { .. }) {
                "B"
            } else {
                "C"
            };
            let snapshot = snapshot.iter().map(link_text).collect::<Vec<_>>().join(";");
            format!("{kind}|{transaction_id}|{snapshot}")
        }
        JournalEntry::Rollback { transaction_id } => format!("R|{transaction_id}"),
        JournalEntry::Transition(t) => {
            let kind = match t.kind {
                TransitionKind::Create => "C",
                TransitionKind::Update => "U",
                TransitionKind::Delete => "D",
            };
            format!(
                "T|{}|{}|{kind}|{}|{}",
                t.transaction_id,
                t.sequence,
                link_text(&t.before),
                link_text(&t.after)
            )
        }
    };
    format!("doublets-tx-v1|{body}")
}

fn parse_link<T: LinkReference>(text: &str) -> io::Result<Link<T>> {
    let fields: Vec<_> = text.split(',').collect();
    let [index, source, target] = fields.as_slice() else {
        return Err(invalid("invalid link triple"));
    };
    let address = |text: &str| {
        let value = text.parse::<u128>().map_err(invalid)?;
        T::try_from(value)
            .map_err(|_| invalid(format!("address {value} exceeds journal address type")))
    };
    Ok(Link::new(
        address(index)?,
        address(source)?,
        address(target)?,
    ))
}

fn decode<T: LinkReference>(line: &str) -> io::Result<JournalEntry<T>> {
    let (payload, hash) = line
        .rsplit_once('#')
        .ok_or_else(|| invalid("missing journal checksum"))?;
    if checksum(payload.as_bytes()) != u64::from_str_radix(hash, 16).map_err(invalid)? {
        return Err(invalid("journal checksum mismatch"));
    }
    let fields: Vec<_> = payload.split('|').collect();
    if fields.first() != Some(&"doublets-tx-v1") {
        return Err(invalid("unsupported journal version"));
    }
    match &fields[1..] {
        [kind @ ("B" | "C"), id, snapshot] => {
            let transaction_id = id.parse().map_err(invalid)?;
            let snapshot = if snapshot.is_empty() {
                Vec::new()
            } else {
                snapshot
                    .split(';')
                    .map(parse_link)
                    .collect::<io::Result<Vec<_>>>()?
            };
            Ok(if *kind == "B" {
                JournalEntry::Begin {
                    transaction_id,
                    snapshot,
                }
            } else {
                JournalEntry::Commit {
                    transaction_id,
                    snapshot,
                }
            })
        }
        ["R", id] => Ok(JournalEntry::Rollback {
            transaction_id: id.parse().map_err(invalid)?,
        }),
        ["T", id, sequence, kind, before, after] => {
            let t = Transition::new(
                id.parse().map_err(invalid)?,
                sequence.parse().map_err(invalid)?,
                parse_link(before)?,
                parse_link(after)?,
            );
            let expected = match t.kind {
                TransitionKind::Create => "C",
                TransitionKind::Update => "U",
                TransitionKind::Delete => "D",
            };
            if *kind != expected {
                return Err(invalid("invalid transition kind"));
            }
            Ok(JournalEntry::Transition(t))
        }
        _ => Err(invalid("invalid journal record")),
    }
}
