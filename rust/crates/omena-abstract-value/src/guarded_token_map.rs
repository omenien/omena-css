use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    FALSE_NODE_ID_V0, FirstWitnessErrorV0, FirstWitnessManagerConfigV0, FirstWitnessManagerV0,
    NodeId, TRUE_NODE_ID_V0, VariableOrderRegistrationV0,
};
use omena_syntax::ident::{CanonicalClassKeyV0, ClassNameV0};
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardedTokenLanguageV0 {
    Concrete(CanonicalClassKeyV0),
    Symbolic { raw_language: String },
}

impl GuardedTokenLanguageV0 {
    pub fn concrete(authored: impl Into<String>) -> Self {
        Self::Concrete(ClassNameV0::new(authored).canonical_key())
    }

    pub fn symbolic(raw_language: impl Into<String>) -> Self {
        Self::Symbolic {
            raw_language: raw_language.into(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Concrete(token) => token.as_str(),
            Self::Symbolic { raw_language } => raw_language,
        }
    }
}

impl Serialize for GuardedTokenLanguageV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardAtomV0 {
    pub atom: String,
    pub polarity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenObserverProjectionV0 {
    pub raw_string: String,
    pub ordered_word: Vec<String>,
    pub support_set: BTreeSet<String>,
}

impl TokenObserverProjectionV0 {
    pub fn exact(token: &GuardedTokenLanguageV0) -> Self {
        let token = token.label().to_string();
        Self {
            raw_string: token.clone(),
            ordered_word: vec![token.clone()],
            support_set: BTreeSet::from([token]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedTokenInputV0 {
    pub token: GuardedTokenLanguageV0,
    pub guards: Vec<GuardAtomV0>,
    pub observers: TokenObserverProjectionV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedTokenMapInputV0 {
    pub tokens: Vec<GuardedTokenInputV0>,
    pub site_usage_guards: Vec<GuardAtomV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardedTokenObserverV0 {
    RawString,
    OrderedWord,
    SupportSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceConditionV0 {
    pub root: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedTokenMapEntryV0 {
    pub token: GuardedTokenLanguageV0,
    pub condition: PresenceConditionV0,
    pub observers: TokenObserverProjectionV0,
}

#[derive(Debug, Clone)]
pub struct GuardedTokenMapV0 {
    manager: FirstWitnessManagerV0,
    entries: Vec<GuardedTokenMapEntryV0>,
    site_usage: PresenceConditionV0,
}

impl GuardedTokenMapV0 {
    /// Builds a free-boolean condition map.
    ///
    /// Treating predicates as independent atoms enlarges the assignment space,
    /// so an unsatisfiable or tautological result remains sound under later
    /// refinement. Precision may be lost. Numeric predicate relationships are
    /// deliberately not inferred here.
    pub fn build(input: GuardedTokenMapInputV0) -> Result<Self, FirstWitnessErrorV0> {
        Self::build_with_config(input, FirstWitnessManagerConfigV0::default())
    }

    pub fn build_with_config(
        input: GuardedTokenMapInputV0,
        config: FirstWitnessManagerConfigV0,
    ) -> Result<Self, FirstWitnessErrorV0> {
        let first_appearance = input
            .tokens
            .iter()
            .flat_map(|token| token.guards.iter())
            .chain(input.site_usage_guards.iter())
            .map(|guard| guard.atom.clone())
            .collect::<Vec<_>>();
        let order = VariableOrderRegistrationV0::site_first_appearance(first_appearance)?;
        let mut manager = FirstWitnessManagerV0::new(order, config);
        let site_usage = condition_from_guards(&mut manager, &input.site_usage_guards)?;
        let mut entries = BTreeMap::<GuardedTokenLanguageV0, GuardedTokenMapEntryV0>::new();
        for token in input.tokens {
            let condition = condition_from_guards(&mut manager, &token.guards)?;
            if let Some(existing) = entries.get_mut(&token.token) {
                existing.condition.root = manager.or(existing.condition.root, condition)?;
            } else {
                entries.insert(
                    token.token.clone(),
                    GuardedTokenMapEntryV0 {
                        token: token.token,
                        condition: PresenceConditionV0 { root: condition },
                        observers: token.observers,
                    },
                );
            }
        }
        Ok(Self {
            manager,
            entries: entries.into_values().collect(),
            site_usage: PresenceConditionV0 { root: site_usage },
        })
    }

    pub fn entries(&self) -> &[GuardedTokenMapEntryV0] {
        &self.entries
    }

    pub fn manager(&self) -> &FirstWitnessManagerV0 {
        &self.manager
    }

    pub fn condition(&self, token: &GuardedTokenLanguageV0) -> Option<PresenceConditionV0> {
        self.entry(token).map(|entry| entry.condition.clone())
    }

    pub fn is_must(&self, token: &GuardedTokenLanguageV0) -> bool {
        self.entry(token)
            .is_some_and(|entry| self.manager.is_tautology(entry.condition.root))
    }

    pub fn is_may(&self, token: &GuardedTokenLanguageV0) -> bool {
        self.entry(token)
            .is_some_and(|entry| self.manager.is_satisfiable(entry.condition.root))
    }

    pub fn is_dead_css(
        &mut self,
        token: &GuardedTokenLanguageV0,
    ) -> Result<bool, FirstWitnessErrorV0> {
        let condition = self
            .entry(token)
            .map(|entry| entry.condition.root)
            .unwrap_or(FALSE_NODE_ID_V0);
        let conjunction = self.manager.and(condition, self.site_usage.root)?;
        Ok(!self.manager.is_satisfiable(conjunction))
    }

    pub fn rename_is_safe(
        &self,
        from: &GuardedTokenLanguageV0,
        to: &GuardedTokenLanguageV0,
        observer: GuardedTokenObserverV0,
    ) -> bool {
        let Some(from) = self.entry(from) else {
            return false;
        };
        let Some(to) = self.entry(to) else {
            return false;
        };
        if from.condition.root != to.condition.root {
            return false;
        }
        match observer {
            GuardedTokenObserverV0::RawString => {
                from.observers.raw_string == to.observers.raw_string
            }
            GuardedTokenObserverV0::OrderedWord => {
                from.observers.ordered_word == to.observers.ordered_word
            }
            GuardedTokenObserverV0::SupportSet => {
                from.observers.support_set == to.observers.support_set
            }
        }
    }

    pub fn reclaim_if_due(
        &mut self,
    ) -> Result<Option<omena_cascade::FirstWitnessRebuildReportV0>, FirstWitnessErrorV0> {
        let mut roots = self
            .entries
            .iter()
            .map(|entry| entry.condition.root)
            .chain(std::iter::once(self.site_usage.root))
            .collect::<Vec<_>>();
        let report = self.manager.reclaim_if_due(&mut roots)?;
        if report.is_some() {
            for (entry, root) in self.entries.iter_mut().zip(roots.iter().copied()) {
                entry.condition.root = root;
            }
            self.site_usage.root = roots.last().copied().unwrap_or(TRUE_NODE_ID_V0);
        }
        Ok(report)
    }

    fn entry(&self, token: &GuardedTokenLanguageV0) -> Option<&GuardedTokenMapEntryV0> {
        self.entries.iter().find(|entry| &entry.token == token)
    }
}

fn condition_from_guards(
    manager: &mut FirstWitnessManagerV0,
    guards: &[GuardAtomV0],
) -> Result<NodeId, FirstWitnessErrorV0> {
    let mut root = TRUE_NODE_ID_V0;
    for guard in guards {
        let variable = manager.variable(&guard.atom)?;
        let literal = if guard.polarity {
            variable
        } else {
            manager.not(variable)?
        };
        root = manager.and(root, literal)?;
    }
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guarded(token: &str, guards: Vec<GuardAtomV0>) -> GuardedTokenInputV0 {
        let token = GuardedTokenLanguageV0::concrete(token);
        GuardedTokenInputV0 {
            observers: TokenObserverProjectionV0::exact(&token),
            token,
            guards,
        }
    }

    fn atom(atom: &str, polarity: bool) -> GuardAtomV0 {
        GuardAtomV0 {
            atom: atom.to_string(),
            polarity,
        }
    }

    #[test]
    fn diagram_answers_must_may_dead_css_and_observer_rename() -> Result<(), FirstWitnessErrorV0> {
        let always = GuardedTokenLanguageV0::concrete("always");
        let maybe = GuardedTokenLanguageV0::concrete("maybe");
        let impossible = GuardedTokenLanguageV0::concrete("impossible");
        let raw_alias = GuardedTokenLanguageV0::symbolic("alias-raw");
        let ordered_alias = GuardedTokenLanguageV0::symbolic("alias-ordered");
        let shared_observers = TokenObserverProjectionV0 {
            raw_string: "different-raw-a".to_string(),
            ordered_word: vec!["same".to_string()],
            support_set: BTreeSet::from(["same".to_string()]),
        };
        let mut map = GuardedTokenMapV0::build(GuardedTokenMapInputV0 {
            tokens: vec![
                guarded("always", vec![atom("c", true)]),
                guarded("always", vec![atom("c", false)]),
                guarded("maybe", vec![atom("c", true)]),
                guarded("impossible", vec![atom("c", true), atom("c", false)]),
                GuardedTokenInputV0 {
                    token: raw_alias.clone(),
                    guards: vec![atom("c", true)],
                    observers: shared_observers.clone(),
                },
                GuardedTokenInputV0 {
                    token: ordered_alias.clone(),
                    guards: vec![atom("c", true)],
                    observers: TokenObserverProjectionV0 {
                        raw_string: "different-raw-b".to_string(),
                        ..shared_observers
                    },
                },
            ],
            site_usage_guards: Vec::new(),
        })?;
        assert!(map.is_must(&always));
        assert!(map.is_may(&maybe));
        assert!(map.is_dead_css(&impossible)?);
        assert!(!map.is_dead_css(&maybe)?);
        assert!(!map.rename_is_safe(
            &raw_alias,
            &ordered_alias,
            GuardedTokenObserverV0::RawString
        ));
        assert!(map.rename_is_safe(
            &raw_alias,
            &ordered_alias,
            GuardedTokenObserverV0::OrderedWord
        ));
        assert!(map.rename_is_safe(
            &raw_alias,
            &ordered_alias,
            GuardedTokenObserverV0::SupportSet
        ));
        Ok(())
    }

    #[test]
    fn dead_css_changes_when_the_site_guard_becomes_satisfiable() -> Result<(), FirstWitnessErrorV0>
    {
        let token = GuardedTokenLanguageV0::concrete("conditional");
        let build = |usage| {
            GuardedTokenMapV0::build(GuardedTokenMapInputV0 {
                tokens: vec![guarded("conditional", vec![atom("c", true)])],
                site_usage_guards: usage,
            })
        };
        let mut dead = build(vec![atom("c", false)])?;
        let mut alive = build(Vec::new())?;
        assert!(dead.is_dead_css(&token)?);
        assert!(!alive.is_dead_css(&token)?);
        Ok(())
    }
}
