use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScssEvalPathV0 {
    Cst,
    LegacyScanner,
}

thread_local! {
    static SCSS_EVAL_PATH: Cell<ScssEvalPathV0> = const { Cell::new(ScssEvalPathV0::Cst) };
}

pub(crate) fn with_legacy_scss_eval_scanner_path<T>(f: impl FnOnce() -> T) -> T {
    SCSS_EVAL_PATH.with(|path| {
        let previous = path.replace(ScssEvalPathV0::LegacyScanner);
        let _guard = ScssEvalPathGuardV0 { path, previous };
        f()
    })
}

pub(crate) fn use_legacy_scss_eval_scanner_path() -> bool {
    SCSS_EVAL_PATH.with(|path| path.get() == ScssEvalPathV0::LegacyScanner)
}

struct ScssEvalPathGuardV0<'a> {
    path: &'a Cell<ScssEvalPathV0>,
    previous: ScssEvalPathV0,
}

impl Drop for ScssEvalPathGuardV0<'_> {
    fn drop(&mut self) {
        self.path.set(self.previous);
    }
}
