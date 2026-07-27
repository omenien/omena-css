use std::fmt;

use crate::ReactiveStateV0;

pub type ChangeComparatorV0 = fn(&ReactiveStateV0, &ReactiveStateV0) -> bool;

/// A named semantic equivalence relation used to decide whether a new node
/// value should propagate.
///
/// There is intentionally no `Default` implementation. A graph constructor
/// cannot create a node without selecting a policy.
#[derive(Clone, Copy)]
pub struct ChangePolicyV0 {
    name: &'static str,
    equivalent: ChangeComparatorV0,
}

impl ChangePolicyV0 {
    pub const fn exact(name: &'static str) -> Self {
        Self {
            name,
            equivalent: ReactiveStateV0::eq,
        }
    }

    pub const fn custom(name: &'static str, equivalent: ChangeComparatorV0) -> Self {
        Self { name, equivalent }
    }

    pub fn name(self) -> &'static str {
        self.name
    }

    pub fn equivalent(self, previous: &ReactiveStateV0, next: &ReactiveStateV0) -> bool {
        (self.equivalent)(previous, next)
    }
}

impl fmt::Debug for ChangePolicyV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChangePolicyV0")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}
