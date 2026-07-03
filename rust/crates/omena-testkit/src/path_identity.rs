use omena_resolver::{canonicalize_module_id_v0, canonicalize_omena_resolver_style_identity_path};
use proptest::prelude::*;

const MIN_NEUTRAL_CASES: u32 = 96;
#[cfg(windows)]
const MIN_WINDOWS_CASES: u32 = 64;

fn safe_segment() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,12}".prop_map(|segment| segment)
}

fn virtual_style_path() -> impl Strategy<Value = String> {
    prop::collection::vec(safe_segment(), 1..5).prop_map(|segments| {
        format!(
            "/virtual-workspace/{}/Style.module.scss",
            segments.join("/")
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: MIN_NEUTRAL_CASES,
        ..ProptestConfig::default()
    })]

    #[test]
    fn path_identity_canonicalization_is_idempotent(path in virtual_style_path()) {
        let once = canonicalize_omena_resolver_style_identity_path(path.as_str());
        let twice = canonicalize_omena_resolver_style_identity_path(once.as_str());

        prop_assert_eq!(twice, once);
    }

    #[test]
    fn distinct_virtual_style_paths_keep_distinct_identities(
        left in safe_segment(),
        right in safe_segment(),
    ) {
        prop_assume!(left != right);

        let left_path = format!("/virtual-workspace/{left}/Style.module.scss");
        let right_path = format!("/virtual-workspace/{right}/Style.module.scss");
        let left_identity = canonicalize_omena_resolver_style_identity_path(left_path.as_str());
        let right_identity = canonicalize_omena_resolver_style_identity_path(right_path.as_str());

        prop_assert_ne!(left_identity, right_identity);
    }

    #[test]
    fn module_canonical_id_round_trips_source_uri(path in virtual_style_path()) {
        let module = canonicalize_module_id_v0(path.clone());
        let round_trip = canonicalize_module_id_v0(module.canonical_id.clone());

        prop_assert_eq!(module.canonical_id.as_str(), path.as_str());
        prop_assert_eq!(round_trip.canonical_id, module.canonical_id);
        prop_assert_eq!(round_trip.source_uri, module.source_uri);
    }
}

#[test]
fn path_identity_neutral_case_floor_is_non_vacuous() {
    let cases = std::hint::black_box(MIN_NEUTRAL_CASES);
    assert!(
        cases >= 64,
        "path identity properties must execute a non-trivial neutral corpus"
    );
}

#[cfg(windows)]
proptest! {
    #![proptest_config(ProptestConfig {
        cases: MIN_WINDOWS_CASES,
        ..ProptestConfig::default()
    })]

    #[test]
    fn path_identity_covers_windows_equivalence_classes(file in safe_segment()) {
        let slash = canonicalize_omena_resolver_style_identity_path(
            format!(r"C:/workspace/src/{file}.module.scss").as_str(),
        );
        let backslash = canonicalize_omena_resolver_style_identity_path(
            format!(r"C:\workspace\src\{file}.module.scss").as_str(),
        );
        prop_assert_eq!(slash, backslash);

        let upper = canonicalize_omena_resolver_style_identity_path(
            format!(r"C:\workspace\src\{file}.module.scss").as_str(),
        );
        let lower = canonicalize_omena_resolver_style_identity_path(
            format!(r"c:\workspace\src\{file}.module.scss").as_str(),
        );
        prop_assert_eq!(upper, lower);

        let normal = canonicalize_omena_resolver_style_identity_path(
            format!(r"C:\workspace\src\{file}.module.scss").as_str(),
        );
        let verbatim = canonicalize_omena_resolver_style_identity_path(
            format!(r"\\?\C:\workspace\src\{file}.module.scss").as_str(),
        );
        prop_assert_eq!(normal, verbatim);

        let unc_backslash = canonicalize_omena_resolver_style_identity_path(
            format!(r"\\server\share\src\{file}.module.scss").as_str(),
        );
        let unc_slash = canonicalize_omena_resolver_style_identity_path(
            format!(r"//server/share/src/{file}.module.scss").as_str(),
        );
        prop_assert_eq!(unc_backslash, unc_slash);
    }
}

#[cfg(windows)]
#[test]
fn path_identity_windows_case_floor_is_non_vacuous() {
    let cases = std::hint::black_box(MIN_WINDOWS_CASES);
    assert!(
        cases >= 32,
        "path identity properties must execute a non-trivial Windows corpus"
    );
}
