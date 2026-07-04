//! Typed closed-world inputs shared by the linker and transform consumers.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
pub struct ClosedWorldLinkedModuleV0 {
    pub instance: ModuleInstanceKeyV0,
    pub dependencies: Vec<ModuleInstanceKeyV0>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachabilityIndexV0 {
    module_instances: Vec<ModuleInstanceKeyV0>,
    class_names: Vec<String>,
    keyframe_names: Vec<String>,
    value_names: Vec<String>,
    custom_property_names: Vec<String>,
}

impl ReachabilityIndexV0 {
    pub fn module_instances(&self) -> &[ModuleInstanceKeyV0] {
        &self.module_instances
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
}

impl ClosedWorldBundleV0 {
    pub fn try_from_linked_modules(
        entrypoints: Vec<ModuleInstanceKeyV0>,
        linked_modules: Vec<ClosedWorldLinkedModuleV0>,
    ) -> Result<Self, ClosedWorldBundleBuildErrorV0> {
        if entrypoints.is_empty() {
            return Err(ClosedWorldBundleBuildErrorV0::EmptyEntrypoints);
        }

        let mut by_instance = BTreeMap::new();
        for module in linked_modules {
            by_instance.insert(module.instance.clone(), module);
        }

        let reachability = compute_reachability(entrypoints.as_slice(), &by_instance)?;
        let linked_modules = reachability.module_instances.clone();
        let closure_hash = stable_closure_hash(entrypoints.as_slice(), &by_instance, &reachability);

        Ok(Self {
            entrypoints,
            linked_modules,
            reachability,
            closure_hash,
        })
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
}

pub fn summarize_closed_world_reachability_bitset_parity_v0(
    entrypoints: Vec<ModuleInstanceKeyV0>,
    linked_modules: Vec<ClosedWorldLinkedModuleV0>,
) -> Result<ClosedWorldReachabilityBitsetParityReportV0, ClosedWorldBundleBuildErrorV0> {
    if entrypoints.is_empty() {
        return Err(ClosedWorldBundleBuildErrorV0::EmptyEntrypoints);
    }

    let mut by_instance = BTreeMap::new();
    for module in linked_modules {
        by_instance.insert(module.instance.clone(), module);
    }

    let btreeset_reachability = compute_reachability(entrypoints.as_slice(), &by_instance)?;
    let bitset_reachability = compute_reachability_bitset(entrypoints.as_slice(), &by_instance)?;
    let btreeset_closure_hash =
        stable_closure_hash(entrypoints.as_slice(), &by_instance, &btreeset_reachability);
    let bitset_closure_hash =
        stable_closure_hash(entrypoints.as_slice(), &by_instance, &bitset_reachability);
    let symbol_name_count = bitset_reachability
        .class_names
        .len()
        .saturating_add(bitset_reachability.keyframe_names.len())
        .saturating_add(bitset_reachability.value_names.len())
        .saturating_add(bitset_reachability.custom_property_names.len());

    Ok(ClosedWorldReachabilityBitsetParityReportV0 {
        schema_version: "0",
        product: "omena-parser.closed-world-reachability-bitset-parity",
        module_instance_count: bitset_reachability.module_instances.len(),
        symbol_name_count,
        reachability_equal: btreeset_reachability == bitset_reachability,
        closure_hash_equal: btreeset_closure_hash == bitset_closure_hash,
        btreeset_closure_hash,
        bitset_closure_hash,
    })
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

fn compute_reachability(
    entrypoints: &[ModuleInstanceKeyV0],
    by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldLinkedModuleV0>,
) -> Result<ReachabilityIndexV0, ClosedWorldBundleBuildErrorV0> {
    let mut queue = VecDeque::new();
    let mut seen = BTreeSet::new();
    for entrypoint in entrypoints {
        if !by_instance.contains_key(entrypoint) {
            return Err(ClosedWorldBundleBuildErrorV0::MissingEntrypoint {
                module: entrypoint.clone(),
            });
        }
        queue.push_back(entrypoint.clone());
    }

    while let Some(instance) = queue.pop_front() {
        if !seen.insert(instance.clone()) {
            continue;
        }
        let Some(module) = by_instance.get(&instance) else {
            return Err(ClosedWorldBundleBuildErrorV0::MissingEntrypoint { module: instance });
        };
        for dependency in &module.dependencies {
            if !by_instance.contains_key(dependency) {
                return Err(ClosedWorldBundleBuildErrorV0::MissingDependency {
                    module: instance.clone(),
                    dependency: dependency.clone(),
                });
            }
            queue.push_back(dependency.clone());
        }
    }

    reachability_index_from_seen(&seen, by_instance)
}

fn compute_reachability_bitset(
    entrypoints: &[ModuleInstanceKeyV0],
    by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldLinkedModuleV0>,
) -> Result<ReachabilityIndexV0, ClosedWorldBundleBuildErrorV0> {
    for entrypoint in entrypoints {
        if !by_instance.contains_key(entrypoint) {
            return Err(ClosedWorldBundleBuildErrorV0::MissingEntrypoint {
                module: entrypoint.clone(),
            });
        }
    }
    let dense_index = DenseModuleInstanceIndexV0::from_modules(entrypoints, by_instance);
    let mut seen = DenseModuleInstanceBitsetV0::new(dense_index.len());
    let mut queue = entrypoints
        .iter()
        .filter_map(|entrypoint| dense_index.index_of(entrypoint))
        .collect::<VecDeque<_>>();

    while let Some(instance_index) = queue.pop_front() {
        if !seen.insert(instance_index) {
            continue;
        }
        let instance = dense_index.instance(instance_index);
        let Some(module) = by_instance.get(instance) else {
            return Err(ClosedWorldBundleBuildErrorV0::MissingEntrypoint {
                module: instance.clone(),
            });
        };
        for dependency in &module.dependencies {
            if !by_instance.contains_key(dependency) {
                return Err(ClosedWorldBundleBuildErrorV0::MissingDependency {
                    module: instance.clone(),
                    dependency: dependency.clone(),
                });
            }
            if let Some(dependency_index) = dense_index.index_of(dependency) {
                queue.push_back(dependency_index);
            }
        }
    }

    let seen_instances = dense_index.instances_for_bitset(&seen);
    reachability_index_from_seen(&seen_instances, by_instance)
}

fn reachability_index_from_seen(
    seen: &BTreeSet<ModuleInstanceKeyV0>,
    by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldLinkedModuleV0>,
) -> Result<ReachabilityIndexV0, ClosedWorldBundleBuildErrorV0> {
    let mut class_names = BTreeSet::new();
    let mut keyframe_names = BTreeSet::new();
    let mut value_names = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();

    for instance in seen {
        if let Some(module) = by_instance.get(instance) {
            class_names.extend(module.class_names.iter().cloned());
            keyframe_names.extend(module.keyframe_names.iter().cloned());
            value_names.extend(module.value_names.iter().cloned());
            custom_property_names.extend(module.custom_property_names.iter().cloned());
        }
    }

    Ok(ReachabilityIndexV0 {
        module_instances: seen.iter().cloned().collect(),
        class_names: class_names.into_iter().collect(),
        keyframe_names: keyframe_names.into_iter().collect(),
        value_names: value_names.into_iter().collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
    })
}

#[derive(Debug, Clone)]
struct DenseModuleInstanceIndexV0 {
    instances: Vec<ModuleInstanceKeyV0>,
    positions: BTreeMap<ModuleInstanceKeyV0, usize>,
}

impl DenseModuleInstanceIndexV0 {
    fn from_modules(
        entrypoints: &[ModuleInstanceKeyV0],
        by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldLinkedModuleV0>,
    ) -> Self {
        let mut instances = BTreeSet::new();
        instances.extend(entrypoints.iter().cloned());
        for (instance, module) in by_instance {
            instances.insert(instance.clone());
            instances.extend(module.dependencies.iter().cloned());
        }
        let instances = instances.into_iter().collect::<Vec<_>>();
        let positions = instances
            .iter()
            .enumerate()
            .map(|(index, instance)| (instance.clone(), index))
            .collect::<BTreeMap<_, _>>();
        Self {
            instances,
            positions,
        }
    }

    fn len(&self) -> usize {
        self.instances.len()
    }

    fn index_of(&self, instance: &ModuleInstanceKeyV0) -> Option<usize> {
        self.positions.get(instance).copied()
    }

    fn instance(&self, index: usize) -> &ModuleInstanceKeyV0 {
        &self.instances[index]
    }

    fn instances_for_bitset(
        &self,
        bitset: &DenseModuleInstanceBitsetV0,
    ) -> BTreeSet<ModuleInstanceKeyV0> {
        self.instances
            .iter()
            .enumerate()
            .filter(|(index, _)| bitset.contains(*index))
            .map(|(_, instance)| instance.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
struct DenseModuleInstanceBitsetV0 {
    words: Vec<u64>,
}

impl DenseModuleInstanceBitsetV0 {
    fn new(len: usize) -> Self {
        Self {
            words: vec![0; len.div_ceil(64)],
        }
    }

    fn insert(&mut self, index: usize) -> bool {
        let word_index = index / 64;
        let mask = 1u64 << (index % 64);
        let word = &mut self.words[word_index];
        let was_empty = *word & mask == 0;
        *word |= mask;
        was_empty
    }

    fn contains(&self, index: usize) -> bool {
        self.words
            .get(index / 64)
            .is_some_and(|word| word & (1u64 << (index % 64)) != 0)
    }
}

fn stable_closure_hash(
    entrypoints: &[ModuleInstanceKeyV0],
    by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldLinkedModuleV0>,
    reachability: &ReachabilityIndexV0,
) -> String {
    let mut hash = StableFnv64::new();
    hash.piece("omena-parser.closed-world-bundle");
    for entrypoint in entrypoints {
        hash.instance(entrypoint);
    }
    for instance in &reachability.module_instances {
        hash.instance(instance);
        if let Some(module) = by_instance.get(instance) {
            for dependency in &module.dependencies {
                hash.instance(dependency);
            }
        }
    }
    for name in &reachability.class_names {
        hash.piece("class");
        hash.piece(name);
    }
    for name in &reachability.keyframe_names {
        hash.piece("keyframe");
        hash.piece(name);
    }
    for name in &reachability.value_names {
        hash.piece("value");
        hash.piece(name);
    }
    for name in &reachability.custom_property_names {
        hash.piece("custom-property");
        hash.piece(name);
    }
    hash.finish_hex()
}

struct StableFnv64(u64);

impl StableFnv64 {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn piece(&mut self, value: &str) {
        for byte in value.as_bytes().iter().copied().chain([0]) {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn instance(&mut self, instance: &ModuleInstanceKeyV0) {
        self.piece(instance.module().as_str());
        self.piece(instance.configuration().as_str());
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldLinkedModuleV0,
        ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0,
        summarize_closed_world_reachability_bitset_parity_v0,
    };

    #[test]
    fn module_instance_key_distinguishes_configurations() {
        let module = ModuleIdV0::new("src/theme.scss");
        let light =
            ModuleInstanceKeyV0::new(module.clone(), ConfigurationHashV0::new("with:brand=light"));
        let dark = ModuleInstanceKeyV0::new(module, ConfigurationHashV0::new("with:brand=dark"));

        assert_ne!(light, dark);
        assert_eq!(light.module().as_str(), "src/theme.scss");
        assert_eq!(dark.configuration().as_str(), "with:brand=dark");
    }

    #[test]
    fn closed_world_bundle_constructor_computes_reachability() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/tokens.css"));
        let modules = vec![
            ClosedWorldLinkedModuleV0::new(app.clone())
                .with_dependency(tokens.clone())
                .with_class_name("button"),
            ClosedWorldLinkedModuleV0::new(tokens.clone())
                .with_custom_property_name("--brand")
                .with_value_name("spacing"),
        ];

        let bundle = ClosedWorldBundleV0::try_from_linked_modules(vec![app], modules)
            .map_err(|err| format!("{err:?}"))?;

        assert_eq!(bundle.linked_modules().len(), 2);
        assert!(bundle.reachability().module_instances().contains(&tokens));
        assert!(
            bundle
                .reachability()
                .class_names()
                .contains(&"button".to_string())
        );
        assert!(
            bundle
                .reachability()
                .custom_property_names()
                .contains(&"--brand".to_string())
        );
        Ok(())
    }

    #[test]
    fn closed_world_bundle_closure_hash_is_content_addressed() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let module = ClosedWorldLinkedModuleV0::new(app.clone()).with_class_name("button");
        let first = ClosedWorldBundleV0::try_from_linked_modules(vec![app.clone()], vec![module])
            .map_err(|err| format!("{err:?}"))?;
        let same = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app.clone()).with_class_name("button")],
        )
        .map_err(|err| format!("{err:?}"))?;
        let changed = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app).with_class_name("card")],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert_eq!(first.closure_hash(), same.closure_hash());
        assert_ne!(first.closure_hash(), changed.closure_hash());
        Ok(())
    }

    #[test]
    fn closed_world_bitset_reachability_preserves_closure_hash() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/tokens.css"));
        let theme = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/theme.css"));
        let report = summarize_closed_world_reachability_bitset_parity_v0(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app)
                    .with_dependency(tokens.clone())
                    .with_class_name("button"),
                ClosedWorldLinkedModuleV0::new(tokens.clone())
                    .with_dependency(theme.clone())
                    .with_value_name("spacing"),
                ClosedWorldLinkedModuleV0::new(theme)
                    .with_keyframe_name("fade")
                    .with_custom_property_name("--brand"),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert!(report.reachability_equal, "{report:#?}");
        assert!(report.closure_hash_equal, "{report:#?}");
        assert!(report.module_instance_count >= 3, "{report:#?}");
        assert!(report.symbol_name_count >= 3, "{report:#?}");
        Ok(())
    }

    #[test]
    fn closed_world_bundle_rejects_missing_dependency() {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let missing = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/missing.css"));
        let err = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app.clone()).with_dependency(missing.clone())],
        );

        assert_eq!(
            err,
            Err(ClosedWorldBundleBuildErrorV0::MissingDependency {
                module: app,
                dependency: missing,
            })
        );
    }
}
