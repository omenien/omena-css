use serde::Serialize;

pub trait ProvenanceSemiringV0: Default + Clone + PartialEq + Eq + Serialize {
    const IDENTIFIER: &'static str;

    fn semiring_identifier(&self) -> &'static str {
        Self::IDENTIFIER
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
}
