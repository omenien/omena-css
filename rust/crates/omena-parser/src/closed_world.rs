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

    let mut class_names = BTreeSet::new();
    let mut keyframe_names = BTreeSet::new();
    let mut value_names = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();

    for instance in &seen {
        let Some(module) = by_instance.get(instance) else {
            continue;
        };
        class_names.extend(module.class_names.iter().cloned());
        keyframe_names.extend(module.keyframe_names.iter().cloned());
        value_names.extend(module.value_names.iter().cloned());
        custom_property_names.extend(module.custom_property_names.iter().cloned());
    }

    Ok(ReachabilityIndexV0 {
        module_instances: seen.into_iter().collect(),
        class_names: class_names.into_iter().collect(),
        keyframe_names: keyframe_names.into_iter().collect(),
        value_names: value_names.into_iter().collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
    })
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
