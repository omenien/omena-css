use serde::Serialize;
use std::fmt::Debug;

pub trait ProvenanceSemiringV0: Default + Clone + PartialEq + Eq + Serialize {
    const IDENTIFIER: &'static str;
    type Element: Clone + Debug + PartialEq + Eq + Serialize;

    fn semiring_identifier(&self) -> &'static str {
        Self::IDENTIFIER
    }

    fn zero(&self) -> Self::Element;
    fn one(&self) -> Self::Element;
    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element;
    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element;
    fn idempotent_addition(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceSemiringLawReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_identifier: &'static str,
    pub fixture_count: usize,
    pub additive_identity_holds: bool,
    pub multiplicative_identity_holds: bool,
    pub zero_is_absorbing: bool,
    pub addition_associative: bool,
    pub multiplication_associative: bool,
    pub multiplication_distributes_over_addition: bool,
    pub additive_idempotence_matches_descriptor: bool,
    pub all_fixture_laws_hold: bool,
}

pub fn verify_provenance_semiring_laws_on_fixtures<K: ProvenanceSemiringV0>(
    semiring: &K,
    fixtures: &[K::Element],
) -> ProvenanceSemiringLawReportV0 {
    let zero = semiring.zero();
    let one = semiring.one();
    let additive_identity_holds = fixtures
        .iter()
        .all(|value| semiring.add(value, &zero) == *value && semiring.add(&zero, value) == *value);
    let multiplicative_identity_holds = fixtures.iter().all(|value| {
        semiring.multiply(value, &one) == *value && semiring.multiply(&one, value) == *value
    });
    let zero_is_absorbing = fixtures.iter().all(|value| {
        semiring.multiply(value, &zero) == zero && semiring.multiply(&zero, value) == zero
    });
    let addition_associative = fixtures.iter().all(|lhs| {
        fixtures.iter().all(|mid| {
            fixtures.iter().all(|rhs| {
                semiring.add(&semiring.add(lhs, mid), rhs)
                    == semiring.add(lhs, &semiring.add(mid, rhs))
            })
        })
    });
    let multiplication_associative = fixtures.iter().all(|lhs| {
        fixtures.iter().all(|mid| {
            fixtures.iter().all(|rhs| {
                semiring.multiply(&semiring.multiply(lhs, mid), rhs)
                    == semiring.multiply(lhs, &semiring.multiply(mid, rhs))
            })
        })
    });
    let multiplication_distributes_over_addition = fixtures.iter().all(|lhs| {
        fixtures.iter().all(|mid| {
            fixtures.iter().all(|rhs| {
                semiring.multiply(lhs, &semiring.add(mid, rhs))
                    == semiring.add(&semiring.multiply(lhs, mid), &semiring.multiply(lhs, rhs))
                    && semiring.multiply(&semiring.add(mid, rhs), lhs)
                        == semiring.add(&semiring.multiply(mid, lhs), &semiring.multiply(rhs, lhs))
            })
        })
    });
    let additive_idempotence_matches_descriptor = if semiring.idempotent_addition() {
        fixtures
            .iter()
            .all(|value| semiring.add(value, value) == *value)
    } else {
        fixtures
            .iter()
            .any(|value| semiring.add(value, value) != *value)
    };
    let all_fixture_laws_hold = additive_identity_holds
        && multiplicative_identity_holds
        && zero_is_absorbing
        && addition_associative
        && multiplication_associative
        && multiplication_distributes_over_addition
        && additive_idempotence_matches_descriptor;

    ProvenanceSemiringLawReportV0 {
        schema_version: "0",
        product: "omena-abstract-value.provenance-semiring-law-report",
        layer_marker: "qtt-graded",
        feature_gate: "qtt-provenance",
        semiring_identifier: semiring.semiring_identifier(),
        fixture_count: fixtures.len(),
        additive_identity_holds,
        multiplicative_identity_holds,
        zero_is_absorbing,
        addition_associative,
        multiplication_associative,
        multiplication_distributes_over_addition,
        additive_idempotence_matches_descriptor,
        all_fixture_laws_hold,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lin01ProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: &'static str,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl Lin01ProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "0",
            one: "1",
            addition: "or",
            multiplication: "andThen",
            idempotent_addition: true,
        }
    }
}

impl Default for Lin01ProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for Lin01ProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "lin01";
    type Element = bool;

    fn zero(&self) -> Self::Element {
        false
    }

    fn one(&self) -> Self::Element {
        true
    }

    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        *lhs || *rhs
    }

    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        *lhs && *rhs
    }

    fn idempotent_addition(&self) -> bool {
        self.idempotent_addition
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalCountProvenanceSemiringV0 {
    pub zero: u8,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl NaturalCountProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            one: 1,
            addition: "plus",
            multiplication: "times",
            idempotent_addition: false,
        }
    }
}

impl Default for NaturalCountProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for NaturalCountProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "naturalCount";
    type Element = u16;

    fn zero(&self) -> Self::Element {
        self.zero.into()
    }

    fn one(&self) -> Self::Element {
        self.one.into()
    }

    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        lhs.saturating_add(*rhs)
    }

    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        lhs.saturating_mul(*rhs)
    }

    fn idempotent_addition(&self) -> bool {
        self.idempotent_addition
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TropicalCostV0 {
    Finite(u16),
    Infinity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TropicalProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl TropicalProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "infinity",
            one: 0,
            addition: "min",
            multiplication: "plus",
            idempotent_addition: true,
        }
    }
}

impl Default for TropicalProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for TropicalProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "tropical";
    type Element = TropicalCostV0;

    fn zero(&self) -> Self::Element {
        TropicalCostV0::Infinity
    }

    fn one(&self) -> Self::Element {
        TropicalCostV0::Finite(self.one.into())
    }

    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        (*lhs).min(*rhs)
    }

    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        match (lhs, rhs) {
            (TropicalCostV0::Finite(lhs), TropicalCostV0::Finite(rhs)) => {
                TropicalCostV0::Finite(lhs.saturating_add(*rhs))
            }
            _ => TropicalCostV0::Infinity,
        }
    }

    fn idempotent_addition(&self) -> bool {
        self.idempotent_addition
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViterbiProvenanceSemiringV0 {
    pub zero: u8,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl ViterbiProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            one: 1,
            addition: "max",
            multiplication: "times",
            idempotent_addition: true,
        }
    }
}

impl Default for ViterbiProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for ViterbiProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "viterbi";
    type Element = u8;

    fn zero(&self) -> Self::Element {
        self.zero
    }

    fn one(&self) -> Self::Element {
        self.one
    }

    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        (*lhs).max(*rhs)
    }

    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        lhs.saturating_mul(*rhs)
    }

    fn idempotent_addition(&self) -> bool {
        self.idempotent_addition
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SecurityLabelV0 {
    Public,
    Trusted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityLabelProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: &'static str,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl SecurityLabelProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "public",
            one: "trusted",
            addition: "leastUpperBound",
            multiplication: "flowThen",
            idempotent_addition: true,
        }
    }
}

impl Default for SecurityLabelProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for SecurityLabelProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "securityLabel";
    type Element = SecurityLabelV0;

    fn zero(&self) -> Self::Element {
        SecurityLabelV0::Public
    }

    fn one(&self) -> Self::Element {
        SecurityLabelV0::Trusted
    }

    fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        (*lhs).max(*rhs)
    }

    fn multiply(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element {
        (*lhs).min(*rhs)
    }

    fn idempotent_addition(&self) -> bool {
        self.idempotent_addition
    }
}

pub fn m4_alpha_provenance_semiring_law_reports_v0() -> Vec<ProvenanceSemiringLawReportV0> {
    vec![
        verify_provenance_semiring_laws_on_fixtures(
            &Lin01ProvenanceSemiringV0::new(),
            &[false, true],
        ),
        verify_provenance_semiring_laws_on_fixtures(
            &NaturalCountProvenanceSemiringV0::new(),
            &[0, 1, 2, 3],
        ),
        verify_provenance_semiring_laws_on_fixtures(
            &TropicalProvenanceSemiringV0::new(),
            &[
                TropicalCostV0::Finite(0),
                TropicalCostV0::Finite(1),
                TropicalCostV0::Finite(3),
                TropicalCostV0::Infinity,
            ],
        ),
        verify_provenance_semiring_laws_on_fixtures(&ViterbiProvenanceSemiringV0::new(), &[0, 1]),
        verify_provenance_semiring_laws_on_fixtures(
            &SecurityLabelProvenanceSemiringV0::new(),
            &[SecurityLabelV0::Public, SecurityLabelV0::Trusted],
        ),
    ]
}
