use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    OmegaCascadeTruthValueV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeFrameV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub frame_id: String,
    pub worlds: Vec<String>,
    pub edges: Vec<KripkeEdgeV0>,
    pub valuations: Vec<KripkeValuationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeEdgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_world: String,
    pub to_world: String,
    pub relation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeValuationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub world: String,
    pub atom: String,
    pub truth_value: OmegaCascadeTruthValueV0,
}

pub fn empty_s4_kripke_frame_v0(frame_id: impl Into<String>) -> KripkeFrameV0 {
    KripkeFrameV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.kripke-frame",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        frame_id: frame_id.into(),
        worlds: Vec::new(),
        edges: Vec::new(),
        valuations: Vec::new(),
    }
}

/// Build a real S4 Kripke frame whose worlds are cascade nesting contexts. Each
/// input world is `(context_stack, resolved_value)`, where `context_stack` is the
/// ordered nesting prefix (outermost first — the enclosing `@media`/`@supports`
/// conditions) and `resolved_value` is the property value the real cascade
/// ranking resolves in that context. Worlds are connected by the
/// reflexive-transitive prefix-of accessibility relation: `a -> b` whenever `a`'s
/// context stack is a prefix of `b`'s. That yields an S4 frame (reflexive +
/// transitive) over only the observed contexts, with no powerset blow-up. Each
/// valuation's atom encodes the resolved value as `"{property}={value}"`, so
/// box-stability compares value strings; `truth_value` is only a lattice presence
/// marker, because `OmegaCascadeTruthValueV0::from_outcome` collapses every
/// `Definite` outcome to `Closed` and could never witness a value divergence.
pub fn build_cascade_prefix_kripke_frame_v0(
    frame_id: impl Into<String>,
    property: &str,
    worlds: &[(Vec<String>, String)],
) -> KripkeFrameV0 {
    let world_ids: Vec<String> = worlds
        .iter()
        .map(|(stack, _)| cascade_context_world_id_v0(stack))
        .collect();
    let mut edges = Vec::new();
    for (from_index, (from_stack, _)) in worlds.iter().enumerate() {
        for (to_index, (to_stack, _)) in worlds.iter().enumerate() {
            if context_stack_is_prefix_v0(from_stack, to_stack) {
                edges.push(KripkeEdgeV0 {
                    schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
                    product: "omena-categorical.kripke-edge",
                    layer_marker: CATEGORICAL_LAYER_MARKER_V0,
                    feature_gate: CATEGORICAL_FEATURE_GATE_V0,
                    from_world: world_ids[from_index].clone(),
                    to_world: world_ids[to_index].clone(),
                    relation: "cascade-context-prefix",
                });
            }
        }
    }
    let valuations = worlds
        .iter()
        .enumerate()
        .map(|(index, (_, value))| KripkeValuationV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.kripke-valuation",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            world: world_ids[index].clone(),
            atom: format!("{property}={value}"),
            truth_value: OmegaCascadeTruthValueV0::Closed,
        })
        .collect();
    KripkeFrameV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.kripke-frame",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        frame_id: frame_id.into(),
        worlds: world_ids,
        edges,
        valuations,
    }
}

fn cascade_context_world_id_v0(context_stack: &[String]) -> String {
    if context_stack.is_empty() {
        "base".to_string()
    } else {
        format!("base|{}", context_stack.join("|"))
    }
}

fn context_stack_is_prefix_v0(prefix: &[String], full: &[String]) -> bool {
    prefix.len() <= full.len() && full[..prefix.len()] == *prefix
}

/// Evaluate `□(property)` (necessity) at the frame's root context — the world
/// whose accessibility set covers every world (the prefix-minimal context). The
/// resolved value is *necessary* iff every world reachable from the root shares
/// the root world's resolved-value atom. A nested context that overrides the
/// value breaks necessity, so the verdict is computed from the analysed frame,
/// never echoed. An empty frame is rejected.
pub fn cascade_box_stable_v0(frame: &KripkeFrameV0) -> bool {
    if frame.worlds.is_empty() {
        return false;
    }
    let accessible = |world: &str| -> Vec<&str> {
        frame
            .edges
            .iter()
            .filter(|edge| edge.from_world == world)
            .map(|edge| edge.to_world.as_str())
            .collect()
    };
    let Some(root) = frame
        .worlds
        .iter()
        .max_by_key(|world| accessible(world).len())
    else {
        return false;
    };
    let reachable = accessible(root);
    if reachable.is_empty() {
        return false;
    }
    let atom_of = |world: &str| {
        frame
            .valuations
            .iter()
            .find(|valuation| valuation.world == world)
            .map(|valuation| valuation.atom.as_str())
    };
    let mut atoms = reachable.iter().filter_map(|world| atom_of(world));
    let Some(first) = atoms.next() else {
        return false;
    };
    atoms.all(|atom| atom == first)
}
