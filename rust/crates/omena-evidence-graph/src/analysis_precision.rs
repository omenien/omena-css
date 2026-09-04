use serde::{Deserialize, Serialize};

use super::{
    ContextPrecisionV1, EffectivePrecisionV1, FlowPrecisionV1, ProviderCompletenessV1,
    RevisionIdentityV1, ValueDomainPrecisionV1, WorldAssumptionV1,
};

/// The six independent axes used to explain the precision of an analysis result.
///
/// Construction is intentionally limited to this type's named constructor family. The
/// compile-fail/must-compile pairs below keep the public boundary executable in doctests.
///
/// A downstream crate cannot add an inherent constructor (`E0116`):
///
/// ```compile_fail,E0116
/// use omena_evidence_graph::{
///     AnalysisPrecisionV1, ContextPrecisionV1, FlowPrecisionV1, ProviderCompletenessV1,
///     RevisionIdentityV1, ValueDomainPrecisionV1, WorldAssumptionV1,
/// };
///
/// impl AnalysisPrecisionV1 {
///     fn forged() -> Self {
///         Self {
///             value_domain: ValueDomainPrecisionV1::Unknown,
///             flow: FlowPrecisionV1::Unknown,
///             context: ContextPrecisionV1::Unknown,
///             provider_completeness: ProviderCompletenessV1::Unknown,
///             world_assumption: WorldAssumptionV1::Unknown,
///             revision: RevisionIdentityV1::Unknown,
///         }
///     }
/// }
/// ```
///
/// A downstream crate can define an extension trait that delegates to the family:
///
/// ```
/// use omena_evidence_graph::AnalysisPrecisionV1;
///
/// trait PrecisionFloor {
///     fn floor() -> AnalysisPrecisionV1;
/// }
/// struct QueryPrecision;
/// impl PrecisionFloor for QueryPrecision {
///     fn floor() -> AnalysisPrecisionV1 {
///         AnalysisPrecisionV1::unknown()
///     }
/// }
/// assert_eq!(QueryPrecision::floor(), AnalysisPrecisionV1::unknown());
/// ```
///
/// An axis cannot be assigned directly (`E0616`):
///
/// ```compile_fail,E0616
/// use omena_evidence_graph::{AnalysisPrecisionV1, ContextPrecisionV1};
///
/// let mut precision = AnalysisPrecisionV1::unknown();
/// precision.context = ContextPrecisionV1::KLimitedCallSite;
/// ```
///
/// The corresponding plain setter remains available:
///
/// ```
/// use omena_evidence_graph::{AnalysisPrecisionV1, ContextPrecisionV1};
///
/// let precision = AnalysisPrecisionV1::unknown()
///     .with_context(ContextPrecisionV1::KLimitedCallSite);
/// assert_eq!(precision.context(), ContextPrecisionV1::KLimitedCallSite);
/// ```
///
/// A trait implementation in another crate cannot return a struct expression (`E0451`):
///
/// ```compile_fail,E0451
/// use omena_evidence_graph::{
///     AnalysisPrecisionV1, ContextPrecisionV1, FlowPrecisionV1, ProviderCompletenessV1,
///     RevisionIdentityV1, ValueDomainPrecisionV1, WorldAssumptionV1,
/// };
///
/// trait BuildsPrecision {
///     fn build() -> AnalysisPrecisionV1;
/// }
/// struct QueryPrecision;
/// impl BuildsPrecision for QueryPrecision {
///     fn build() -> AnalysisPrecisionV1 {
///         AnalysisPrecisionV1 {
///             value_domain: ValueDomainPrecisionV1::Unknown,
///             flow: FlowPrecisionV1::Unknown,
///             context: ContextPrecisionV1::Unknown,
///             provider_completeness: ProviderCompletenessV1::Unknown,
///             world_assumption: WorldAssumptionV1::Unknown,
///             revision: RevisionIdentityV1::Unknown,
///         }
///     }
/// }
/// ```
///
/// The same trait implementation can use the named constructor:
///
/// ```
/// use omena_evidence_graph::{
///     AnalysisPrecisionV1, ContextPrecisionV1, FlowPrecisionV1, ProviderCompletenessV1,
///     RevisionIdentityV1, ValueDomainPrecisionV1, WorldAssumptionV1,
/// };
///
/// trait BuildsPrecision {
///     fn build() -> AnalysisPrecisionV1;
/// }
/// struct QueryPrecision;
/// impl BuildsPrecision for QueryPrecision {
///     fn build() -> AnalysisPrecisionV1 {
///         AnalysisPrecisionV1::from_axes(
///             ValueDomainPrecisionV1::Unknown,
///             FlowPrecisionV1::Unknown,
///             ContextPrecisionV1::Unknown,
///             ProviderCompletenessV1::Unknown,
///             WorldAssumptionV1::Unknown,
///             RevisionIdentityV1::Unknown,
///         )
///     }
/// }
/// assert_eq!(QueryPrecision::build(), AnalysisPrecisionV1::unknown());
/// ```
///
/// Functional record update cannot bypass the private fields (`E0451`):
///
/// ```compile_fail,E0451
/// use omena_evidence_graph::{AnalysisPrecisionV1, ContextPrecisionV1};
///
/// let base = AnalysisPrecisionV1::unknown();
/// let _ = AnalysisPrecisionV1 {
///     context: ContextPrecisionV1::KLimitedCallSite,
///     ..base
/// };
/// ```
///
/// The equivalent family member is public and explicit:
///
/// ```
/// use omena_evidence_graph::{AnalysisPrecisionV1, ContextPrecisionV1};
///
/// let precision = AnalysisPrecisionV1::unknown()
///     .with_context(ContextPrecisionV1::KLimitedCallSite);
/// assert_eq!(precision.context(), ContextPrecisionV1::KLimitedCallSite);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisPrecisionV1 {
    value_domain: ValueDomainPrecisionV1,
    #[serde(rename = "flowSensitivity")]
    flow: FlowPrecisionV1,
    #[serde(rename = "contextSensitivity")]
    context: ContextPrecisionV1,
    provider_completeness: ProviderCompletenessV1,
    world_assumption: WorldAssumptionV1,
    #[serde(rename = "revisionAxis")]
    revision: RevisionIdentityV1,
}

impl AnalysisPrecisionV1 {
    pub const fn unknown() -> Self {
        Self::from_axes(
            ValueDomainPrecisionV1::Unknown,
            FlowPrecisionV1::Unknown,
            ContextPrecisionV1::Unknown,
            ProviderCompletenessV1::Unknown,
            WorldAssumptionV1::Unknown,
            RevisionIdentityV1::Unknown,
        )
    }

    pub const fn from_axes(
        value_domain: ValueDomainPrecisionV1,
        flow: FlowPrecisionV1,
        context: ContextPrecisionV1,
        provider_completeness: ProviderCompletenessV1,
        world_assumption: WorldAssumptionV1,
        revision: RevisionIdentityV1,
    ) -> Self {
        Self {
            value_domain,
            flow,
            context,
            provider_completeness,
            world_assumption,
            revision,
        }
    }

    #[doc(hidden)]
    pub const fn from_axes_for_tests(
        value_domain: ValueDomainPrecisionV1,
        flow: FlowPrecisionV1,
        context: ContextPrecisionV1,
        provider_completeness: ProviderCompletenessV1,
        world_assumption: WorldAssumptionV1,
        revision: RevisionIdentityV1,
    ) -> Self {
        Self::from_axes(
            value_domain,
            flow,
            context,
            provider_completeness,
            world_assumption,
            revision,
        )
    }

    pub const fn value_domain(self) -> ValueDomainPrecisionV1 {
        self.value_domain
    }

    pub const fn flow(self) -> FlowPrecisionV1 {
        self.flow
    }

    pub const fn context(self) -> ContextPrecisionV1 {
        self.context
    }

    pub const fn provider_completeness(self) -> ProviderCompletenessV1 {
        self.provider_completeness
    }

    pub const fn world_assumption(self) -> WorldAssumptionV1 {
        self.world_assumption
    }

    pub const fn revision(self) -> RevisionIdentityV1 {
        self.revision
    }

    pub const fn with_value_domain(self, value_domain: ValueDomainPrecisionV1) -> Self {
        Self {
            value_domain,
            ..self
        }
    }

    pub const fn with_flow(self, flow: FlowPrecisionV1) -> Self {
        Self { flow, ..self }
    }

    pub const fn with_context(self, context: ContextPrecisionV1) -> Self {
        Self { context, ..self }
    }

    pub const fn with_provider_completeness(
        self,
        provider_completeness: ProviderCompletenessV1,
    ) -> Self {
        Self {
            provider_completeness,
            ..self
        }
    }

    pub const fn with_world_assumption(self, world_assumption: WorldAssumptionV1) -> Self {
        Self {
            world_assumption,
            ..self
        }
    }

    pub const fn with_revision(self, revision: RevisionIdentityV1) -> Self {
        Self { revision, ..self }
    }

    pub const fn effective_precision(self) -> EffectivePrecisionV1 {
        self.value_domain
            .effective_precision()
            .meet(self.flow.effective_precision())
            .meet(self.context.effective_precision())
            .meet(self.provider_completeness.effective_precision())
            .meet(self.world_assumption.effective_precision())
            .meet(self.revision.effective_precision())
    }
}
