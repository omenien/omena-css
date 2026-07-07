//! The epoch ledger (rfcs#111 §4.1, §8.2): one monotone clock plus a
//! high-water mark per semantic input kind. Jobs carry a footprint-restricted
//! stamp; validity compares ONLY the marks inside the footprint, so an input
//! mutation that a derivation never reads cannot stale its in-flight job
//! (the review BLOCKER: an unfootprinted global clock would discard more
//! in-flight work than the per-subsystem counters it replaces).

/// The semantic-input taxonomy. Every state mutation the scheduler cares
/// about advances the ledger under one or more of these kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TideInputKindV0 {
    /// didOpen / didChange document text.
    DocumentText = 0,
    /// Corpus membership: background-scan admits AND document-event foreign
    /// admits (the second producer the frontier must see — rfcs#111 §4.2).
    DocumentSet = 1,
    /// Lockfile content identity. A fingerprinted source node: watched-file
    /// events over lockfiles advance this mark, so the SIF job's execution-
    /// time read is covered by an input instead of being out-of-band.
    LockfileFingerprint = 2,
    PackageManifest = 3,
    ResolutionSettings = 4,
    /// Diagnostics severity / deep-analysis settings. Wiring this kind is
    /// the structural fix for the config-staleness bug (rfcs#111 §2).
    DiagnosticSettings = 5,
    WorkspaceFolders = 6,
}

pub const TIDE_INPUT_KIND_COUNT: usize = 7;

// The footprint bitset is a u8: adding a ninth input kind must widen it,
// not silently shift out of range.
const _: () = assert!(TIDE_INPUT_KIND_COUNT <= 8);

impl TideInputKindV0 {
    pub const ALL: [TideInputKindV0; TIDE_INPUT_KIND_COUNT] = [
        TideInputKindV0::DocumentText,
        TideInputKindV0::DocumentSet,
        TideInputKindV0::LockfileFingerprint,
        TideInputKindV0::PackageManifest,
        TideInputKindV0::ResolutionSettings,
        TideInputKindV0::DiagnosticSettings,
        TideInputKindV0::WorkspaceFolders,
    ];
}

/// A declared input set, constant per derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TideFootprintV0(u8);

impl TideFootprintV0 {
    pub const fn of(kinds: &[TideInputKindV0]) -> Self {
        let mut bits = 0u8;
        let mut index = 0;
        while index < kinds.len() {
            bits |= 1 << (kinds[index] as u8);
            index += 1;
        }
        Self(bits)
    }

    pub const fn contains(&self, kind: TideInputKindV0) -> bool {
        self.0 & (1 << (kind as u8)) != 0
    }
}

/// The marks a job read at dispatch time, restricted (logically) to its
/// footprint. The full array is carried for simplicity; comparisons only
/// ever consult footprint members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TideFootprintStampV0 {
    pub footprint: TideFootprintV0,
    pub epoch: u64,
    marks: [u64; TIDE_INPUT_KIND_COUNT],
}

#[derive(Debug, Clone, Default)]
pub struct TideEpochLedgerV0 {
    epoch: u64,
    marks: [u64; TIDE_INPUT_KIND_COUNT],
}

impl TideEpochLedgerV0 {
    /// One state mutation: the epoch advances once and stamps the mark of
    /// every kind the mutation touched.
    pub fn advance(&mut self, kinds: &[TideInputKindV0]) -> u64 {
        self.epoch = self.epoch.saturating_add(1);
        for kind in kinds {
            self.marks[*kind as usize] = self.epoch;
        }
        self.epoch
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn mark(&self, kind: TideInputKindV0) -> u64 {
        self.marks[kind as usize]
    }

    pub fn stamp(&self, footprint: TideFootprintV0) -> TideFootprintStampV0 {
        TideFootprintStampV0 {
            footprint,
            epoch: self.epoch,
            marks: self.marks,
        }
    }

    /// Footprint-scoped validity: current iff no input kind INSIDE the
    /// footprint advanced past the stamp. Mutations of unrelated kinds are
    /// invisible here by construction (rfcs#111 §9.3).
    pub fn is_current(&self, stamp: &TideFootprintStampV0) -> bool {
        TideInputKindV0::ALL.iter().all(|kind| {
            !stamp.footprint.contains(*kind)
                || self.marks[*kind as usize] == stamp.marks[*kind as usize]
        })
    }
}
