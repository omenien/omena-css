//! Closed-world contract types: module identity, instance keys, linked-module
//! facts, reachability, and the sealed bundle exposed to consumers.

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleIdV0(String);

impl ModuleIdV0 {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ModuleIdV0 {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ModuleIdV0 {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationHashV0(String);

impl ConfigurationHashV0 {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn none() -> Self {
        Self::new("with:none")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ConfigurationHashV0 {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInstanceKeyV0 {
    module: ModuleIdV0,
    configuration: ConfigurationHashV0,
}

impl ModuleInstanceKeyV0 {
    pub fn new(module: ModuleIdV0, configuration: ConfigurationHashV0) -> Self {
        Self {
            module,
            configuration,
        }
    }

    pub fn unconfigured(module: ModuleIdV0) -> Self {
        Self::new(module, ConfigurationHashV0::none())
    }

    pub fn module(&self) -> &ModuleIdV0 {
        &self.module
    }

    pub fn configuration(&self) -> &ConfigurationHashV0 {
        &self.configuration
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldComposesEdgeV0 {
    pub from_module: ModuleInstanceKeyV0,
    pub from_symbol: String,
    pub to_module: ModuleInstanceKeyV0,
    pub to_symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldLinkedModuleV0 {
    pub instance: ModuleInstanceKeyV0,
    pub dependencies: Vec<ModuleInstanceKeyV0>,
    pub composes_edges: Vec<ClosedWorldComposesEdgeV0>,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
}

impl ClosedWorldLinkedModuleV0 {
    pub fn new(instance: ModuleInstanceKeyV0) -> Self {
        Self {
            instance,
            dependencies: Vec::new(),
            composes_edges: Vec::new(),
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: ModuleInstanceKeyV0) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn with_composes_edge(mut self, edge: ClosedWorldComposesEdgeV0) -> Self {
        self.composes_edges.push(edge);
        self
    }

    pub fn with_class_name(mut self, name: impl Into<String>) -> Self {
        self.class_names.push(name.into());
        self
    }

    pub fn with_keyframe_name(mut self, name: impl Into<String>) -> Self {
        self.keyframe_names.push(name.into());
        self
    }

    pub fn with_value_name(mut self, name: impl Into<String>) -> Self {
        self.value_names.push(name.into());
        self
    }

    pub fn with_custom_property_name(mut self, name: impl Into<String>) -> Self {
        self.custom_property_names.push(name.into());
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldSourcePrecisionSummaryV0 {
    pub exact_source_count: usize,
    pub conservative_source_count: usize,
    pub heuristic_source_count: usize,
    pub unknown_source_count: usize,
}

impl ClosedWorldSourcePrecisionSummaryV0 {
    pub(crate) fn merge(&mut self, other: Self) {
        self.exact_source_count = self
            .exact_source_count
            .saturating_add(other.exact_source_count);
        self.conservative_source_count = self
            .conservative_source_count
            .saturating_add(other.conservative_source_count);
        self.heuristic_source_count = self
            .heuristic_source_count
            .saturating_add(other.heuristic_source_count);
        self.unknown_source_count = self
            .unknown_source_count
            .saturating_add(other.unknown_source_count);
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClosedWorldModuleReachabilityEvidenceV0 {
    Supplied,
    #[default]
    ModuleReachabilityInputAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedWorldModuleMetadataV0 {
    module_instance: ModuleInstanceKeyV0,
    interface_hash: Option<String>,
    source_precision: Option<ClosedWorldSourcePrecisionSummaryV0>,
    reachability_evidence: ClosedWorldModuleReachabilityEvidenceV0,
}

impl ClosedWorldModuleMetadataV0 {
    pub fn new(module_instance: ModuleInstanceKeyV0) -> Self {
        Self {
            module_instance,
            interface_hash: None,
            source_precision: None,
            reachability_evidence:
                ClosedWorldModuleReachabilityEvidenceV0::ModuleReachabilityInputAbsent,
        }
    }

    pub fn module_instance(&self) -> &ModuleInstanceKeyV0 {
        &self.module_instance
    }

    pub fn with_interface_hash(mut self, interface_hash: impl Into<String>) -> Self {
        self.interface_hash = Some(interface_hash.into());
        self
    }

    pub fn interface_hash(&self) -> Option<&str> {
        self.interface_hash.as_deref()
    }

    pub fn with_source_precision(
        mut self,
        source_precision: ClosedWorldSourcePrecisionSummaryV0,
    ) -> Self {
        self.source_precision = Some(source_precision);
        self
    }

    pub fn source_precision(&self) -> Option<ClosedWorldSourcePrecisionSummaryV0> {
        self.source_precision
    }

    pub fn with_reachability_evidence(
        mut self,
        reachability_evidence: ClosedWorldModuleReachabilityEvidenceV0,
    ) -> Self {
        self.reachability_evidence = reachability_evidence;
        self
    }

    pub fn reachability_evidence(&self) -> ClosedWorldModuleReachabilityEvidenceV0 {
        self.reachability_evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum ClosedWorldInterfaceHashAvailabilityV0 {
    Known { interface_hash: String },
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldInterfaceHashEntryV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub availability: ClosedWorldInterfaceHashAvailabilityV0,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldInterfaceHashSetV0 {
    entries: Vec<ClosedWorldInterfaceHashEntryV0>,
}

impl ClosedWorldInterfaceHashSetV0 {
    pub(crate) fn new(entries: Vec<ClosedWorldInterfaceHashEntryV0>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[ClosedWorldInterfaceHashEntryV0] {
        &self.entries
    }

    pub fn all_absent(&self) -> bool {
        self.entries.iter().all(|entry| {
            matches!(
                entry.availability,
                ClosedWorldInterfaceHashAvailabilityV0::Absent
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleQualifiedSymbolSetV0 {
    module_instance: ModuleInstanceKeyV0,
    reachable: bool,
    class_names: Vec<String>,
    keyframe_names: Vec<String>,
    value_names: Vec<String>,
    custom_property_names: Vec<String>,
}

impl ModuleQualifiedSymbolSetV0 {
    pub(crate) fn new(
        module_instance: ModuleInstanceKeyV0,
        reachable: bool,
        class_names: Vec<String>,
        keyframe_names: Vec<String>,
        value_names: Vec<String>,
        custom_property_names: Vec<String>,
    ) -> Self {
        Self {
            module_instance,
            reachable,
            class_names,
            keyframe_names,
            value_names,
            custom_property_names,
        }
    }

    pub fn module_instance(&self) -> &ModuleInstanceKeyV0 {
        &self.module_instance
    }

    pub fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }

    pub fn keyframe_names(&self) -> &[String] {
        &self.keyframe_names
    }

    pub fn value_names(&self) -> &[String] {
        &self.value_names
    }

    pub fn custom_property_names(&self) -> &[String] {
        &self.custom_property_names
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachabilityIndexV0 {
    module_instances: Vec<ModuleInstanceKeyV0>,
    module_qualified_symbols: Vec<ModuleQualifiedSymbolSetV0>,
    class_names: Vec<String>,
    keyframe_names: Vec<String>,
    value_names: Vec<String>,
    custom_property_names: Vec<String>,
}

impl ReachabilityIndexV0 {
    pub(crate) fn from_parts(
        module_instances: Vec<ModuleInstanceKeyV0>,
        module_qualified_symbols: Vec<ModuleQualifiedSymbolSetV0>,
        class_names: Vec<String>,
        keyframe_names: Vec<String>,
        value_names: Vec<String>,
        custom_property_names: Vec<String>,
    ) -> Self {
        Self {
            module_instances,
            module_qualified_symbols,
            class_names,
            keyframe_names,
            value_names,
            custom_property_names,
        }
    }

    pub fn module_instances(&self) -> &[ModuleInstanceKeyV0] {
        &self.module_instances
    }

    pub fn module_qualified_symbols(&self) -> &[ModuleQualifiedSymbolSetV0] {
        &self.module_qualified_symbols
    }

    pub fn symbols_for_module(
        &self,
        module_instance: &ModuleInstanceKeyV0,
    ) -> Option<&ModuleQualifiedSymbolSetV0> {
        self.module_qualified_symbols
            .binary_search_by(|entry| entry.module_instance().cmp(module_instance))
            .ok()
            .map(|index| &self.module_qualified_symbols[index])
    }

    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }

    pub fn keyframe_names(&self) -> &[String] {
        &self.keyframe_names
    }

    pub fn value_names(&self) -> &[String] {
        &self.value_names
    }

    pub fn custom_property_names(&self) -> &[String] {
        &self.custom_property_names
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldReachabilityBitsetParityReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_instance_count: usize,
    pub symbol_name_count: usize,
    pub module_qualified_symbols: Vec<ModuleQualifiedSymbolSetV0>,
    pub reachability_equal: bool,
    pub closure_hash_equal: bool,
    pub btreeset_closure_hash: String,
    pub bitset_closure_hash: String,
}

/// Closed-world bundle constructed from linked module facts.
///
/// External callers cannot use field-literal construction:
///
/// ```compile_fail
/// use omena_parser::ClosedWorldBundleV0;
///
/// let _bundle = ClosedWorldBundleV0 {
///     closure_hash: String::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedWorldBundleV0 {
    entrypoints: Vec<ModuleInstanceKeyV0>,
    linked_modules: Vec<ModuleInstanceKeyV0>,
    reachability: ReachabilityIndexV0,
    closure_hash: String,
    #[serde(skip_serializing_if = "ClosedWorldInterfaceHashSetV0::all_absent")]
    interface_hashes: ClosedWorldInterfaceHashSetV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_precision: Option<ClosedWorldSourcePrecisionSummaryV0>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    composes_edges: Vec<ClosedWorldComposesEdgeV0>,
    #[serde(skip)]
    module_reachability_evidence:
        BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
}

impl ClosedWorldBundleV0 {
    pub(crate) fn seal(
        entrypoints: Vec<ModuleInstanceKeyV0>,
        linked_modules: Vec<ModuleInstanceKeyV0>,
        reachability: ReachabilityIndexV0,
        closure_hash: String,
        interface_hashes: ClosedWorldInterfaceHashSetV0,
        source_precision: Option<ClosedWorldSourcePrecisionSummaryV0>,
        composes_edges: Vec<ClosedWorldComposesEdgeV0>,
        module_reachability_evidence: BTreeMap<
            ModuleInstanceKeyV0,
            ClosedWorldModuleReachabilityEvidenceV0,
        >,
    ) -> Self {
        Self {
            entrypoints,
            linked_modules,
            reachability,
            closure_hash,
            interface_hashes,
            source_precision,
            composes_edges,
            module_reachability_evidence,
        }
    }

    pub fn entrypoints(&self) -> &[ModuleInstanceKeyV0] {
        &self.entrypoints
    }

    pub fn linked_modules(&self) -> &[ModuleInstanceKeyV0] {
        &self.linked_modules
    }

    pub fn reachability(&self) -> &ReachabilityIndexV0 {
        &self.reachability
    }

    pub fn closure_hash(&self) -> &str {
        &self.closure_hash
    }

    pub fn module_qualified_ownership_digest(&self) -> String {
        let mut digest = StableModuleOwnershipDigestV0::new();
        let mut modules = self
            .reachability
            .module_qualified_symbols()
            .iter()
            .filter(|symbols| symbols.is_reachable())
            .collect::<Vec<_>>();
        modules.sort_by(|left, right| left.module_instance().cmp(right.module_instance()));

        for symbols in modules {
            digest.module(symbols.module_instance());
            digest.symbol_names("class", symbols.class_names());
            digest.symbol_names("keyframe", symbols.keyframe_names());
            digest.symbol_names("value", symbols.value_names());
            digest.symbol_names("custom-property", symbols.custom_property_names());
        }

        digest.finish_hex()
    }

    pub fn interface_hashes(&self) -> &ClosedWorldInterfaceHashSetV0 {
        &self.interface_hashes
    }

    pub fn source_precision(&self) -> Option<ClosedWorldSourcePrecisionSummaryV0> {
        self.source_precision
    }

    pub fn composes_edges(&self) -> &[ClosedWorldComposesEdgeV0] {
        self.composes_edges.as_slice()
    }

    pub fn module_reachability_evidence(
        &self,
        module_instance: &ModuleInstanceKeyV0,
    ) -> ClosedWorldModuleReachabilityEvidenceV0 {
        self.module_reachability_evidence
            .get(module_instance)
            .copied()
            .unwrap_or_default()
    }
}

struct StableModuleOwnershipDigestV0(u64);

impl StableModuleOwnershipDigestV0 {
    fn new() -> Self {
        let mut digest = Self(0xcbf2_9ce4_8422_2325);
        digest.piece("omena-parser.module-qualified-ownership");
        digest
    }

    fn piece(&mut self, value: &str) {
        for byte in value.as_bytes().iter().copied().chain([0]) {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn module(&mut self, module_instance: &ModuleInstanceKeyV0) {
        self.piece("module");
        self.piece(module_instance.module().as_str());
        self.piece(module_instance.configuration().as_str());
    }

    fn symbol_names(&mut self, kind: &str, names: &[String]) {
        self.piece(kind);
        let names = names
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        for name in names {
            self.piece(name);
        }
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWorldSnapshotV0 {
    reason: String,
}

impl OpenWorldSnapshotV0 {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClosedWorldBundleBuildErrorV0 {
    EmptyEntrypoints,
    MissingEntrypoint {
        module: ModuleInstanceKeyV0,
    },
    MissingDependency {
        module: ModuleInstanceKeyV0,
        dependency: ModuleInstanceKeyV0,
    },
}
