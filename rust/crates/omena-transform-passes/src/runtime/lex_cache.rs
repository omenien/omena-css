use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use omena_parser::{LexResult, LexedToken, StyleDialect, lex};

#[derive(Debug, Clone)]
pub(crate) struct CachedLexResultV0 {
    inner: Rc<LexResult>,
}

impl CachedLexResultV0 {
    pub(crate) fn tokens(&self) -> &[LexedToken] {
        self.inner.tokens()
    }
}

#[derive(Default)]
struct TransformLexCacheV0 {
    entries: BTreeMap<(StyleDialect, String), Rc<LexResult>>,
}

thread_local! {
    static ACTIVE_TRANSFORM_LEX_CACHES: RefCell<Vec<TransformLexCacheV0>> =
        const { RefCell::new(Vec::new()) };
}

struct TransformLexCacheScopeGuard;

impl Drop for TransformLexCacheScopeGuard {
    fn drop(&mut self) {
        ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
            caches.borrow_mut().pop();
        });
    }
}

pub(crate) fn with_transform_lex_cache<T>(operation: impl FnOnce() -> T) -> T {
    ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
        caches.borrow_mut().push(TransformLexCacheV0::default());
    });
    let _guard = TransformLexCacheScopeGuard;
    operation()
}

pub(crate) fn lex_cached(source: &str, dialect: StyleDialect) -> CachedLexResultV0 {
    ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let Some(cache) = caches.last_mut() else {
            return CachedLexResultV0 {
                inner: Rc::new(lex(source, dialect)),
            };
        };

        let key = (dialect, source.to_string());
        if let Some(cached) = cache.entries.get(&key) {
            return CachedLexResultV0 {
                inner: Rc::clone(cached),
            };
        }

        let lexed = Rc::new(lex(source, dialect));
        cache.entries.insert(key, Rc::clone(&lexed));
        CachedLexResultV0 { inner: lexed }
    })
}

#[cfg(test)]
mod tests {
    use super::{lex_cached, with_transform_lex_cache};
    use omena_parser::{StyleDialect, with_omena_parser_lex_instrumentation};

    #[test]
    fn transform_lex_cache_materializes_identical_source_once_per_scope() {
        let source = ".button { color: red; }";
        let (token_kinds, instrumentation) = with_omena_parser_lex_instrumentation(|| {
            with_transform_lex_cache(|| {
                let first = lex_cached(source, StyleDialect::Css);
                let second = lex_cached(source, StyleDialect::Css);

                first
                    .tokens()
                    .iter()
                    .zip(second.tokens())
                    .map(|(left, right)| {
                        assert_eq!(left, right);
                        left.kind
                    })
                    .collect::<Vec<_>>()
            })
        });

        assert!(!token_kinds.is_empty());
        assert_eq!(instrumentation.lex_invocation_count, 1);
    }

    #[test]
    fn transform_lex_cache_is_scoped_to_an_execution() {
        let source = ".button { color: red; }";
        let (_, instrumentation) = with_omena_parser_lex_instrumentation(|| {
            with_transform_lex_cache(|| {
                let _ = lex_cached(source, StyleDialect::Css);
            });
            with_transform_lex_cache(|| {
                let _ = lex_cached(source, StyleDialect::Css);
            });
        });

        assert_eq!(instrumentation.lex_invocation_count, 2);
    }
}
