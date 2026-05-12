use crate::{
    AbstractClassValueProvenanceNodeV0, AbstractClassValueProvenanceTreeV0,
    AbstractClassValueProvenanceV0, AbstractClassValueV0, abstract_class_value_kind,
};

pub fn summarize_abstract_class_value_provenance_tree(
    value: &AbstractClassValueV0,
) -> AbstractClassValueProvenanceTreeV0 {
    let value_kind = abstract_class_value_kind(value);
    let value_provenance = abstract_class_value_provenance(value);

    AbstractClassValueProvenanceTreeV0 {
        schema_version: "0",
        product: "omena-abstract-value.provenance-tree",
        value_kind,
        value: value.clone(),
        value_provenance,
        root: AbstractClassValueProvenanceNodeV0 {
            operation: root_operation(value, value_provenance),
            result_kind: value_kind,
            result_provenance: value_provenance,
            detail: root_detail(value),
            reason: root_reason(value, value_provenance),
            children: constraint_children(value),
        },
    }
}

fn abstract_class_value_provenance(
    value: &AbstractClassValueV0,
) -> Option<AbstractClassValueProvenanceV0> {
    match value {
        AbstractClassValueV0::Prefix { provenance, .. }
        | AbstractClassValueV0::Suffix { provenance, .. }
        | AbstractClassValueV0::PrefixSuffix { provenance, .. }
        | AbstractClassValueV0::CharInclusion { provenance, .. }
        | AbstractClassValueV0::Composite { provenance, .. } => *provenance,
        _ => None,
    }
}

fn root_operation(
    value: &AbstractClassValueV0,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> &'static str {
    match provenance {
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars)
        | Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite) => "finiteSetWidening",
        Some(AbstractClassValueProvenanceV0::PrefixJoinLcp) => "prefixJoinLongestCommonPrefix",
        Some(AbstractClassValueProvenanceV0::SuffixJoinLcs) => "suffixJoinLongestCommonSuffix",
        Some(AbstractClassValueProvenanceV0::PrefixSuffixJoin)
        | Some(AbstractClassValueProvenanceV0::CompositeJoin) => "reducedProductJoin",
        Some(AbstractClassValueProvenanceV0::CompositeConcat) => "reducedProductConcat",
        None => match value {
            AbstractClassValueV0::Bottom => "bottomDomain",
            AbstractClassValueV0::Exact { .. } => "exactLiteral",
            AbstractClassValueV0::FiniteSet { .. } => "finiteSetDomain",
            AbstractClassValueV0::Prefix { .. }
            | AbstractClassValueV0::Suffix { .. }
            | AbstractClassValueV0::PrefixSuffix { .. }
            | AbstractClassValueV0::CharInclusion { .. }
            | AbstractClassValueV0::Composite { .. } => "constraintDomain",
            AbstractClassValueV0::Top => "topDomain",
        },
    }
}

fn root_reason(
    value: &AbstractClassValueV0,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> &'static str {
    match provenance {
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars) => {
            "large finite set widened to character constraints"
        }
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite) => {
            "large finite set widened to preserved edge and character constraints"
        }
        Some(AbstractClassValueProvenanceV0::PrefixJoinLcp) => {
            "branch merge retained the meaningful longest common prefix"
        }
        Some(AbstractClassValueProvenanceV0::SuffixJoinLcs) => {
            "branch merge retained the meaningful longest common suffix"
        }
        Some(AbstractClassValueProvenanceV0::PrefixSuffixJoin)
        | Some(AbstractClassValueProvenanceV0::CompositeJoin) => {
            "reduced product combined compatible constraints from multiple domains"
        }
        Some(AbstractClassValueProvenanceV0::CompositeConcat) => {
            "reduced product concatenated compatible constraints without widening to top"
        }
        None => match value {
            AbstractClassValueV0::Bottom => "no class value can satisfy the current constraints",
            AbstractClassValueV0::Exact { .. } => "the class value is known exactly",
            AbstractClassValueV0::FiniteSet { .. } => "the class value is one of a bounded set",
            AbstractClassValueV0::Prefix { .. }
            | AbstractClassValueV0::Suffix { .. }
            | AbstractClassValueV0::PrefixSuffix { .. }
            | AbstractClassValueV0::CharInclusion { .. }
            | AbstractClassValueV0::Composite { .. } => {
                "the class value is represented by explicit domain constraints"
            }
            AbstractClassValueV0::Top => "the class value is unconstrained",
        },
    }
}

fn root_detail(value: &AbstractClassValueV0) -> Option<String> {
    match value {
        AbstractClassValueV0::Exact { value } => Some(format!("value={value}")),
        AbstractClassValueV0::FiniteSet { values } => Some(format!("valueCount={}", values.len())),
        _ => None,
    }
}

fn constraint_children(value: &AbstractClassValueV0) -> Vec<AbstractClassValueProvenanceNodeV0> {
    let mut children = Vec::new();

    match value {
        AbstractClassValueV0::Prefix { prefix, .. } => {
            children.push(constraint_node(
                "prefixConstraint",
                "prefix",
                prefix.clone(),
            ));
        }
        AbstractClassValueV0::Suffix { suffix, .. } => {
            children.push(constraint_node(
                "suffixConstraint",
                "suffix",
                suffix.clone(),
            ));
        }
        AbstractClassValueV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
            ..
        } => {
            children.push(constraint_node(
                "prefixConstraint",
                "prefix",
                prefix.clone(),
            ));
            children.push(constraint_node(
                "suffixConstraint",
                "suffix",
                suffix.clone(),
            ));
            children.push(constraint_node(
                "lengthConstraint",
                "minLength",
                min_length.to_string(),
            ));
        }
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            push_char_constraint_children(
                &mut children,
                must_chars,
                may_chars,
                *may_include_other_chars,
            );
        }
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            if let Some(prefix) = prefix {
                children.push(constraint_node(
                    "prefixConstraint",
                    "prefix",
                    prefix.clone(),
                ));
            }
            if let Some(suffix) = suffix {
                children.push(constraint_node(
                    "suffixConstraint",
                    "suffix",
                    suffix.clone(),
                ));
            }
            if let Some(min_length) = min_length {
                children.push(constraint_node(
                    "lengthConstraint",
                    "minLength",
                    min_length.to_string(),
                ));
            }
            push_char_constraint_children(
                &mut children,
                must_chars,
                may_chars,
                *may_include_other_chars,
            );
        }
        AbstractClassValueV0::Bottom
        | AbstractClassValueV0::Exact { .. }
        | AbstractClassValueV0::FiniteSet { .. }
        | AbstractClassValueV0::Top => {}
    }

    children
}

fn push_char_constraint_children(
    children: &mut Vec<AbstractClassValueProvenanceNodeV0>,
    must_chars: &str,
    may_chars: &str,
    may_include_other_chars: bool,
) {
    if !must_chars.is_empty() {
        children.push(constraint_node(
            "characterMustConstraint",
            "mustChars",
            must_chars.to_string(),
        ));
    }
    if !may_include_other_chars {
        children.push(constraint_node(
            "characterMayConstraint",
            "mayChars",
            may_chars.to_string(),
        ));
    }
}

fn constraint_node(
    operation: &'static str,
    label: &'static str,
    value: String,
) -> AbstractClassValueProvenanceNodeV0 {
    AbstractClassValueProvenanceNodeV0 {
        operation,
        result_kind: "constraint",
        result_provenance: None,
        detail: Some(format!("{label}={value}")),
        reason: "constraint retained by the abstract value domain",
        children: Vec::new(),
    }
}
