use omena_query::{
    OmenaParserStyleDialect, OmenaQueryParseTreeNodeV0, OmenaWorkspaceSnapshotIdV0,
    parse_style_document_typed_v0,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const ABSENT_CONTENT_DIGEST: &str = "absent";
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum WorkspaceEditSafetyClassV0 {
    FormattingOnly,
    Safe,
    Conservative,
    EvidenceRequired,
    PlanFirst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedContentDigestV0 {
    path: PathBuf,
    digest: String,
}

impl ExpectedContentDigestV0 {
    pub(crate) fn from_bytes(path: &Path, content: &[u8]) -> Self {
        Self {
            path: path.to_path_buf(),
            digest: content_digest(content),
        }
    }

    pub(crate) fn observe(path: &Path) -> Result<Self, WorkspaceEditTransactionErrorV0> {
        match fs::read(path) {
            Ok(content) => Ok(Self::from_bytes(path, content.as_slice())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                path: path.to_path_buf(),
                digest: ABSENT_CONTENT_DIGEST.to_string(),
            }),
            Err(error) => Err(io_error("read expected content", path, error)),
        }
    }
}

type PostconditionCheck = Box<dyn Fn(&Path, &[u8]) -> Result<(), String> + Send + Sync + 'static>;

pub(crate) struct WorkspaceEditPostconditionV0 {
    name: &'static str,
    check: PostconditionCheck,
}

impl WorkspaceEditPostconditionV0 {
    pub(crate) fn new(
        name: &'static str,
        check: impl Fn(&Path, &[u8]) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name,
            check: Box::new(check),
        }
    }

    pub(crate) fn style_reparse(dialect: OmenaParserStyleDialect) -> Self {
        Self::new("styleReparse", move |_path, content| {
            let source = std::str::from_utf8(content)
                .map_err(|error| format!("staged style output is not UTF-8: {error}"))?;
            let tree = parse_style_document_typed_v0(source, dialect);
            if parse_tree_has_error(&tree) {
                return Err("staged style output did not reparse cleanly".to_string());
            }
            Ok(())
        })
    }

    pub(crate) fn text_reparse_for_path(path: &Path) -> Self {
        match text_syntax_for_path(path) {
            WorkspaceEditTextSyntaxV0::Style(dialect) => Self::style_reparse(dialect),
            WorkspaceEditTextSyntaxV0::Utf8 => Self::utf8_text(),
        }
    }

    pub(crate) fn style_reparse_for_admitted_output(path: &Path, admitted: &[u8]) -> Self {
        match text_syntax_for_path(path) {
            WorkspaceEditTextSyntaxV0::Style(dialect)
                if style_bytes_reparse_cleanly(admitted, dialect) =>
            {
                Self::style_reparse(dialect)
            }
            WorkspaceEditTextSyntaxV0::Style(_) => {
                Self::new("partialStyleOutputUtf8", |_path, content| {
                    std::str::from_utf8(content).map(|_| ()).map_err(|error| {
                        format!("staged partial style output is not UTF-8: {error}")
                    })
                })
            }
            WorkspaceEditTextSyntaxV0::Utf8 => Self::utf8_text(),
        }
    }

    pub(crate) fn utf8_text() -> Self {
        Self::new("utf8Text", |_path, content| {
            std::str::from_utf8(content)
                .map(|_| ())
                .map_err(|error| format!("staged text output is not UTF-8: {error}"))
        })
    }

    pub(crate) fn json_reparse() -> Self {
        Self::new("jsonReparse", |_path, content| {
            serde_json::from_slice::<serde_json::Value>(content)
                .map(|_| ())
                .map_err(|error| format!("staged JSON output did not reparse cleanly: {error}"))
        })
    }

    pub(crate) fn byte_identity(expected: &[u8]) -> Self {
        let expected_digest = content_digest(expected);
        Self::new("byteIdentity", move |_path, content| {
            let actual_digest = content_digest(content);
            if actual_digest == expected_digest {
                Ok(())
            } else {
                Err(format!(
                    "staged output digest {actual_digest} did not match admitted output digest {expected_digest}"
                ))
            }
        })
    }
}

pub(crate) struct FileEditV0 {
    path: PathBuf,
    content: Vec<u8>,
    postconditions: Vec<WorkspaceEditPostconditionV0>,
}

impl FileEditV0 {
    pub(crate) fn new(path: &Path, content: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.to_path_buf(),
            content: content.into(),
            postconditions: Vec::new(),
        }
    }

    pub(crate) fn with_postcondition(
        mut self,
        postcondition: WorkspaceEditPostconditionV0,
    ) -> Self {
        self.postconditions.push(postcondition);
        self
    }
}

pub(crate) struct WorkspaceEditTransaction {
    pub(crate) revision: Option<OmenaWorkspaceSnapshotIdV0>,
    pub(crate) expected_digests: Vec<ExpectedContentDigestV0>,
    pub(crate) edits: Vec<FileEditV0>,
    pub(crate) safety_class: WorkspaceEditSafetyClassV0,
    #[cfg(test)]
    failpoint: Option<WorkspaceEditFailpointV0>,
    #[cfg(test)]
    staging_directory: Option<PathBuf>,
}

impl WorkspaceEditTransaction {
    pub(crate) fn new(
        revision: Option<OmenaWorkspaceSnapshotIdV0>,
        safety_class: WorkspaceEditSafetyClassV0,
    ) -> Self {
        Self {
            revision,
            expected_digests: Vec::new(),
            edits: Vec::new(),
            safety_class,
            #[cfg(test)]
            failpoint: None,
            #[cfg(test)]
            staging_directory: None,
        }
    }

    pub(crate) fn expect(mut self, expected: ExpectedContentDigestV0) -> Self {
        self.expected_digests.push(expected);
        self
    }

    pub(crate) fn edit(mut self, edit: FileEditV0) -> Self {
        self.edits.push(edit);
        self
    }

    pub(crate) fn commit(
        self,
    ) -> Result<WorkspaceEditTransactionReportV0, WorkspaceEditTransactionErrorV0> {
        self.validate_shape()?;
        let transaction_id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
        let _transaction_locks = TransactionLockGuard::acquire(&self.edits, transaction_id)?;
        self.verify_expected_content()?;
        let mut staged = self.stage_edits(transaction_id)?;
        if let Err(error) = self.verify_staged_edits(staged.as_slice()) {
            cleanup_staged(staged.as_slice());
            return Err(error);
        }
        if let Err(error) = self.verify_expected_content() {
            cleanup_staged(staged.as_slice());
            return Err(error);
        }
        #[cfg(test)]
        if self.failpoint == Some(WorkspaceEditFailpointV0::BeforeRename) {
            cleanup_staged(staged.as_slice());
            return Err(WorkspaceEditTransactionErrorV0::InjectedFailure {
                point: "beforeRename",
            });
        }

        let journal_path = journal_path(&self.edits[0].path, transaction_id);
        if let Err(error) = self.write_journal(journal_path.as_path(), staged.as_slice()) {
            cleanup_staged(staged.as_slice());
            return Err(error);
        }
        let commit_result = self.rename_all(staged.as_mut_slice());
        match commit_result {
            Ok(()) => {
                cleanup_backups(staged.as_slice(), journal_path.as_path())?;
                Ok(WorkspaceEditTransactionReportV0 {
                    revision: self.revision,
                    safety_class: self.safety_class,
                    edited_file_count: self.edits.len(),
                    postcondition_count: self
                        .edits
                        .iter()
                        .map(|edit| edit.postconditions.len())
                        .sum(),
                })
            }
            Err(cause) => {
                let rollback_failures = rollback(staged.as_mut_slice());
                cleanup_staged(staged.as_slice());
                if rollback_failures.is_empty() {
                    remove_if_exists(journal_path.as_path())?;
                    Err(cause)
                } else {
                    Err(WorkspaceEditTransactionErrorV0::RollbackFailed {
                        cause: cause.to_string(),
                        failures: rollback_failures,
                        journal_path: journal_path.to_string_lossy().into_owned(),
                    })
                }
            }
        }
    }

    fn validate_shape(&self) -> Result<(), WorkspaceEditTransactionErrorV0> {
        if self.edits.is_empty() {
            return Err(WorkspaceEditTransactionErrorV0::EmptyTransaction);
        }
        for (index, edit) in self.edits.iter().enumerate() {
            if self.edits[..index]
                .iter()
                .any(|other| other.path == edit.path)
            {
                return Err(WorkspaceEditTransactionErrorV0::DuplicateEditPath {
                    path: edit.path.to_string_lossy().into_owned(),
                });
            }
            if !self
                .expected_digests
                .iter()
                .any(|expected| expected.path == edit.path)
            {
                return Err(WorkspaceEditTransactionErrorV0::MissingExpectedDigest {
                    path: edit.path.to_string_lossy().into_owned(),
                });
            }
        }
        Ok(())
    }

    fn verify_expected_content(&self) -> Result<(), WorkspaceEditTransactionErrorV0> {
        for expected in &self.expected_digests {
            let actual = ExpectedContentDigestV0::observe(expected.path.as_path())?;
            if actual.digest != expected.digest {
                return Err(WorkspaceEditTransactionErrorV0::StaleInput {
                    path: expected.path.to_string_lossy().into_owned(),
                    expected_digest: expected.digest.clone(),
                    actual_digest: actual.digest,
                });
            }
        }
        Ok(())
    }

    fn stage_edits(
        &self,
        transaction_id: u64,
    ) -> Result<Vec<StagedEditV0>, WorkspaceEditTransactionErrorV0> {
        let mut staged = Vec::with_capacity(self.edits.len());
        for (index, edit) in self.edits.iter().enumerate() {
            let target_parent = path_parent(edit.path.as_path());
            #[cfg(test)]
            let staging_parent = self
                .staging_directory
                .as_deref()
                .unwrap_or(target_parent.as_path());
            #[cfg(not(test))]
            let staging_parent = target_parent.as_path();
            if staging_parent != target_parent {
                cleanup_staged(staged.as_slice());
                return Err(
                    WorkspaceEditTransactionErrorV0::CrossFilesystemStagingDirectory {
                        target: edit.path.to_string_lossy().into_owned(),
                        staging_directory: staging_parent.to_string_lossy().into_owned(),
                    },
                );
            }
            let stage_path = sidecar_path(edit.path.as_path(), "stage", transaction_id, index);
            let backup_path = sidecar_path(edit.path.as_path(), "backup", transaction_id, index);
            let stage_result =
                write_staged_product_bytes(stage_path.as_path(), edit.content.as_slice());
            if let Err(error) = stage_result {
                cleanup_staged(staged.as_slice());
                return Err(error);
            }
            if let Ok(metadata) = fs::metadata(edit.path.as_path())
                && let Err(error) =
                    fs::set_permissions(stage_path.as_path(), metadata.permissions())
            {
                cleanup_staged(staged.as_slice());
                remove_if_exists(stage_path.as_path())?;
                return Err(io_error(
                    "preserve destination permissions",
                    &stage_path,
                    error,
                ));
            }
            staged.push(StagedEditV0 {
                destination: edit.path.clone(),
                stage: stage_path,
                backup: backup_path,
                destination_existed: edit.path.exists(),
                backup_ready: false,
                replacement_moved: false,
            });
        }
        Ok(staged)
    }

    fn verify_staged_edits(
        &self,
        staged: &[StagedEditV0],
    ) -> Result<(), WorkspaceEditTransactionErrorV0> {
        for (edit, staged_edit) in self.edits.iter().zip(staged) {
            let staged_content = fs::read(staged_edit.stage.as_path())
                .map_err(|error| io_error("read staged content", &staged_edit.stage, error))?;
            for postcondition in &edit.postconditions {
                (postcondition.check)(staged_edit.stage.as_path(), staged_content.as_slice())
                    .map_err(
                        |message| WorkspaceEditTransactionErrorV0::PostconditionFailed {
                            destination: edit.path.to_string_lossy().into_owned(),
                            staged_path: staged_edit.stage.to_string_lossy().into_owned(),
                            postcondition: postcondition.name,
                            message,
                        },
                    )?;
            }
        }
        Ok(())
    }

    fn write_journal(
        &self,
        path: &Path,
        staged: &[StagedEditV0],
    ) -> Result<(), WorkspaceEditTransactionErrorV0> {
        let journal = WorkspaceEditJournalV0 {
            schema_version: "0",
            product: "omena-cli.workspace-edit-journal",
            revision: self.revision,
            safety_class: self.safety_class,
            entries: staged
                .iter()
                .map(|edit| WorkspaceEditJournalEntryV0 {
                    destination: edit.destination.to_string_lossy().into_owned(),
                    stage: edit.stage.to_string_lossy().into_owned(),
                    backup: edit.backup.to_string_lossy().into_owned(),
                    destination_existed: edit.destination_existed,
                })
                .collect(),
        };
        let mut encoded = serde_json::to_vec_pretty(&journal).map_err(|error| {
            WorkspaceEditTransactionErrorV0::JournalSerialization {
                message: error.to_string(),
            }
        })?;
        encoded.push(b'\n');
        write_transaction_journal_file(path, encoded.as_slice())
    }

    fn rename_all(
        &self,
        staged: &mut [StagedEditV0],
    ) -> Result<(), WorkspaceEditTransactionErrorV0> {
        for (index, edit) in staged.iter_mut().enumerate() {
            #[cfg(not(test))]
            let _ = index;
            if edit.destination_existed {
                prepare_rollback_backup(edit)?;
                edit.backup_ready = true;
            }
            fs::rename(edit.stage.as_path(), edit.destination.as_path()).map_err(|error| {
                io_error(
                    "atomically publish staged content",
                    &edit.destination,
                    error,
                )
            })?;
            edit.replacement_moved = true;
            #[cfg(test)]
            if self.failpoint == Some(WorkspaceEditFailpointV0::AfterRenames(index + 1)) {
                return Err(WorkspaceEditTransactionErrorV0::InjectedFailure {
                    point: "afterRename",
                });
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn with_failpoint(mut self, failpoint: WorkspaceEditFailpointV0) -> Self {
        self.failpoint = Some(failpoint);
        self
    }

    #[cfg(test)]
    fn with_staging_directory(mut self, path: &Path) -> Self {
        self.staging_directory = Some(path.to_path_buf());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceEditTransactionReportV0 {
    pub(crate) revision: Option<OmenaWorkspaceSnapshotIdV0>,
    pub(crate) safety_class: WorkspaceEditSafetyClassV0,
    pub(crate) edited_file_count: usize,
    pub(crate) postcondition_count: usize,
}

#[derive(Debug)]
pub(crate) enum WorkspaceEditTransactionErrorV0 {
    EmptyTransaction,
    ConcurrentTransaction {
        destination: String,
        lock_path: String,
    },
    DuplicateEditPath {
        path: String,
    },
    MissingExpectedDigest {
        path: String,
    },
    StaleInput {
        path: String,
        expected_digest: String,
        actual_digest: String,
    },
    CrossFilesystemStagingDirectory {
        target: String,
        staging_directory: String,
    },
    PostconditionFailed {
        destination: String,
        staged_path: String,
        postcondition: &'static str,
        message: String,
    },
    JournalSerialization {
        message: String,
    },
    Io {
        operation: &'static str,
        path: String,
        message: String,
    },
    RollbackFailed {
        cause: String,
        failures: Vec<String>,
        journal_path: String,
    },
    #[cfg(test)]
    InjectedFailure {
        point: &'static str,
    },
}

impl fmt::Display for WorkspaceEditTransactionErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTransaction => write!(formatter, "workspace edit transaction has no edits"),
            Self::ConcurrentTransaction {
                destination,
                lock_path,
            } => write!(
                formatter,
                "workspace edit refused concurrent or unrecovered transaction for {destination}; lock is {lock_path}"
            ),
            Self::DuplicateEditPath { path } => {
                write!(
                    formatter,
                    "workspace edit transaction repeats destination {path}"
                )
            }
            Self::MissingExpectedDigest { path } => write!(
                formatter,
                "workspace edit transaction has no analysis-snapshot digest for {path}"
            ),
            Self::StaleInput {
                path,
                expected_digest,
                actual_digest,
            } => write!(
                formatter,
                "workspace edit refused stale input {path}: expected {expected_digest}, found {actual_digest}"
            ),
            Self::CrossFilesystemStagingDirectory {
                target,
                staging_directory,
            } => write!(
                formatter,
                "workspace edit staging directory {staging_directory} is not beside target {target}"
            ),
            Self::PostconditionFailed {
                destination,
                staged_path,
                postcondition,
                message,
            } => write!(
                formatter,
                "workspace edit postcondition {postcondition} failed for staged {staged_path} before publishing {destination}: {message}"
            ),
            Self::JournalSerialization { message } => {
                write!(
                    formatter,
                    "failed to serialize workspace edit journal: {message}"
                )
            }
            Self::Io {
                operation,
                path,
                message,
            } => write!(
                formatter,
                "workspace edit failed to {operation} {path}: {message}"
            ),
            Self::RollbackFailed {
                cause,
                failures,
                journal_path,
            } => write!(
                formatter,
                "workspace edit failed ({cause}) and rollback was incomplete ({}); journal retained at {journal_path}",
                failures.join("; ")
            ),
            #[cfg(test)]
            Self::InjectedFailure { point } => {
                write!(formatter, "injected workspace edit failure at {point}")
            }
        }
    }
}

#[derive(Debug)]
struct StagedEditV0 {
    destination: PathBuf,
    stage: PathBuf,
    backup: PathBuf,
    destination_existed: bool,
    backup_ready: bool,
    replacement_moved: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceEditJournalV0 {
    schema_version: &'static str,
    product: &'static str,
    revision: Option<OmenaWorkspaceSnapshotIdV0>,
    safety_class: WorkspaceEditSafetyClassV0,
    entries: Vec<WorkspaceEditJournalEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceEditJournalEntryV0 {
    destination: String,
    stage: String,
    backup: String,
    destination_existed: bool,
}

struct TransactionLockGuard {
    paths: Vec<PathBuf>,
}

impl TransactionLockGuard {
    fn acquire(
        edits: &[FileEditV0],
        transaction_id: u64,
    ) -> Result<Self, WorkspaceEditTransactionErrorV0> {
        let mut destinations = edits
            .iter()
            .map(|edit| edit.path.clone())
            .collect::<Vec<_>>();
        destinations.sort();
        destinations.dedup();
        let mut guard = Self { paths: Vec::new() };
        for destination in destinations {
            let lock_path = transaction_lock_path(destination.as_path());
            write_transaction_lock_file(
                lock_path.as_path(),
                destination.as_path(),
                transaction_id,
            )?;
            guard.paths.push(lock_path.clone());
        }
        Ok(guard)
    }
}

impl Drop for TransactionLockGuard {
    fn drop(&mut self) {
        for path in self.paths.iter().rev() {
            let _ = fs::remove_file(path);
        }
    }
}

fn write_transaction_lock_file(
    lock_path: &Path,
    destination: &Path,
    transaction_id: u64,
) -> Result<(), WorkspaceEditTransactionErrorV0> {
    let mut lock = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        Ok(lock) => lock,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(WorkspaceEditTransactionErrorV0::ConcurrentTransaction {
                destination: destination.to_string_lossy().into_owned(),
                lock_path: lock_path.to_string_lossy().into_owned(),
            });
        }
        Err(error) => {
            return Err(io_error("create transaction lock", lock_path, error));
        }
    };
    if let Err(error) =
        writeln!(lock, "{}:{transaction_id}", std::process::id()).and_then(|()| lock.sync_all())
    {
        let _ = fs::remove_file(lock_path);
        return Err(io_error("write transaction lock", lock_path, error));
    }
    Ok(())
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceEditFailpointV0 {
    BeforeRename,
    AfterRenames(usize),
}

fn write_staged_product_bytes(
    path: &Path,
    content: &[u8],
) -> Result<(), WorkspaceEditTransactionErrorV0> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("create staged file", path, error))?;
    file.write_all(content)
        .map_err(|error| io_error("write staged file", path, error))?;
    file.sync_all()
        .map_err(|error| io_error("sync staged file", path, error))
}

fn write_transaction_journal_file(
    path: &Path,
    content: &[u8],
) -> Result<(), WorkspaceEditTransactionErrorV0> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("create transaction journal", path, error))?;
    file.write_all(content)
        .map_err(|error| io_error("write transaction journal", path, error))?;
    file.sync_all()
        .map_err(|error| io_error("sync transaction journal", path, error))
}

fn cleanup_backups(
    staged: &[StagedEditV0],
    journal_path: &Path,
) -> Result<(), WorkspaceEditTransactionErrorV0> {
    for edit in staged {
        remove_if_exists(edit.backup.as_path())?;
        remove_if_exists(edit.stage.as_path())?;
    }
    remove_if_exists(journal_path)
}

fn rollback(staged: &mut [StagedEditV0]) -> Vec<String> {
    let mut failures = Vec::new();
    for edit in staged.iter_mut().rev() {
        if edit.replacement_moved
            && let Err(error) = fs::remove_file(edit.destination.as_path())
            && error.kind() != std::io::ErrorKind::NotFound
        {
            failures.push(format!(
                "failed to remove replacement {}: {error}",
                edit.destination.display()
            ));
            continue;
        }
        if edit.backup_ready
            && let Err(error) = fs::rename(edit.backup.as_path(), edit.destination.as_path())
        {
            failures.push(format!(
                "failed to restore original {}: {error}",
                edit.destination.display()
            ));
        }
    }
    failures
}

fn cleanup_staged(staged: &[StagedEditV0]) {
    for edit in staged {
        let _ = fs::remove_file(edit.stage.as_path());
        if !edit.backup_ready {
            let _ = fs::remove_file(edit.backup.as_path());
        }
    }
}

fn remove_if_exists(path: &Path) -> Result<(), WorkspaceEditTransactionErrorV0> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("remove transaction sidecar", path, error)),
    }
}

#[cfg(windows)]
fn prepare_rollback_backup(edit: &StagedEditV0) -> Result<(), WorkspaceEditTransactionErrorV0> {
    fs::rename(edit.destination.as_path(), edit.backup.as_path())
        .map_err(|error| io_error("move original to rollback backup", &edit.destination, error))
}

#[cfg(not(windows))]
fn prepare_rollback_backup(edit: &StagedEditV0) -> Result<(), WorkspaceEditTransactionErrorV0> {
    fs::copy(edit.destination.as_path(), edit.backup.as_path())
        .map_err(|error| io_error("copy original to rollback backup", &edit.destination, error))?;
    fs::File::open(edit.backup.as_path())
        .and_then(|file| file.sync_all())
        .map_err(|error| io_error("sync rollback backup", &edit.backup, error))
}

fn path_parent(path: &Path) -> PathBuf {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn sidecar_path(path: &Path, kind: &str, transaction_id: u64, index: usize) -> PathBuf {
    let parent = path_parent(path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    parent.join(format!(
        ".{file_name}.omena-{kind}-{}-{transaction_id}-{index}",
        std::process::id()
    ))
}

fn journal_path(path: &Path, transaction_id: u64) -> PathBuf {
    sidecar_path(path, "journal", transaction_id, 0)
}

fn transaction_lock_path(path: &Path) -> PathBuf {
    let parent = path_parent(path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    parent.join(format!(".{file_name}.omena-workspace-edit.lock"))
}

fn content_digest(content: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(content);
    let mut encoded = String::with_capacity("sha256:".len() + digest.len() * 2);
    encoded.push_str("sha256:");
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn parse_tree_has_error(node: &OmenaQueryParseTreeNodeV0) -> bool {
    node.error.is_some()
        || node.bogus == Some(true)
        || node.children.iter().any(parse_tree_has_error)
}

enum WorkspaceEditTextSyntaxV0 {
    Style(OmenaParserStyleDialect),
    Utf8,
}

fn text_syntax_for_path(path: &Path) -> WorkspaceEditTextSyntaxV0 {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => WorkspaceEditTextSyntaxV0::Style(OmenaParserStyleDialect::Css),
        Some("scss") => WorkspaceEditTextSyntaxV0::Style(OmenaParserStyleDialect::Scss),
        Some("sass") => WorkspaceEditTextSyntaxV0::Style(OmenaParserStyleDialect::Sass),
        Some("less") => WorkspaceEditTextSyntaxV0::Style(OmenaParserStyleDialect::Less),
        Some("js" | "jsx" | "ts" | "tsx") => WorkspaceEditTextSyntaxV0::Utf8,
        Some(_) | None => WorkspaceEditTextSyntaxV0::Utf8,
    }
}

fn style_bytes_reparse_cleanly(content: &[u8], dialect: OmenaParserStyleDialect) -> bool {
    std::str::from_utf8(content)
        .is_ok_and(|source| !parse_tree_has_error(&parse_style_document_typed_v0(source, dialect)))
}

fn io_error(
    operation: &'static str,
    path: &Path,
    error: std::io::Error,
) -> WorkspaceEditTransactionErrorV0 {
    WorkspaceEditTransactionErrorV0::Io {
        operation,
        path: path.to_string_lossy().into_owned(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, mpsc};
    use std::thread;
    use std::time::Duration;

    fn fixture_root(label: &str) -> PathBuf {
        let id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-workspace-edit-{label}-{}-{id}",
            std::process::id()
        ))
    }

    fn transaction_for_existing(
        path: &Path,
        original: &[u8],
        replacement: &[u8],
    ) -> WorkspaceEditTransaction {
        WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::Safe)
            .expect(ExpectedContentDigestV0::from_bytes(path, original))
            .edit(
                FileEditV0::new(path, replacement)
                    .with_postcondition(WorkspaceEditPostconditionV0::byte_identity(replacement)),
            )
    }

    #[test]
    fn concurrent_transaction_is_refused() -> Result<(), String> {
        let root = fixture_root("concurrent");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        fs::write(&path, b".original {}\n").map_err(|error| error.to_string())?;
        let (staged_tx, staged_rx) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let first_release = Arc::clone(&release);
        let first = WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::Safe)
            .expect(ExpectedContentDigestV0::from_bytes(
                &path,
                b".original {}\n",
            ))
            .edit(
                FileEditV0::new(&path, b".first {}\n".to_vec()).with_postcondition(
                    WorkspaceEditPostconditionV0::new(
                        "concurrencyBarrier",
                        move |_path, _content| {
                            staged_tx.send(()).map_err(|error| {
                                format!("failed to signal staged write: {error}")
                            })?;
                            first_release.wait();
                            Ok(())
                        },
                    ),
                ),
            );
        let second = transaction_for_existing(&path, b".original {}\n", b".second {}\n");
        let first_handle = thread::spawn(move || first.commit());
        staged_rx
            .recv()
            .map_err(|error| format!("first transaction did not reach staging: {error}"))?;
        let (second_tx, second_rx) = mpsc::channel();
        let second_handle = thread::spawn(move || {
            let _ = second_tx.send(second.commit());
        });
        let second_result = second_rx.recv_timeout(Duration::from_secs(2));
        release.wait();
        first_handle
            .join()
            .map_err(|_| "first transaction panicked".to_string())?
            .map_err(|error| error.to_string())?;
        second_handle
            .join()
            .map_err(|_| "second transaction panicked".to_string())?;
        let second_result = second_result
            .map_err(|error| format!("concurrent transaction did not refuse promptly: {error}"))?;
        let Err(error) = second_result else {
            return Err("second transaction unexpectedly overwrote the first".to_string());
        };
        assert!(matches!(
            error,
            WorkspaceEditTransactionErrorV0::ConcurrentTransaction { .. }
        ));
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            b".first {}\n"
        );
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn external_edit_after_analysis_is_stale() -> Result<(), String> {
        let root = fixture_root("stale");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        fs::write(&path, b".original {}\n").map_err(|error| error.to_string())?;
        let transaction = transaction_for_existing(&path, b".original {}\n", b".replacement {}\n");
        fs::write(&path, b".external-edit {}\n").map_err(|error| error.to_string())?;
        let result = transaction.commit();
        let Err(error) = result else {
            return Err("stale transaction unexpectedly overwrote an external edit".to_string());
        };
        assert!(matches!(
            error,
            WorkspaceEditTransactionErrorV0::StaleInput { .. }
        ));
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            b".external-edit {}\n"
        );
        assert_no_transaction_sidecars(&root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn crash_before_rename_leaves_original_intact() -> Result<(), String> {
        let root = fixture_root("crash");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        let original = b".original { color: green; }\n";
        let replacement = b".replacement { color: blue; }\n";
        fs::write(&path, original).map_err(|error| error.to_string())?;

        let mut direct = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|error| error.to_string())?;
        direct
            .write_all(&replacement[..5])
            .map_err(|error| error.to_string())?;
        direct.sync_all().map_err(|error| error.to_string())?;
        drop(direct);
        assert_ne!(
            fs::read(&path).map_err(|error| error.to_string())?,
            original,
            "the bare-write control arm must expose truncation"
        );

        fs::write(&path, original).map_err(|error| error.to_string())?;
        let crash_result = transaction_for_existing(&path, original, replacement)
            .with_failpoint(WorkspaceEditFailpointV0::BeforeRename)
            .commit();
        let Err(error) = crash_result else {
            return Err("before-rename failpoint did not abort the transaction".to_string());
        };
        assert!(matches!(
            error,
            WorkspaceEditTransactionErrorV0::InjectedFailure {
                point: "beforeRename"
            }
        ));
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            original
        );
        assert_no_transaction_sidecars(&root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn staged_postcondition_fires_before_rename() -> Result<(), String> {
        let root = fixture_root("postcondition");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        let original = b".original {}\n";
        fs::write(&path, original).map_err(|error| error.to_string())?;
        let transaction =
            WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::FormattingOnly)
                .expect(ExpectedContentDigestV0::from_bytes(&path, original))
                .edit(
                    FileEditV0::new(&path, b".corrupt {".to_vec()).with_postcondition(
                        WorkspaceEditPostconditionV0::style_reparse(OmenaParserStyleDialect::Css),
                    ),
                );
        let postcondition_result = transaction.commit();
        let Err(error) = postcondition_result else {
            return Err("corrupt staged bytes unexpectedly passed reparsing".to_string());
        };
        assert!(matches!(
            error,
            WorkspaceEditTransactionErrorV0::PostconditionFailed { .. }
        ));
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            original
        );
        assert_no_transaction_sidecars(&root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn multi_file_failure_restores_originals_and_removes_new_files() -> Result<(), String> {
        let root = fixture_root("rollback");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let first = root.join("first.css");
        let second = root.join("second.css");
        let third = root.join("new.css");
        fs::write(&first, b"first-original").map_err(|error| error.to_string())?;
        fs::write(&second, b"second-original").map_err(|error| error.to_string())?;
        let transaction =
            WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::EvidenceRequired)
                .expect(ExpectedContentDigestV0::from_bytes(
                    &first,
                    b"first-original",
                ))
                .expect(ExpectedContentDigestV0::from_bytes(
                    &second,
                    b"second-original",
                ))
                .expect(
                    ExpectedContentDigestV0::observe(&third).map_err(|error| error.to_string())?,
                )
                .edit(FileEditV0::new(&first, b"first-replacement".to_vec()))
                .edit(FileEditV0::new(&second, b"second-replacement".to_vec()))
                .edit(FileEditV0::new(&third, b"third-new".to_vec()))
                .with_failpoint(WorkspaceEditFailpointV0::AfterRenames(3));
        if transaction.commit().is_ok() {
            return Err("k-of-n failpoint did not trigger rollback".to_string());
        }
        assert_eq!(
            fs::read(&first).map_err(|error| error.to_string())?,
            b"first-original"
        );
        assert_eq!(
            fs::read(&second).map_err(|error| error.to_string())?,
            b"second-original"
        );
        assert!(!third.exists());
        assert_no_transaction_sidecars(&root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn success_consumes_journal_and_preserves_permissions() -> Result<(), String> {
        let root = fixture_root("success");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        let original = b".original {}\n";
        fs::write(&path, original).map_err(|error| error.to_string())?;
        let permissions = fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .permissions();
        let report = transaction_for_existing(&path, original, b".replacement {}\n")
            .commit()
            .map_err(|error| error.to_string())?;
        assert_eq!(report.edited_file_count, 1);
        assert_eq!(
            fs::metadata(&path)
                .map_err(|error| error.to_string())?
                .permissions(),
            permissions
        );
        assert_no_transaction_sidecars(&root)?;
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn non_sibling_staging_directory_is_refused() -> Result<(), String> {
        let root = fixture_root("same-fs");
        let other = fixture_root("other-fs");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::create_dir_all(&other).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        fs::write(&path, b".original {}\n").map_err(|error| error.to_string())?;
        let staging_result = transaction_for_existing(&path, b".original {}\n", b".next {}\n")
            .with_staging_directory(&other)
            .commit();
        let Err(error) = staging_result else {
            return Err("non-sibling staging directory unexpectedly succeeded".to_string());
        };
        assert!(matches!(
            error,
            WorkspaceEditTransactionErrorV0::CrossFilesystemStagingDirectory { .. }
        ));
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            b".original {}\n"
        );
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        fs::remove_dir_all(other).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn product_write_census_is_zero() -> Result<(), String> {
        let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let product_surfaces = [
            "write_safety.rs",
            "format.rs",
            "lint/fixes.rs",
            "migrate/mod.rs",
            "migrate/apply.rs",
            "minify.rs",
            "build.rs",
            "sif.rs",
        ];
        let mut bypassers = Vec::new();
        for relative_path in product_surfaces {
            let source = fs::read_to_string(source_root.join(relative_path))
                .map_err(|error| format!("failed to read {relative_path}: {error}"))?;
            let production_source = source
                .split("#[cfg(test)]")
                .next()
                .unwrap_or(source.as_str());
            for (index, line) in production_source.lines().enumerate() {
                if line.contains("fs::write(") {
                    bypassers.push(format!("{relative_path}:{}", index + 1));
                }
            }
        }
        assert!(
            bypassers.is_empty(),
            "product/source writes bypass WorkspaceEditTransaction: {}",
            bypassers.join(", ")
        );
        Ok(())
    }

    fn assert_no_transaction_sidecars(root: &Path) -> Result<(), String> {
        let sidecars = fs::read_dir(root)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".omena-"))
            .collect::<Vec<_>>();
        assert!(
            sidecars.is_empty(),
            "orphan transaction sidecars: {sidecars:?}"
        );
        Ok(())
    }
}
