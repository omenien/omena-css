//! Closed-world linking authority: builds sealed bundles from linked-module facts.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::contract::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldInterfaceHashAvailabilityV0,
    ClosedWorldInterfaceHashEntryV0, ClosedWorldInterfaceHashSetV0, ClosedWorldLinkedModuleV0,
    ClosedWorldModuleMetadataV0, ClosedWorldReachabilityBitsetParityReportV0,
    ClosedWorldSourcePrecisionSummaryV0, ModuleInstanceKeyV0, ModuleQualifiedSymbolSetV0,
    ReachabilityIndexV0,
};

impl ClosedWorldBundleV0 {
    pub fn try_from_linked_modules(
        entrypoints: Vec<ModuleInstanceKeyV0>,
        linked_modules: Vec<ClosedWorldLinkedModuleV0>,
    ) -> Result<Self, ClosedWorldBundleBuildErrorV0> {
        Self::try_from_linked_modules_with_metadata(entrypoints, linked_modules, Vec::new())
    }

    pub fn try_from_linked_modules_with_metadata(
        entrypoints: Vec<ModuleInstanceKeyV0>,
        linked_modules: Vec<ClosedWorldLinkedModuleV0>,
        module_metadata: Vec<ClosedWorldModuleMetadataV0>,
    ) -> Result<Self, ClosedWorldBundleBuildErrorV0> {
        if entrypoints.is_empty() {
            return Err(ClosedWorldBundleBuildErrorV0::EmptyEntrypoints);
        }

        let mut by_instance = BTreeMap::new();
        for module in linked_modules {
            by_instance.insert(module.instance.clone(), module);
        }
        let metadata_by_instance = module_metadata
            .into_iter()
            .map(|metadata| (metadata.module_instance().clone(), metadata))
            .collect::<BTreeMap<_, _>>();

        let reachability = compute_reachability(entrypoints.as_slice(), &by_instance)?;
        let linked_modules = reachability.module_instances().to_vec();
        let interface_hashes =
            interface_hashes_for_reachable_modules(&linked_modules, &metadata_by_instance);
        let source_precision =
            source_precision_for_reachable_modules(&linked_modules, &metadata_by_instance);
        let closure_hash = stable_closure_hash(entrypoints.as_slice(), &by_instance, &reachability);

        Ok(Self::seal(
            entrypoints,
            linked_modules,
            reachability,
            closure_hash,
            interface_hashes,
            source_precision,
        ))
    }
}

fn interface_hashes_for_reachable_modules(
    reachable: &[ModuleInstanceKeyV0],
    metadata_by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleMetadataV0>,
) -> ClosedWorldInterfaceHashSetV0 {
    ClosedWorldInterfaceHashSetV0::new(
        reachable
            .iter()
            .map(|instance| ClosedWorldInterfaceHashEntryV0 {
                module_instance: instance.clone(),
                availability: metadata_by_instance
                    .get(instance)
                    .and_then(|metadata| metadata.interface_hash())
                    .map_or(
                        ClosedWorldInterfaceHashAvailabilityV0::Absent,
                        |interface_hash| ClosedWorldInterfaceHashAvailabilityV0::Known {
                            interface_hash: interface_hash.to_string(),
                        },
                    ),
            })
            .collect(),
    )
}

fn source_precision_for_reachable_modules(
    reachable: &[ModuleInstanceKeyV0],
    metadata_by_instance: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleMetadataV0>,
) -> Option<ClosedWorldSourcePrecisionSummaryV0> {
    let mut aggregate = ClosedWorldSourcePrecisionSummaryV0::default();
    let mut observed = false;
    for instance in reachable {
        if let Some(source_precision) = metadata_by_instance
            .get(instance)
            .and_then(ClosedWorldModuleMetadataV0::source_precision)
        {
            aggregate.merge(source_precision);
            observed = true;
        }
    }
    observed.then_some(aggregate)
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
        .class_names()
        .len()
        .saturating_add(bitset_reachability.keyframe_names().len())
        .saturating_add(bitset_reachability.value_names().len())
        .saturating_add(bitset_reachability.custom_property_names().len());

    Ok(ClosedWorldReachabilityBitsetParityReportV0 {
        schema_version: "0",
        product: "omena-parser.closed-world-reachability-bitset-parity",
        module_instance_count: bitset_reachability.module_instances().len(),
        symbol_name_count,
        reachability_equal: btreeset_reachability == bitset_reachability,
        closure_hash_equal: btreeset_closure_hash == bitset_closure_hash,
        btreeset_closure_hash,
        bitset_closure_hash,
    })
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
    let mut module_qualified_symbols = Vec::with_capacity(by_instance.len());

    for (instance, module) in by_instance {
        let reachable = seen.contains(instance);
        if reachable {
            class_names.extend(module.class_names.iter().cloned());
            keyframe_names.extend(module.keyframe_names.iter().cloned());
            value_names.extend(module.value_names.iter().cloned());
            custom_property_names.extend(module.custom_property_names.iter().cloned());
        }
        module_qualified_symbols.push(ModuleQualifiedSymbolSetV0::new(
            instance.clone(),
            reachable,
            reachable
                .then(|| dedupe_symbol_names(&module.class_names))
                .unwrap_or_default(),
            reachable
                .then(|| dedupe_symbol_names(&module.keyframe_names))
                .unwrap_or_default(),
            reachable
                .then(|| dedupe_symbol_names(&module.value_names))
                .unwrap_or_default(),
            reachable
                .then(|| dedupe_symbol_names(&module.custom_property_names))
                .unwrap_or_default(),
        ));
    }

    Ok(ReachabilityIndexV0::from_parts(
        seen.iter().cloned().collect(),
        module_qualified_symbols,
        class_names.into_iter().collect(),
        keyframe_names.into_iter().collect(),
        value_names.into_iter().collect(),
        custom_property_names.into_iter().collect(),
    ))
}

fn dedupe_symbol_names(names: &[String]) -> Vec<String> {
    names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
    for instance in reachability.module_instances() {
        hash.instance(instance);
        if let Some(module) = by_instance.get(instance) {
            for dependency in &module.dependencies {
                hash.instance(dependency);
            }
        }
    }
    for name in reachability.class_names() {
        hash.piece("class");
        hash.piece(name);
    }
    for name in reachability.keyframe_names() {
        hash.piece("keyframe");
        hash.piece(name);
    }
    for name in reachability.value_names() {
        hash.piece("value");
        hash.piece(name);
    }
    for name in reachability.custom_property_names() {
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
