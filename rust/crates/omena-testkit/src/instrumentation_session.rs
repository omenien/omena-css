use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResolverStyleIdentityIndexCountsV0 {
    pub build_count: usize,
    pub build_work_count: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SalsaQueryRunCountsV0 {
    pub digest: usize,
    pub dependency: usize,
    pub transitive_leaf: usize,
    pub transitive_a: usize,
    pub transitive_b: usize,
    pub transitive_c: usize,
    pub transitive_unrelated: usize,
}

#[derive(Debug, Clone, Default)]
pub struct InstrumentationSessionV0 {
    inner: Arc<InstrumentationSessionInnerV0>,
}

#[derive(Debug, Default)]
struct InstrumentationSessionInnerV0 {
    resolver_identity_index_build_count: AtomicUsize,
    resolver_identity_index_build_work_count: AtomicUsize,
    salsa_digest_query_runs: AtomicUsize,
    salsa_dependency_query_runs: AtomicUsize,
    salsa_transitive_leaf_query_runs: AtomicUsize,
    salsa_transitive_a_query_runs: AtomicUsize,
    salsa_transitive_b_query_runs: AtomicUsize,
    salsa_transitive_c_query_runs: AtomicUsize,
    salsa_transitive_unrelated_query_runs: AtomicUsize,
}

thread_local! {
    static CURRENT_INSTRUMENTATION_SESSION: RefCell<Option<InstrumentationSessionV0>> =
        const { RefCell::new(None) };
}

static DEFAULT_INSTRUMENTATION_SESSION: OnceLock<InstrumentationSessionV0> = OnceLock::new();

pub fn default_instrumentation_session_v0() -> InstrumentationSessionV0 {
    DEFAULT_INSTRUMENTATION_SESSION
        .get_or_init(InstrumentationSessionV0::default)
        .clone()
}

pub fn current_instrumentation_session_v0() -> InstrumentationSessionV0 {
    CURRENT_INSTRUMENTATION_SESSION
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(default_instrumentation_session_v0)
}

pub fn with_instrumentation_session<R>(
    session: InstrumentationSessionV0,
    body: impl FnOnce() -> R,
) -> R {
    let guard = InstalledInstrumentationSessionV0::new(session);
    let result = body();
    drop(guard);
    result
}

struct InstalledInstrumentationSessionV0 {
    previous: Option<InstrumentationSessionV0>,
}

impl InstalledInstrumentationSessionV0 {
    fn new(session: InstrumentationSessionV0) -> Self {
        let previous = CURRENT_INSTRUMENTATION_SESSION.with(|slot| slot.replace(Some(session)));
        Self { previous }
    }
}

impl Drop for InstalledInstrumentationSessionV0 {
    fn drop(&mut self) {
        let previous = self.previous.take();
        CURRENT_INSTRUMENTATION_SESSION.with(|slot| {
            let _ = slot.replace(previous);
        });
    }
}

impl InstrumentationSessionV0 {
    pub fn reset_resolver_style_identity_index_counts(&self) {
        self.inner
            .resolver_identity_index_build_count
            .store(0, Ordering::Release);
        self.inner
            .resolver_identity_index_build_work_count
            .store(0, Ordering::Release);
    }

    pub fn record_resolver_style_identity_index_build(&self, work_count: usize) {
        self.inner
            .resolver_identity_index_build_count
            .fetch_add(1, Ordering::AcqRel);
        self.inner
            .resolver_identity_index_build_work_count
            .fetch_add(work_count, Ordering::AcqRel);
    }

    pub fn resolver_style_identity_index_counts(&self) -> ResolverStyleIdentityIndexCountsV0 {
        ResolverStyleIdentityIndexCountsV0 {
            build_count: self
                .inner
                .resolver_identity_index_build_count
                .load(Ordering::Acquire),
            build_work_count: self
                .inner
                .resolver_identity_index_build_work_count
                .load(Ordering::Acquire),
        }
    }

    pub fn reset_salsa_query_run_counts(&self) {
        self.inner
            .salsa_digest_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_dependency_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_transitive_leaf_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_transitive_a_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_transitive_b_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_transitive_c_query_runs
            .store(0, Ordering::Release);
        self.inner
            .salsa_transitive_unrelated_query_runs
            .store(0, Ordering::Release);
    }

    pub fn record_salsa_digest_query_run(&self) {
        self.inner
            .salsa_digest_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_dependency_query_run(&self) {
        self.inner
            .salsa_dependency_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_transitive_leaf_query_run(&self) {
        self.inner
            .salsa_transitive_leaf_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_transitive_a_query_run(&self) {
        self.inner
            .salsa_transitive_a_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_transitive_b_query_run(&self) {
        self.inner
            .salsa_transitive_b_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_transitive_c_query_run(&self) {
        self.inner
            .salsa_transitive_c_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn record_salsa_transitive_unrelated_query_run(&self) {
        self.inner
            .salsa_transitive_unrelated_query_runs
            .fetch_add(1, Ordering::AcqRel);
    }

    pub fn salsa_query_run_counts(&self) -> SalsaQueryRunCountsV0 {
        SalsaQueryRunCountsV0 {
            digest: self.inner.salsa_digest_query_runs.load(Ordering::Acquire),
            dependency: self
                .inner
                .salsa_dependency_query_runs
                .load(Ordering::Acquire),
            transitive_leaf: self
                .inner
                .salsa_transitive_leaf_query_runs
                .load(Ordering::Acquire),
            transitive_a: self
                .inner
                .salsa_transitive_a_query_runs
                .load(Ordering::Acquire),
            transitive_b: self
                .inner
                .salsa_transitive_b_query_runs
                .load(Ordering::Acquire),
            transitive_c: self
                .inner
                .salsa_transitive_c_query_runs
                .load(Ordering::Acquire),
            transitive_unrelated: self
                .inner
                .salsa_transitive_unrelated_query_runs
                .load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn instrumentation_sessions_isolate_concurrent_counts() -> Result<(), Box<dyn std::error::Error>>
    {
        let first = InstrumentationSessionV0::default();
        let second = InstrumentationSessionV0::default();
        let barrier = Arc::new(Barrier::new(2));

        let first_worker = {
            let session = first.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                with_instrumentation_session(session, || {
                    barrier.wait();
                    for _ in 0..11 {
                        current_instrumentation_session_v0()
                            .record_resolver_style_identity_index_build(3);
                    }
                });
            })
        };
        let second_worker = {
            let session = second.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                with_instrumentation_session(session, || {
                    barrier.wait();
                    for _ in 0..7 {
                        current_instrumentation_session_v0()
                            .record_resolver_style_identity_index_build(5);
                    }
                });
            })
        };

        first_worker
            .join()
            .map_err(|_| std::io::Error::other("first instrumentation worker panicked"))?;
        second_worker
            .join()
            .map_err(|_| std::io::Error::other("second instrumentation worker panicked"))?;

        assert_eq!(
            first.resolver_style_identity_index_counts(),
            ResolverStyleIdentityIndexCountsV0 {
                build_count: 11,
                build_work_count: 33
            }
        );
        assert_eq!(
            second.resolver_style_identity_index_counts(),
            ResolverStyleIdentityIndexCountsV0 {
                build_count: 7,
                build_work_count: 35
            }
        );
        Ok(())
    }

    #[test]
    fn instrumentation_session_restores_previous_session() {
        let outer = InstrumentationSessionV0::default();
        let inner = InstrumentationSessionV0::default();

        with_instrumentation_session(outer.clone(), || {
            current_instrumentation_session_v0().record_salsa_digest_query_run();
            with_instrumentation_session(inner.clone(), || {
                current_instrumentation_session_v0().record_salsa_digest_query_run();
                current_instrumentation_session_v0().record_salsa_dependency_query_run();
            });
            current_instrumentation_session_v0().record_salsa_dependency_query_run();
        });

        assert_eq!(
            outer.salsa_query_run_counts(),
            SalsaQueryRunCountsV0 {
                digest: 1,
                dependency: 1,
                ..SalsaQueryRunCountsV0::default()
            }
        );
        assert_eq!(
            inner.salsa_query_run_counts(),
            SalsaQueryRunCountsV0 {
                digest: 1,
                dependency: 1,
                ..SalsaQueryRunCountsV0::default()
            }
        );
    }
}
