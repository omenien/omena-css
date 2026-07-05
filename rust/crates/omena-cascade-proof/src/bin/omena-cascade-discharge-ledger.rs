#[cfg(not(feature = "smt-z3"))]
fn main() {
    eprintln!("run with --features smt-z3 to build the discharge ledger");
    std::process::exit(1);
}

#[cfg(feature = "smt-z3")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    enabled::run()
}

#[cfg(feature = "smt-z3")]
mod enabled {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
    };

    use omena_cascade::{
        BoxLonghandInputV0, LayerFlattenInputV0, LonghandMergeInputV0, ScopeFlattenInputV0,
        StaticSupportsAssumptionV0,
    };
    use omena_cascade_proof::{
        CanonicalSmtInputV0, CascadeSMTProofV0, LayerInversionDeclarationV0, SmtBackendSatResultV0,
        StubSmtBackendV0, canonical_layer_flatten_inversion_input_v0,
        layer_inversion_declaration_v0, smt_evaluate_static_supports_condition_v0,
        smt_prove_box_shorthand_combination_v0, smt_prove_layer_flatten_candidate_v0,
        smt_prove_longhand_merge_v0, smt_prove_scope_flatten_candidate_v0,
    };
    use omena_smt::SmtBackendV0 as Z3Backend;
    use serde::Serialize;

    const SCHEMA_VERSION: &str = "1";
    const PRODUCT: &str = "omena-cascade-proof.discharge-ledger";
    const EXACT: DischargeBoundednessV1 = DischargeBoundednessV1 {
        kind: "exact",
        k: None,
    };
    const LAYER_INVERSION_BOUND: usize = 3;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DischargeLedgerV1 {
        schema_version: &'static str,
        product: &'static str,
        pins: DischargeLedgerPinsV1,
        coverage: Vec<DischargeLedgerCoverageV1>,
        entries: Vec<DischargeLedgerEntryV1>,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DischargeLedgerPinsV1 {
        theory_signature_hash: String,
        spec_digest: String,
        encoder_content_hash: String,
        solver_version: String,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DischargeLedgerEntryV1 {
        obligation_family: &'static str,
        cell_family: &'static str,
        obligation_id: String,
        l1_primitive: &'static str,
        cell_key: String,
        canonical_term_count: usize,
        canonical_terms: Vec<String>,
        verdict: &'static str,
        boundedness: DischargeBoundednessV1,
        reference_kind: &'static str,
        reference_sat_result: &'static str,
        solver_kind: &'static str,
        solver_sat_result: &'static str,
        reference_matches_solver: bool,
        pins: DischargeLedgerPinsV1,
    }

    #[derive(Debug, Clone, Copy, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DischargeBoundednessV1 {
        kind: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        k: Option<usize>,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DischargeLedgerCoverageV1 {
        obligation_family: &'static str,
        cell_family: &'static str,
        cell_count: usize,
        exhaustive: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        bound: Option<String>,
    }

    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let pins = ledger_pins()?;
        let mut entries = BTreeMap::<String, DischargeLedgerEntryV1>::new();

        emit_box_shorthand_cells(&pins, &mut entries);
        emit_longhand_merge_cells(&pins, &mut entries);
        emit_scope_flatten_cells(&pins, &mut entries);
        emit_layer_flatten_cells(&pins, &mut entries);
        emit_static_supports_cells(&pins, &mut entries);
        emit_layer_inversion_cells(&pins, &mut entries);

        let entries = entries.into_values().collect::<Vec<_>>();
        let coverage = coverage_for_entries(&entries);
        let ledger = DischargeLedgerV1 {
            schema_version: SCHEMA_VERSION,
            product: PRODUCT,
            pins,
            coverage,
            entries,
        };
        println!("{}", serde_json::to_string_pretty(&ledger)?);
        Ok(())
    }

    fn emit_box_shorthand_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        for supported in [false, true] {
            for canonical_order in [false, true] {
                if canonical_order && !supported {
                    continue;
                }
                for no_important in [false, true] {
                    for no_empty_value in [false, true] {
                        for adjacent_source_order in [false, true] {
                            let shorthand = if supported { "margin" } else { "unsupported" };
                            let longhands = longhands_for(
                                shorthand,
                                canonical_order,
                                no_important,
                                no_empty_value,
                                adjacent_source_order,
                            );
                            let proof = smt_prove_box_shorthand_combination_v0(
                                shorthand,
                                &longhands,
                                &StubSmtBackendV0::default(),
                            );
                            insert_propositional_entry(
                                pins,
                                entries,
                                "longhandShorthandCascadeOutcome",
                                "boxShorthandCombination",
                                proof,
                            );
                        }
                    }
                }
            }
        }
    }

    fn emit_longhand_merge_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        for expected_present in [false, true] {
            for canonical_order in [false, true] {
                if canonical_order && !expected_present {
                    continue;
                }
                for no_important in [false, true] {
                    for no_empty_value in [false, true] {
                        for adjacent_source_order in [false, true] {
                            let expected = if expected_present {
                                longhand_names("margin")
                                    .into_iter()
                                    .map(str::to_string)
                                    .collect::<Vec<_>>()
                            } else {
                                Vec::new()
                            };
                            let longhands = longhands_for(
                                "margin",
                                canonical_order,
                                no_important,
                                no_empty_value,
                                adjacent_source_order,
                            );
                            let proof = smt_prove_longhand_merge_v0(
                                "margin",
                                &expected,
                                &longhands,
                                &StubSmtBackendV0::default(),
                            );
                            insert_propositional_entry(
                                pins,
                                entries,
                                "longhandShorthandCascadeOutcome",
                                "longhandMerge",
                                proof,
                            );
                        }
                    }
                }
            }
        }
    }

    fn emit_scope_flatten_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        for no_limit_selector in [false, true] {
            for root_scope in [false, true] {
                for no_peer_scope in [false, true] {
                    for no_competing_unscoped_rule in [false, true] {
                        for not_inside_layer in [false, true] {
                            let input = ScopeFlattenInputV0 {
                                root_selector: if root_scope { ":root" } else { ".scope" }
                                    .to_string(),
                                limit_selector: (!no_limit_selector).then(|| ".limit".to_string()),
                                scoped_rule_count: 1,
                                peer_scope_count: if no_peer_scope { 0 } else { 1 },
                                competing_unscoped_rule_count: if no_competing_unscoped_rule {
                                    0
                                } else {
                                    1
                                },
                                inside_layer: !not_inside_layer,
                            };
                            let proof = smt_prove_scope_flatten_candidate_v0(
                                input,
                                &StubSmtBackendV0::default(),
                            );
                            insert_propositional_entry(
                                pins,
                                entries,
                                "scopedMatching",
                                "scopeFlattenCandidate",
                                proof,
                            );
                        }
                    }
                }
            }
        }
    }

    fn emit_layer_flatten_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        for closed_bundle in [false, true] {
            for no_peer_layer in [false, true] {
                for no_unlayered_rule in [false, true] {
                    for no_important_declaration in [false, true] {
                        let input = LayerFlattenInputV0 {
                            layer_name: Some("components".to_string()),
                            layer_rule_count: 1,
                            peer_layer_count: if no_peer_layer { 0 } else { 1 },
                            unlayered_rule_count: if no_unlayered_rule { 0 } else { 1 },
                            important_declaration_count: if no_important_declaration {
                                0
                            } else {
                                1
                            },
                            closed_bundle,
                        };
                        let proof = smt_prove_layer_flatten_candidate_v0(
                            input,
                            &StubSmtBackendV0::default(),
                        );
                        insert_propositional_entry(
                            pins,
                            entries,
                            "layerOrderComparison",
                            "layerFlattenCandidate",
                            proof,
                        );
                    }
                }
            }
        }
    }

    fn emit_static_supports_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        for condition in [
            "(display: grid)",
            "not (display: grid)",
            "font-tech(unknown-thing)",
        ] {
            let proof = smt_evaluate_static_supports_condition_v0(
                condition,
                StaticSupportsAssumptionV0::ModernBrowser,
                &StubSmtBackendV0::default(),
            );
            insert_propositional_entry(
                pins,
                entries,
                "targetFeaturePredicate",
                "staticSupportsCondition",
                proof,
            );
        }
    }

    fn emit_layer_inversion_cells(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
    ) {
        let values = [-1, 0, 1];
        for count in 0..=LAYER_INVERSION_BOUND {
            let vector_count = values.len().pow((count * 2) as u32);
            for vector_index in 0..vector_count {
                let declarations = layer_inversion_declarations(count, vector_index, values);
                insert_layer_inversion_entry(pins, entries, declarations);
            }
        }
    }

    fn insert_propositional_entry(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
        obligation_family: &'static str,
        cell_family: &'static str,
        proof: CascadeSMTProofV0,
    ) {
        let canonical_input = proof.canonical_input;
        let z3_input = to_z3_input(&canonical_input);
        let z3_check = omena_smt::Z3SmtBackendV0::default().check_canonical_input_v0(&z3_input);
        let cell_key = cell_key(&canonical_input);
        let reference_sat_result = sat_result_label(proof.solver_check.sat_result);
        let solver_sat_result = z3_sat_result_label(z3_check.sat_result);
        let reference_matches_solver = reference_sat_result == solver_sat_result;

        entries
            .entry(cell_key.clone())
            .or_insert(DischargeLedgerEntryV1 {
                obligation_family,
                cell_family,
                obligation_id: canonical_input.obligation_id.clone(),
                l1_primitive: canonical_input.l1_primitive,
                cell_key,
                canonical_term_count: canonical_input.canonical_terms.len(),
                canonical_terms: canonical_input.canonical_terms.clone(),
                verdict: z3_verdict_label(z3_check.sat_result),
                boundedness: EXACT,
                reference_kind: "productStubBackend",
                reference_sat_result,
                solver_kind: "z3",
                solver_sat_result,
                reference_matches_solver,
                pins: pins.clone(),
            });
    }

    fn insert_layer_inversion_entry(
        pins: &DischargeLedgerPinsV1,
        entries: &mut BTreeMap<String, DischargeLedgerEntryV1>,
        declarations: Vec<LayerInversionDeclarationV0>,
    ) {
        let canonical_input = canonical_layer_flatten_inversion_input_v0(&declarations);
        let smt_declarations = declarations
            .iter()
            .map(|declaration| {
                omena_smt::layer_inversion_declaration_v0(
                    declaration.declaration_id.clone(),
                    declaration.layer_rank,
                    declaration.source_order,
                )
            })
            .collect::<Vec<_>>();
        let z3_verdict = omena_smt::smt_check_layer_flatten_inversion_v0(
            &smt_declarations,
            &omena_smt::Z3SmtBackendV0::default(),
        );
        let reference_sat_result = if has_layer_ordering_inversion(&declarations) {
            "sat"
        } else {
            "unsat"
        };
        let solver_sat_result = z3_sat_result_label(z3_verdict.sat_result);
        let cell_key = cell_key(&canonical_input);
        entries
            .entry(cell_key.clone())
            .or_insert(DischargeLedgerEntryV1 {
                obligation_family: "layerOrderComparison",
                cell_family: "layerFlattenCascadeInversion",
                obligation_id: canonical_input.obligation_id.clone(),
                l1_primitive: canonical_input.l1_primitive,
                cell_key,
                canonical_term_count: canonical_input.canonical_terms.len(),
                canonical_terms: canonical_input.canonical_terms.clone(),
                verdict: z3_verdict_label(z3_verdict.sat_result),
                boundedness: DischargeBoundednessV1 {
                    kind: "boundedK",
                    k: Some(LAYER_INVERSION_BOUND),
                },
                reference_kind: "boundedLayerOrderingPredicate",
                reference_sat_result,
                solver_kind: "z3",
                solver_sat_result,
                reference_matches_solver: reference_sat_result == solver_sat_result,
                pins: pins.clone(),
            });
    }

    fn longhands_for(
        shorthand: &str,
        canonical_order: bool,
        no_important: bool,
        no_empty_value: bool,
        adjacent_source_order: bool,
    ) -> Vec<BoxLonghandInputV0> {
        let mut names = longhand_names(if shorthand == "unsupported" {
            "margin"
        } else {
            shorthand
        });
        if !canonical_order {
            names.swap(1, 2);
        }
        names
            .into_iter()
            .enumerate()
            .map(|(index, property)| LonghandMergeInputV0 {
                property: property.to_string(),
                value: if no_empty_value {
                    "1px".to_string()
                } else {
                    String::new()
                },
                important: !no_important && index == 0,
                source_order: if adjacent_source_order {
                    index as u32 + 1
                } else {
                    index as u32 * 2 + 1
                },
            })
            .collect()
    }

    fn longhand_names(shorthand: &str) -> [&'static str; 4] {
        match shorthand {
            "padding" => [
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
            ],
            _ => ["margin-top", "margin-right", "margin-bottom", "margin-left"],
        }
    }

    fn layer_inversion_declarations(
        count: usize,
        mut vector_index: usize,
        values: [i64; 3],
    ) -> Vec<LayerInversionDeclarationV0> {
        let mut declarations = Vec::with_capacity(count);
        for index in 0..count {
            let rank = values[vector_index % values.len()];
            vector_index /= values.len();
            let source_order = values[vector_index % values.len()];
            vector_index /= values.len();
            declarations.push(layer_inversion_declaration_v0(
                format!("decl-{index}"),
                rank,
                source_order,
            ));
        }
        declarations
    }

    fn has_layer_ordering_inversion(declarations: &[LayerInversionDeclarationV0]) -> bool {
        declarations.iter().enumerate().any(|(left_index, left)| {
            declarations.iter().enumerate().any(|(right_index, right)| {
                left_index != right_index
                    && left.layer_rank > right.layer_rank
                    && right.source_order > left.source_order
            })
        })
    }

    fn coverage_for_entries(entries: &[DischargeLedgerEntryV1]) -> Vec<DischargeLedgerCoverageV1> {
        let mut counts = BTreeMap::<(&'static str, &'static str), usize>::new();
        for entry in entries {
            *counts
                .entry((entry.obligation_family, entry.cell_family))
                .or_default() += 1;
        }
        counts
            .into_iter()
            .map(
                |((obligation_family, cell_family), cell_count)| DischargeLedgerCoverageV1 {
                    obligation_family,
                    cell_family,
                    cell_count,
                    exhaustive: true,
                    bound: (cell_family == "layerFlattenCascadeInversion").then(|| {
                        format!("declarationCount<={LAYER_INVERSION_BOUND},rank/source in [-1,0,1]")
                    }),
                },
            )
            .collect()
    }

    fn to_z3_input(input: &CanonicalSmtInputV0) -> omena_smt::CanonicalSmtInputV0 {
        omena_smt::canonical_smt_input_with_script_v0(
            input.obligation_id.clone(),
            input.l1_primitive,
            input.canonical_terms.clone(),
            input.smtlib2_script.clone(),
        )
    }

    fn ledger_pins() -> Result<DischargeLedgerPinsV1, Box<dyn std::error::Error>> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let smt_dir = manifest_dir
            .parent()
            .ok_or("crate directory should have a parent")?
            .join("omena-smt");
        let rust_dir = manifest_dir
            .parent()
            .and_then(Path::parent)
            .ok_or("crate directory should resolve to rust workspace root")?;
        let theory_signature_hash = hash_bytes(&serde_json::to_vec(
            &omena_smt::cascade_theory_signature_v0(),
        )?);
        let spec_digest = hex_bytes(omena_smt::cascade_spec_digest_v0());
        let encoder_content_hash = hash_sources(
            rust_dir,
            &[
                manifest_dir.join("src/lib.rs"),
                smt_dir.join("src/encoder.rs"),
                smt_dir.join("src/obligations.rs"),
                smt_dir.join("src/layer_inversion.rs"),
                smt_dir.join("src/backend/z3.rs"),
                smt_dir.join("src/proof.rs"),
            ],
        )?;
        let solver_version = solver_version_pin(&rust_dir.join("Cargo.lock"))?;
        Ok(DischargeLedgerPinsV1 {
            theory_signature_hash,
            spec_digest,
            encoder_content_hash,
            solver_version,
        })
    }

    fn solver_version_pin(cargo_lock: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let source = fs::read_to_string(cargo_lock)?;
        let mut in_z3_package = false;
        let version = source
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                if trimmed == "[[package]]" {
                    in_z3_package = false;
                    return None;
                }
                if trimmed == "name = \"z3\"" {
                    in_z3_package = true;
                    return None;
                }
                in_z3_package
                    .then(|| trimmed.strip_prefix("version = \""))
                    .flatten()
                    .and_then(|tail| tail.split('"').next())
            })
            .ok_or("resolved z3 version not found")?;
        Ok(format!("z3-crate-{version}-gh-release"))
    }

    fn hash_sources(root: &Path, paths: &[PathBuf]) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = blake3::Hasher::new();
        for path in paths {
            hasher.update(stable_source_label(root, path)?.as_bytes());
            hasher.update(b"\0");
            hasher.update(&fs::read(path)?);
            hasher.update(b"\0");
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn stable_source_label(root: &Path, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let relative = path.strip_prefix(root)?;
        Ok(relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"))
    }

    fn cell_key(input: &CanonicalSmtInputV0) -> String {
        hash_bytes(input.smtlib2_script.as_bytes())
    }

    fn hash_bytes(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    fn hex_bytes(bytes: [u8; 32]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn sat_result_label(result: SmtBackendSatResultV0) -> &'static str {
        match result {
            SmtBackendSatResultV0::Sat => "sat",
            SmtBackendSatResultV0::Unsat => "unsat",
            SmtBackendSatResultV0::Unknown => "unknown",
        }
    }

    fn z3_sat_result_label(result: omena_smt::SmtBackendSatResultV0) -> &'static str {
        match result {
            omena_smt::SmtBackendSatResultV0::Sat => "sat",
            omena_smt::SmtBackendSatResultV0::Unsat => "unsat",
            omena_smt::SmtBackendSatResultV0::Unknown => "unknown",
        }
    }

    fn z3_verdict_label(result: omena_smt::SmtBackendSatResultV0) -> &'static str {
        match result {
            omena_smt::SmtBackendSatResultV0::Sat => "accepted",
            omena_smt::SmtBackendSatResultV0::Unsat => "rejected",
            omena_smt::SmtBackendSatResultV0::Unknown => "unknown",
        }
    }
}
