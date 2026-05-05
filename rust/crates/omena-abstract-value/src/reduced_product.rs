use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, CompositeClassValueInputV0,
    bottom_class_value, char_set_for_string, char_set_is_subset, composite_class_value,
    intersect_char_sets, meaningful_longest_common_prefix, meaningful_longest_common_suffix,
    prefix_suffix_class_value, top_class_value, union_char_sets,
};

pub(crate) fn intersect_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let left = ClassValueReductionFacts::from_abstract_value(left)?;
    let right = ClassValueReductionFacts::from_abstract_value(right)?;
    left.intersect(&right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin))
}

pub(crate) fn join_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let left = ClassValueReductionFacts::from_abstract_value(left)?;
    let right = ClassValueReductionFacts::from_abstract_value(right)?;
    left.join(&right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin))
}

pub(crate) fn concatenate_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let left = ClassValueReductionFacts::from_abstract_value(left)?;
    let right = ClassValueReductionFacts::from_abstract_value(right)?;
    left.concat(&right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeConcat))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassValueReductionFacts {
    prefix: Option<String>,
    suffix: Option<String>,
    min_length: Option<usize>,
    must_chars: String,
    allowed_chars: Option<String>,
}

impl ClassValueReductionFacts {
    fn from_abstract_value(value: &AbstractClassValueV0) -> Option<Self> {
        match value {
            AbstractClassValueV0::Bottom
            | AbstractClassValueV0::Exact { .. }
            | AbstractClassValueV0::FiniteSet { .. } => None,
            AbstractClassValueV0::Prefix { prefix, .. } => Some(Self {
                prefix: Some(prefix.clone()),
                suffix: None,
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::Suffix { suffix, .. } => Some(Self {
                prefix: None,
                suffix: Some(suffix.clone()),
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                ..
            } => Some(Self {
                prefix: Some(prefix.clone()),
                suffix: Some(suffix.clone()),
                min_length: Some(*min_length),
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::CharInclusion {
                must_chars,
                may_chars,
                may_include_other_chars,
                ..
            } => Some(Self {
                prefix: None,
                suffix: None,
                min_length: None,
                must_chars: must_chars.clone(),
                allowed_chars: (!*may_include_other_chars).then_some(may_chars.clone()),
            }),
            AbstractClassValueV0::Composite {
                prefix,
                suffix,
                min_length,
                must_chars,
                may_chars,
                may_include_other_chars,
                ..
            } => Some(Self {
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                min_length: *min_length,
                must_chars: must_chars.clone(),
                allowed_chars: (!*may_include_other_chars).then_some(may_chars.clone()),
            }),
            AbstractClassValueV0::Top => Some(Self {
                prefix: None,
                suffix: None,
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
        }
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        let prefix = intersect_prefixes(self.prefix.as_deref(), other.prefix.as_deref())?;
        let suffix = intersect_suffixes(self.suffix.as_deref(), other.suffix.as_deref())?;
        let min_length = max_optional_usize(self.min_length, other.min_length);
        let edge_chars = char_set_for_string(format!(
            "{}{}",
            prefix.as_deref().unwrap_or(""),
            suffix.as_deref().unwrap_or("")
        ));
        let must_chars = union_char_sets(
            &union_char_sets(&self.must_chars, &other.must_chars),
            &edge_chars,
        );
        let allowed_chars = intersect_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if let Some(allowed_chars) = &allowed_chars
            && !char_set_is_subset(&must_chars, allowed_chars)
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn join(&self, other: &Self) -> Option<Self> {
        let prefix = join_prefixes(self.prefix.as_deref(), other.prefix.as_deref());
        let suffix = join_suffixes(self.suffix.as_deref(), other.suffix.as_deref());
        let min_length = Some(self.lower_bound_length().min(other.lower_bound_length()));
        let must_chars = intersect_char_sets(&self.guaranteed_chars(), &other.guaranteed_chars());
        let allowed_chars = join_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if prefix.is_none() && suffix.is_none() && must_chars.is_empty() && allowed_chars.is_none()
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn concat(&self, other: &Self) -> Option<Self> {
        let prefix = self.prefix.clone();
        let suffix = other.suffix.clone();
        let min_length = Some(self.lower_bound_length() + other.lower_bound_length());
        let must_chars = union_char_sets(&self.guaranteed_chars(), &other.guaranteed_chars());
        let allowed_chars = join_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if prefix.is_none() && suffix.is_none() && must_chars.is_empty() && allowed_chars.is_none()
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn lower_bound_length(&self) -> usize {
        self.min_length.unwrap_or_else(|| {
            let edge_len = self.prefix.as_deref().unwrap_or("").len()
                + self.suffix.as_deref().unwrap_or("").len();
            edge_len.max(self.must_chars.chars().count())
        })
    }

    fn guaranteed_chars(&self) -> String {
        union_char_sets(
            &self.must_chars,
            &char_set_for_string(format!(
                "{}{}",
                self.prefix.as_deref().unwrap_or(""),
                self.suffix.as_deref().unwrap_or("")
            )),
        )
    }

    fn into_abstract_value(
        self,
        provenance: AbstractClassValueProvenanceV0,
    ) -> AbstractClassValueV0 {
        let edge_chars = char_set_for_string(format!(
            "{}{}",
            self.prefix.as_deref().unwrap_or(""),
            self.suffix.as_deref().unwrap_or("")
        ));
        if self.allowed_chars.is_none()
            && (!edge_chars.is_empty() || self.prefix.is_some() || self.suffix.is_some())
            && char_set_is_subset(&self.must_chars, &edge_chars)
        {
            return prefix_suffix_class_value(
                self.prefix.unwrap_or_default(),
                self.suffix.unwrap_or_default(),
                self.min_length,
                Some(provenance),
            );
        }

        let may_include_other_chars = self.allowed_chars.is_none();
        let may_chars = self
            .allowed_chars
            .unwrap_or_else(|| self.must_chars.clone());

        if self.prefix.is_none()
            && self.suffix.is_none()
            && self.must_chars.is_empty()
            && may_include_other_chars
        {
            return top_class_value();
        }

        if self.prefix.is_none()
            && self.suffix.is_none()
            && self.must_chars.is_empty()
            && may_chars.is_empty()
            && !may_include_other_chars
        {
            return bottom_class_value();
        }

        composite_class_value(CompositeClassValueInputV0 {
            prefix: self.prefix,
            suffix: self.suffix,
            min_length: self.min_length,
            must_chars: self.must_chars,
            may_chars,
            may_include_other_chars,
            provenance: Some(provenance),
        })
    }
}

fn intersect_prefixes(left: Option<&str>, right: Option<&str>) -> Option<Option<String>> {
    match (left, right) {
        (None, None) => Some(None),
        (Some(value), None) | (None, Some(value)) => Some(Some(value.to_string())),
        (Some(left), Some(right)) if left.starts_with(right) => Some(Some(left.to_string())),
        (Some(left), Some(right)) if right.starts_with(left) => Some(Some(right.to_string())),
        (Some(_), Some(_)) => None,
    }
}

fn intersect_suffixes(left: Option<&str>, right: Option<&str>) -> Option<Option<String>> {
    match (left, right) {
        (None, None) => Some(None),
        (Some(value), None) | (None, Some(value)) => Some(Some(value.to_string())),
        (Some(left), Some(right)) if left.ends_with(right) => Some(Some(left.to_string())),
        (Some(left), Some(right)) if right.ends_with(left) => Some(Some(right.to_string())),
        (Some(_), Some(_)) => None,
    }
}

fn join_prefixes(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let prefix = meaningful_longest_common_prefix(&[left.to_string(), right.to_string()]);
            (!prefix.is_empty()).then_some(prefix)
        }
        _ => None,
    }
}

fn join_suffixes(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let suffix = meaningful_longest_common_suffix(&[left.to_string(), right.to_string()]);
            (!suffix.is_empty()).then_some(suffix)
        }
        _ => None,
    }
}

fn max_optional_usize(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn intersect_allowed_char_sets(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(intersect_char_sets(left, right)),
        (Some(value), None) | (None, Some(value)) => Some(value.to_string()),
        (None, None) => None,
    }
}

fn join_allowed_char_sets(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(union_char_sets(left, right)),
        _ => None,
    }
}
