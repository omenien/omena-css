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
    pub(crate) fn from_parts(
        module_instances: Vec<ModuleInstanceKeyV0>,
        class_names: Vec<String>,
        keyframe_names: Vec<String>,
        value_names: Vec<String>,
        custom_property_names: Vec<String>,
    ) -> Self {
        Self {
            module_instances,
            class_names,
            keyframe_names,
            value_names,
            custom_property_names,
        }
    }

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
    pub(crate) fn seal(
        entrypoints: Vec<ModuleInstanceKeyV0>,
        linked_modules: Vec<ModuleInstanceKeyV0>,
        reachability: ReachabilityIndexV0,
        closure_hash: String,
    ) -> Self {
        Self {
            entrypoints,
            linked_modules,
            reachability,
            closure_hash,
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
