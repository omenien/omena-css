use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::Value;

use crate::{
    OMENA_SIF_VERSION_V1, OmenaSifExportsV1, OmenaSifForwardExportV1, OmenaSifV1,
    compute_omena_sif_interface_hash_v1,
};

pub const OMENA_SIF_EXPORT_KIND_COUNT_V1: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaSifExportKindV1 {
    Variable,
    Mixin,
    Function,
    Placeholder,
    Forward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaSifStructuralChangeKindV0 {
    Removed,
    Changed,
    VisibilityNarrowed,
    Added,
}

impl OmenaSifStructuralChangeKindV0 {
    pub const fn is_breaking(self) -> bool {
        !matches!(self, Self::Added)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifStructuralChangeV0 {
    pub change_kind: OmenaSifStructuralChangeKindV0,
    pub export_kind: OmenaSifExportKindV1,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifExportKindDiffCensusV0 {
    pub export_kind: OmenaSifExportKindV1,
    pub old_export_count: usize,
    pub new_export_count: usize,
    pub identity_count: usize,
    pub unchanged_count: usize,
    pub classified_change_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifStructuralDiffReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub old_canonical_url: String,
    pub new_canonical_url: String,
    pub export_kind_count: usize,
    pub old_export_count: usize,
    pub new_export_count: usize,
    pub identity_count: usize,
    pub unchanged_count: usize,
    pub classified_change_count: usize,
    pub removed_count: usize,
    pub changed_count: usize,
    pub visibility_narrowed_count: usize,
    pub added_count: usize,
    pub breaking_change_count: usize,
    pub breaking: bool,
    pub stored_interface_hashes_valid: bool,
    pub fast_path: Option<&'static str>,
    pub kind_census: Vec<OmenaSifExportKindDiffCensusV0>,
    pub changes: Vec<OmenaSifStructuralChangeV0>,
}

pub fn summarize_omena_sif_structural_diff_v0(
    old: &OmenaSifV1,
    new: &OmenaSifV1,
) -> Result<OmenaSifStructuralDiffReportV0, serde_json::Error> {
    let old_interface_hash =
        compute_omena_sif_interface_hash_v1(old.generator.toolchain_id.as_str(), &old.exports)?;
    let new_interface_hash =
        compute_omena_sif_interface_hash_v1(new.generator.toolchain_id.as_str(), &new.exports)?;
    let stored_interface_hashes_valid = old.sif_version == OMENA_SIF_VERSION_V1
        && new.sif_version == OMENA_SIF_VERSION_V1
        && old.fingerprints.interface_hash == old_interface_hash
        && new.fingerprints.interface_hash == new_interface_hash;

    if stored_interface_hashes_valid
        && old_interface_hash == new_interface_hash
        && export_count(&old.exports) == export_count(&new.exports)
    {
        return Ok(unchanged_report(old, new));
    }

    let mut changes = Vec::new();
    let mut kind_census = Vec::with_capacity(OMENA_SIF_EXPORT_KIND_COUNT_V1);
    let OmenaSifExportsV1 {
        variables: old_variables,
        mixins: old_mixins,
        functions: old_functions,
        placeholders: old_placeholders,
        forwards: old_forwards,
    } = &old.exports;
    let OmenaSifExportsV1 {
        variables: new_variables,
        mixins: new_mixins,
        functions: new_functions,
        placeholders: new_placeholders,
        forwards: new_forwards,
    } = &new.exports;

    classify_exports(
        OmenaSifExportKindV1::Variable,
        old_variables,
        new_variables,
        |item| item.name.as_str(),
        |_, _| OmenaSifStructuralChangeKindV0::Changed,
        &mut changes,
        &mut kind_census,
    )?;
    classify_exports(
        OmenaSifExportKindV1::Mixin,
        old_mixins,
        new_mixins,
        |item| item.name.as_str(),
        |_, _| OmenaSifStructuralChangeKindV0::Changed,
        &mut changes,
        &mut kind_census,
    )?;
    classify_exports(
        OmenaSifExportKindV1::Function,
        old_functions,
        new_functions,
        |item| item.name.as_str(),
        |_, _| OmenaSifStructuralChangeKindV0::Changed,
        &mut changes,
        &mut kind_census,
    )?;
    classify_exports(
        OmenaSifExportKindV1::Placeholder,
        old_placeholders,
        new_placeholders,
        |item| item.name.as_str(),
        |_, _| OmenaSifStructuralChangeKindV0::Changed,
        &mut changes,
        &mut kind_census,
    )?;
    classify_exports(
        OmenaSifExportKindV1::Forward,
        old_forwards,
        new_forwards,
        |item| item.canonical_url.as_str(),
        |old, new| {
            if forward_visibility_narrowed(old, new) {
                OmenaSifStructuralChangeKindV0::VisibilityNarrowed
            } else {
                OmenaSifStructuralChangeKindV0::Changed
            }
        },
        &mut changes,
        &mut kind_census,
    )?;
    debug_assert_eq!(kind_census.len(), OMENA_SIF_EXPORT_KIND_COUNT_V1);

    changes.sort_by(|left, right| {
        (left.export_kind, left.identity.as_str())
            .cmp(&(right.export_kind, right.identity.as_str()))
    });
    let removed_count = count_change_kind(&changes, OmenaSifStructuralChangeKindV0::Removed);
    let changed_count = count_change_kind(&changes, OmenaSifStructuralChangeKindV0::Changed);
    let visibility_narrowed_count =
        count_change_kind(&changes, OmenaSifStructuralChangeKindV0::VisibilityNarrowed);
    let added_count = count_change_kind(&changes, OmenaSifStructuralChangeKindV0::Added);
    let classified_change_count = changes.len();
    let identity_count = kind_census.iter().map(|row| row.identity_count).sum();
    let unchanged_count = kind_census.iter().map(|row| row.unchanged_count).sum();
    let breaking_change_count = removed_count + changed_count + visibility_narrowed_count;
    debug_assert_eq!(
        classified_change_count,
        removed_count + changed_count + visibility_narrowed_count + added_count
    );
    debug_assert_eq!(identity_count, unchanged_count + classified_change_count);

    Ok(OmenaSifStructuralDiffReportV0 {
        schema_version: "0",
        product: "omena-sif.structural-diff",
        old_canonical_url: old.canonical_url.clone(),
        new_canonical_url: new.canonical_url.clone(),
        export_kind_count: OMENA_SIF_EXPORT_KIND_COUNT_V1,
        old_export_count: export_count(&old.exports),
        new_export_count: export_count(&new.exports),
        identity_count,
        unchanged_count,
        classified_change_count,
        removed_count,
        changed_count,
        visibility_narrowed_count,
        added_count,
        breaking_change_count,
        breaking: breaking_change_count > 0,
        stored_interface_hashes_valid,
        fast_path: None,
        kind_census,
        changes,
    })
}

fn unchanged_report(old: &OmenaSifV1, new: &OmenaSifV1) -> OmenaSifStructuralDiffReportV0 {
    let kind_census = unchanged_kind_census(&old.exports, &new.exports);
    let identity_count = kind_census.iter().map(|row| row.identity_count).sum();
    OmenaSifStructuralDiffReportV0 {
        schema_version: "0",
        product: "omena-sif.structural-diff",
        old_canonical_url: old.canonical_url.clone(),
        new_canonical_url: new.canonical_url.clone(),
        export_kind_count: OMENA_SIF_EXPORT_KIND_COUNT_V1,
        old_export_count: export_count(&old.exports),
        new_export_count: export_count(&new.exports),
        identity_count,
        unchanged_count: identity_count,
        classified_change_count: 0,
        removed_count: 0,
        changed_count: 0,
        visibility_narrowed_count: 0,
        added_count: 0,
        breaking_change_count: 0,
        breaking: false,
        stored_interface_hashes_valid: true,
        fast_path: Some("verifiedInterfaceHash"),
        kind_census,
        changes: Vec::new(),
    }
}

fn unchanged_kind_census(
    old: &OmenaSifExportsV1,
    new: &OmenaSifExportsV1,
) -> Vec<OmenaSifExportKindDiffCensusV0> {
    let OmenaSifExportsV1 {
        variables: old_variables,
        mixins: old_mixins,
        functions: old_functions,
        placeholders: old_placeholders,
        forwards: old_forwards,
    } = old;
    let OmenaSifExportsV1 {
        variables: new_variables,
        mixins: new_mixins,
        functions: new_functions,
        placeholders: new_placeholders,
        forwards: new_forwards,
    } = new;
    [
        (
            OmenaSifExportKindV1::Variable,
            old_variables.len(),
            new_variables.len(),
        ),
        (
            OmenaSifExportKindV1::Mixin,
            old_mixins.len(),
            new_mixins.len(),
        ),
        (
            OmenaSifExportKindV1::Function,
            old_functions.len(),
            new_functions.len(),
        ),
        (
            OmenaSifExportKindV1::Placeholder,
            old_placeholders.len(),
            new_placeholders.len(),
        ),
        (
            OmenaSifExportKindV1::Forward,
            old_forwards.len(),
            new_forwards.len(),
        ),
    ]
    .into_iter()
    .map(|(export_kind, old_export_count, new_export_count)| {
        debug_assert_eq!(old_export_count, new_export_count);
        OmenaSifExportKindDiffCensusV0 {
            export_kind,
            old_export_count,
            new_export_count,
            identity_count: old_export_count,
            unchanged_count: old_export_count,
            classified_change_count: 0,
        }
    })
    .collect()
}

#[allow(clippy::too_many_arguments)]
fn classify_exports<T: Eq + Serialize>(
    export_kind: OmenaSifExportKindV1,
    old_items: &[T],
    new_items: &[T],
    identity: impl Fn(&T) -> &str,
    changed_kind: impl Fn(&T, &T) -> OmenaSifStructuralChangeKindV0,
    changes: &mut Vec<OmenaSifStructuralChangeV0>,
    census: &mut Vec<OmenaSifExportKindDiffCensusV0>,
) -> Result<(), serde_json::Error> {
    let old_by_identity = index_exports(old_items, &identity);
    let new_by_identity = index_exports(new_items, &identity);
    let identities = old_by_identity
        .keys()
        .chain(new_by_identity.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let before = changes.len();
    let mut unchanged_count = 0usize;
    for item_identity in &identities {
        match (
            old_by_identity.get(item_identity),
            new_by_identity.get(item_identity),
        ) {
            (Some(old), Some(new)) if old == new => unchanged_count += 1,
            (Some(old), Some(new)) => changes.push(structural_change(
                changed_kind(old, new),
                export_kind,
                item_identity.clone(),
                Some(*old),
                Some(*new),
            )?),
            (Some(old), None) => changes.push(structural_change(
                OmenaSifStructuralChangeKindV0::Removed,
                export_kind,
                item_identity.clone(),
                Some(*old),
                None::<&T>,
            )?),
            (None, Some(new)) => changes.push(structural_change(
                OmenaSifStructuralChangeKindV0::Added,
                export_kind,
                item_identity.clone(),
                None::<&T>,
                Some(*new),
            )?),
            (None, None) => unreachable!("identity is drawn from at least one export map"),
        }
    }
    census.push(OmenaSifExportKindDiffCensusV0 {
        export_kind,
        old_export_count: old_items.len(),
        new_export_count: new_items.len(),
        identity_count: identities.len(),
        unchanged_count,
        classified_change_count: changes.len() - before,
    });
    Ok(())
}

fn index_exports<'a, T>(items: &'a [T], identity: &impl Fn(&T) -> &str) -> BTreeMap<String, &'a T> {
    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut indexed = BTreeMap::new();
    for item in items {
        let base = identity(item).to_string();
        let occurrence = occurrences.entry(base.clone()).or_default();
        *occurrence += 1;
        let key = if *occurrence == 1 {
            base
        } else {
            format!("{base}#{}", *occurrence)
        };
        indexed.insert(key, item);
    }
    indexed
}

fn structural_change<T: Serialize>(
    change_kind: OmenaSifStructuralChangeKindV0,
    export_kind: OmenaSifExportKindV1,
    identity: String,
    before: Option<&T>,
    after: Option<&T>,
) -> Result<OmenaSifStructuralChangeV0, serde_json::Error> {
    Ok(OmenaSifStructuralChangeV0 {
        change_kind,
        export_kind,
        identity,
        before: before.map(serde_json::to_value).transpose()?,
        after: after.map(serde_json::to_value).transpose()?,
    })
}

fn forward_visibility_narrowed(
    old: &OmenaSifForwardExportV1,
    new: &OmenaSifForwardExportV1,
) -> bool {
    if old.prefix != new.prefix {
        return false;
    }
    let old_show = old.show.iter().collect::<BTreeSet<_>>();
    let new_show = new.show.iter().collect::<BTreeSet<_>>();
    let old_hide = old.hide.iter().collect::<BTreeSet<_>>();
    let new_hide = new.hide.iter().collect::<BTreeSet<_>>();
    let show_narrowed = (old_show.is_empty() && !new_show.is_empty())
        || (!old_show.is_empty() && !old_show.is_subset(&new_show));
    let hide_narrowed = !new_hide.is_subset(&old_hide);
    show_narrowed || hide_narrowed
}

fn export_count(exports: &OmenaSifExportsV1) -> usize {
    let OmenaSifExportsV1 {
        variables,
        mixins,
        functions,
        placeholders,
        forwards,
    } = exports;
    variables.len() + mixins.len() + functions.len() + placeholders.len() + forwards.len()
}

fn count_change_kind(
    changes: &[OmenaSifStructuralChangeV0],
    expected: OmenaSifStructuralChangeKindV0,
) -> usize {
    changes
        .iter()
        .filter(|change| change.change_kind == expected)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OmenaSifCallableExportV1, OmenaSifGeneratorV1, OmenaSifParameterV1,
        OmenaSifPlaceholderExportV1, OmenaSifSourceSyntaxV1, OmenaSifSourceV1,
        OmenaSifVariableExportV1,
    };

    #[test]
    fn structural_diff_classifies_all_export_kinds_and_change_polarities()
    -> Result<(), serde_json::Error> {
        let old = sif(OmenaSifExportsV1 {
            variables: vec![variable("gone", "red"), variable("tone", "red")],
            mixins: vec![callable("theme", false)],
            functions: vec![callable("scale", false)],
            placeholders: vec![placeholder("button")],
            forwards: vec![forward("pkg:tokens", &["brand", "space"], &[])],
        })?;
        let new = sif(OmenaSifExportsV1 {
            variables: vec![variable("tone", "blue"), variable("added", "green")],
            mixins: vec![callable("theme", false)],
            functions: vec![callable("scale", true)],
            placeholders: vec![placeholder("button"), placeholder("card")],
            forwards: vec![forward("pkg:tokens", &["brand"], &[])],
        })?;

        let report = summarize_omena_sif_structural_diff_v0(&old, &new)?;
        assert_eq!(report.export_kind_count, OMENA_SIF_EXPORT_KIND_COUNT_V1);
        assert_eq!(report.kind_census.len(), OMENA_SIF_EXPORT_KIND_COUNT_V1);
        assert_eq!(report.removed_count, 1);
        assert_eq!(report.changed_count, 2);
        assert_eq!(report.visibility_narrowed_count, 1);
        assert_eq!(report.added_count, 2);
        assert_eq!(report.classified_change_count, 6);
        assert_eq!(
            report.classified_change_count,
            report.removed_count
                + report.changed_count
                + report.visibility_narrowed_count
                + report.added_count
        );
        assert_eq!(
            report.identity_count,
            report.unchanged_count + report.classified_change_count
        );
        assert!(report.breaking);
        Ok(())
    }

    #[test]
    fn verified_interface_hash_enables_only_the_unchanged_fast_path()
    -> Result<(), serde_json::Error> {
        let old = sif(OmenaSifExportsV1 {
            variables: vec![variable("tone", "red")],
            ..OmenaSifExportsV1::default()
        })?;
        let unchanged = summarize_omena_sif_structural_diff_v0(&old, &old)?;
        assert_eq!(unchanged.fast_path, Some("verifiedInterfaceHash"));
        assert!(unchanged.stored_interface_hashes_valid);
        assert_eq!(unchanged.unchanged_count, 1);

        let mut tampered_json = serde_json::to_value(&old)?;
        tampered_json["exports"]["variables"][0]["valueRepr"] = Value::String("blue".to_string());
        let tampered: OmenaSifV1 = serde_json::from_value(tampered_json)?;
        let changed = summarize_omena_sif_structural_diff_v0(&old, &tampered)?;
        assert_eq!(changed.fast_path, None);
        assert!(!changed.stored_interface_hashes_valid);
        assert_eq!(changed.changed_count, 1);
        Ok(())
    }

    fn sif(exports: OmenaSifExportsV1) -> Result<OmenaSifV1, serde_json::Error> {
        OmenaSifV1::from_static_exports(
            "pkg:test",
            OmenaSifGeneratorV1 {
                name: "fixture".to_string(),
                version: "1".to_string(),
                toolchain_id: "fixture@1".to_string(),
            },
            OmenaSifSourceV1 {
                syntax: OmenaSifSourceSyntaxV1::Scss,
            },
            exports,
            Vec::new(),
            b"fixture",
        )
    }

    fn variable(name: &str, value: &str) -> OmenaSifVariableExportV1 {
        OmenaSifVariableExportV1 {
            name: name.to_string(),
            defaulted: false,
            value_repr: Some(value.to_string()),
        }
    }

    fn callable(name: &str, accepts_content: bool) -> OmenaSifCallableExportV1 {
        OmenaSifCallableExportV1 {
            name: name.to_string(),
            parameters: vec![OmenaSifParameterV1 {
                name: "value".to_string(),
                default_value_repr: None,
                variadic: false,
            }],
            accepts_content,
        }
    }

    fn placeholder(name: &str) -> OmenaSifPlaceholderExportV1 {
        OmenaSifPlaceholderExportV1 {
            name: name.to_string(),
        }
    }

    fn forward(canonical_url: &str, show: &[&str], hide: &[&str]) -> OmenaSifForwardExportV1 {
        OmenaSifForwardExportV1 {
            canonical_url: canonical_url.to_string(),
            prefix: None,
            show: show.iter().map(|value| (*value).to_string()).collect(),
            hide: hide.iter().map(|value| (*value).to_string()).collect(),
        }
    }
}
