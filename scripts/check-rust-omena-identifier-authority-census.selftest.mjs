#!/usr/bin/env node

import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rawScanChecker = "scripts/check-rust-omena-syntax-authority-raw-scan-census.ts";
const identifierChecker = "scripts/check-rust-omena-identifier-authority-census.ts";
const generatedMatrixOnly = process.argv.includes("--generated-matrix-only");

const generatedOrigins = [
  "field-access",
  "tuple-field-access",
  "bare-parameter",
  "fqn-parameter",
  "alias-parameter",
  "generic-bound-parameter",
  "self-receiver",
  "closure-inferred",
  "for-loop-binding",
  "while-let-binding",
  "if-let-binding",
  "let-else-binding",
  "let-chain-binding",
  "match-arm-binding",
  "at-binding",
  "or-pattern-binding",
  "struct-destructuring",
  "tuple-destructuring",
  "slice-pattern",
  "two-statement-local",
  "wrapper-function",
  "two-step-wrapper",
  "trait-method-return",
  "accessor-return-inline",
  "macro-rules-body",
  "named-escape",
];
const generatedComparisonGrammars = [
  "binary-eq",
  "binary-ne",
  "method-eq",
  "method-ne",
  "eq-ignore-ascii-case",
  "cmp-is-eq",
  "partial-cmp-is-eq",
  "ufcs-str-eq",
  "ufcs-partial-eq",
  "map-insert",
  "map-get",
  "map-entry",
  "map-contains-key",
  "map-remove",
  "set-insert",
  "set-get",
  "set-contains",
  "set-remove",
  "sort",
  "sort-by",
  "sort-by-key",
  "sort-by-cached-key",
  "sort-unstable",
  "sort-unstable-by",
  "sort-unstable-by-key",
  "dedup",
  "dedup-by",
  "dedup-by-key",
  "binary-search",
  "binary-search-by",
  "binary-search-by-key",
  "match-literal",
  "matches-literal",
  "to-ascii-lowercase-fold",
  "to-uppercase-fold",
  "to-lowercase-fold",
  "strip-prefix-normalize",
  "trim-matches-normalize",
  "entry-format-key",
  "manual-partialeq-newtype",
  "manual-hash-newtype",
  "macro-rules-compare",
  "derived-ord-sort",
  "write-into-buffer-compare",
  "map-to-string-collect-sort",
  "chars-eq",
  "bytes-eq",
  "len-and-starts-with",
  "depth-two-return-compare",
  "argument-position-compare",
];
const generatedPositions = ["same-file", "cross-file", "authority-zero-file"];
const generatedEscapeGrammars = [
  "write-into-call",
  "write-into-ufcs",
  "write-into-fn-pointer",
  "render-authored-helper",
  "serde-json-to-string",
  "serde-json-to-string-pretty",
  "serde-json-to-vec",
  "serde-json-to-vec-pretty",
  "serde-json-to-writer",
  "serde-json-to-writer-pretty",
  "serde-json-to-value",
  "serde-json-value-to-string",
  "json-macro",
  "serde-yaml-to-string",
  "serde-yaml-to-writer",
  "toml-to-string",
  "toml-to-string-pretty",
  "toml-value-to-string",
  "toml-serializer",
  "serde-wasm-bindgen-to-value",
  "serialize-method-call",
  "napi-serde-egress",
  "serializer-impl",
  "debug-format-spec",
  "debug-fmt-ufcs",
  "dbg-macro",
  "format-args-debug",
  "tracing-debug-sigil",
  "aliased-escape-path",
  "serde-json-serializer",
  "serde-json-value-serializer",
  "serde-yaml-to-value",
  "serde-yaml-serializer",
  "serde-yaml-value-serializer",
  "toml-value-serializer",
  "serde-wasm-bindgen-serializer",
];

const pinnedOriginBaseline = [
  "field-access",
  "bare-parameter",
  "fqn-parameter",
  "alias-parameter",
  "closure-inferred",
  "two-statement-local",
  "wrapper-function",
  "named-escape",
];
const pinnedComparisonBaseline = generatedComparisonGrammars.slice(0, 33);
const axisBaselines = {
  origins: "7976976895e7c7c6e6041b58cc4738c1a85b321a9fdd5828489591dc89ada27d",
  comparisons: "4f0b68e1929f55090fddf2560e81fd292006d390ac1a5ec94c5ad32b3b26c19e",
  positions: "ffd26a47bdfef655e9e2cafa05f544deb48354f1153ab53ced4f88285008744e",
};

function vectorComparisonBody(operation) {
  return `{ let mut values: Vec<String> = Vec::new(); values.push(value); ${operation}; true }`;
}

function mapComparisonBody(operation) {
  return `{ let mut values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new(); values.insert(value, 1); ${operation}; true }`;
}

function comparisonBody(grammar) {
  switch (grammar) {
    case "binary-eq":
      return "value == expected";
    case "binary-ne":
      return "value != expected";
    case "method-eq":
      return "value.eq(expected)";
    case "method-ne":
      return "value.ne(expected)";
    case "eq-ignore-ascii-case":
      return "value.eq_ignore_ascii_case(expected)";
    case "cmp-is-eq":
      return "value.cmp(&expected.to_string()).is_eq()";
    case "partial-cmp-is-eq":
      return "value.partial_cmp(&expected.to_string()).is_some_and(std::cmp::Ordering::is_eq)";
    case "ufcs-str-eq":
      return "str::eq(value.as_str(), expected)";
    case "ufcs-partial-eq":
      return "PartialEq::eq(&value, &expected.to_string())";
    case "map-insert":
      return "{ let mut values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new(); values.insert(value, 1); true }";
    case "map-get":
      return mapComparisonBody("let _ = values.get(expected)");
    case "map-entry":
      return mapComparisonBody("let _ = values.entry(expected.to_string())");
    case "map-contains-key":
      return mapComparisonBody("let _ = values.contains_key(expected)");
    case "map-remove":
      return mapComparisonBody("let _ = values.remove(expected)");
    case "set-insert":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); true }";
    case "set-get":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.get(expected); true }";
    case "set-contains":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.contains(expected); true }";
    case "set-remove":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.remove(expected); true }";
    case "sort":
      return vectorComparisonBody("values.sort() ");
    case "sort-by":
      return vectorComparisonBody("values.sort_by(|left, right| left.cmp(right))");
    case "sort-by-key":
      return vectorComparisonBody("values.sort_by_key(|item| item.clone())");
    case "sort-by-cached-key":
      return vectorComparisonBody("values.sort_by_cached_key(|item| item.clone())");
    case "sort-unstable":
      return vectorComparisonBody("values.sort_unstable() ");
    case "sort-unstable-by":
      return vectorComparisonBody("values.sort_unstable_by(|left, right| left.cmp(right))");
    case "sort-unstable-by-key":
      return vectorComparisonBody("values.sort_unstable_by_key(|item| item.clone())");
    case "dedup":
      return vectorComparisonBody("values.dedup() ");
    case "dedup-by":
      return vectorComparisonBody("values.dedup_by(|left, right| left == right)");
    case "dedup-by-key":
      return vectorComparisonBody("values.dedup_by_key(|item| item.clone())");
    case "binary-search":
      return vectorComparisonBody("let _ = values.binary_search(&expected.to_string())");
    case "binary-search-by":
      return vectorComparisonBody(
        "let _ = values.binary_search_by(|item| item.as_str().cmp(expected))",
      );
    case "binary-search-by-key":
      return vectorComparisonBody(
        "let _ = values.binary_search_by_key(&expected.to_string(), |item| item.clone())",
      );
    case "match-literal":
      return 'match value.as_str() { "--token" => true, _ => false }';
    case "matches-literal":
      return 'matches!(value.as_str(), "--token")';
    case "to-ascii-lowercase-fold":
      return "value.to_ascii_lowercase() == expected";
    case "to-uppercase-fold":
      return "value.to_uppercase() == expected";
    case "to-lowercase-fold":
      return "value.to_lowercase() == expected";
    case "strip-prefix-normalize":
      return 'value.strip_prefix("--").unwrap_or(value.as_str()) == expected';
    case "trim-matches-normalize":
      return "value.trim_matches('-') == expected";
    case "entry-format-key":
      return '{ let mut values = std::collections::BTreeMap::new(); values.entry(format!("{value}")).or_insert(1); true }';
    case "manual-partialeq-newtype":
      return "ManualPropertyText(value).eq(&ManualPropertyText(expected.to_string()))";
    case "manual-hash-newtype":
      return "{ let mut state = std::collections::hash_map::DefaultHasher::new(); ManualPropertyText(value).hash(&mut state); state.finish() == 0 }";
    case "macro-rules-compare":
      return "raw_property_compare!(value, expected)";
    case "derived-ord-sort":
      return "{ let mut values = vec![DerivedPropertyText(value)]; values.sort(); true }";
    case "write-into-buffer-compare":
      return "{ let mut rendered = String::new(); let _ = std::fmt::Write::write_str(&mut rendered, value.as_str()); rendered == expected }";
    case "map-to-string-collect-sort":
      return "{ let mut values = [value].into_iter().map(ToString::to_string).collect::<Vec<_>>(); values.sort(); true }";
    case "chars-eq":
      return "value.chars().eq(expected.chars())";
    case "bytes-eq":
      return "value.bytes().eq(expected.bytes())";
    case "len-and-starts-with":
      return "value.len() == expected.len() && value.starts_with(expected)";
    case "depth-two-return-compare":
      return "compare_depth_two(pass_depth_one(value), expected)";
    case "argument-position-compare":
      return "compare_argument_position(value, expected)";
    default:
      throw new Error(`unknown generated comparison grammar: ${grammar}`);
  }
}

function originBinding(origin, carrierType) {
  switch (origin) {
    case "field-access":
      return "let authored = &carrier.property;";
    case "tuple-field-access":
      return "let authored = &tuple_carrier.0;";
    case "bare-parameter":
      return "let authored = property;";
    case "fqn-parameter":
      return "let authored = fqn_property;";
    case "alias-parameter":
      return "let authored = alias_property;";
    case "generic-bound-parameter":
      return "let authored = generic_property.as_ref();";
    case "self-receiver":
      return "let authored = &self_value.property;";
    case "closure-inferred":
      return "let authored = carriers.iter().map(|carrier| &carrier.property).next().unwrap();";
    case "for-loop-binding":
      return "let mut authored = property; for carrier in carriers { authored = &carrier.property; break; }";
    case "while-let-binding":
      return "let mut iterator = carriers.iter(); let authored = while let Some(carrier) = iterator.next() { break &carrier.property; };";
    case "if-let-binding":
      return "let authored = if let Some(carrier) = carriers.first() { &carrier.property } else { property };";
    case "let-else-binding":
      return "let Some(carrier) = carriers.first() else { return false; }; let authored = &carrier.property;";
    case "let-chain-binding":
      return "let authored = if let Some(carrier) = carriers.first() && let true = !carrier.property.is_empty() { &carrier.property } else { property };";
    case "match-arm-binding":
      return "let authored = match carriers.first() { Some(carrier) => &carrier.property, None => property };";
    case "at-binding":
      return "let authored_carrier @ _ = carrier; let authored = &authored_carrier.property;";
    case "or-pattern-binding":
      return "let authored = match Some(carrier) { Some(bound) | None if carriers.is_empty() => &bound.property, _ => property };";
    case "struct-destructuring":
      return `let ${carrierType} { property: authored } = carrier;`;
    case "tuple-destructuring":
      return "let (authored, _) = (&tuple_carrier.0, 0_u8);";
    case "slice-pattern":
      return "let authored = match carriers { [first, ..] => &first.property, [] => property };";
    case "two-statement-local":
      return "let first = &carrier.property; let authored = first;";
    case "wrapper-function":
      return "let authored = authored_wrapper(&carrier.property);";
    case "two-step-wrapper":
      return "let authored = authored_wrapper_two(authored_wrapper_one(&carrier.property));";
    case "trait-method-return":
      return "let authored = AuthoredAccess::authored(carrier);";
    case "accessor-return-inline":
      return "let authored = carrier.authored();";
    case "macro-rules-body":
      return "let authored = authored_from_macro!(carrier);";
    case "named-escape":
      return "let authored = named_authored_escape(&carrier.property);";
    default:
      throw new Error(`unknown generated origin: ${origin}`);
  }
}

function escapeBody(escape) {
  switch (escape) {
    case "write-into-call":
      return "let mut value = String::new(); authored.write_into(&mut value).unwrap();";
    case "write-into-ufcs":
      return "let mut value = String::new(); AuthoredPropertyTextV0::write_into(authored, &mut value).unwrap();";
    case "write-into-fn-pointer":
      return "let emit = AuthoredPropertyTextV0::write_into; let mut value = String::new(); emit(authored, &mut value).unwrap();";
    case "render-authored-helper":
      return "let mut value = String::new(); render_authored(authored, &mut value).unwrap();";
    case "serde-json-to-string":
      return "let value = serde_json::to_string(authored).unwrap();";
    case "serde-json-to-string-pretty":
      return "let value = serde_json::to_string_pretty(authored).unwrap();";
    case "serde-json-to-vec":
      return "let value = String::from_utf8(serde_json::to_vec(authored).unwrap()).unwrap();";
    case "serde-json-to-vec-pretty":
      return "let value = String::from_utf8(serde_json::to_vec_pretty(authored).unwrap()).unwrap();";
    case "serde-json-to-writer":
      return "let mut bytes = Vec::new(); serde_json::to_writer(&mut bytes, authored).unwrap(); let value = String::from_utf8(bytes).unwrap();";
    case "serde-json-to-writer-pretty":
      return "let mut bytes = Vec::new(); serde_json::to_writer_pretty(&mut bytes, authored).unwrap(); let value = String::from_utf8(bytes).unwrap();";
    case "serde-json-to-value":
      return "let value = serde_json::to_value(authored).unwrap().to_string();";
    case "serde-json-value-to-string":
      return "let json_value = serde_json::to_value(authored).unwrap(); let value = serde_json::Value::to_string(&json_value);";
    case "json-macro":
      return "let value = serde_json::json!(authored).to_string();";
    case "serde-yaml-to-string":
      return "let value = serde_yaml_ng::to_string(authored).unwrap();";
    case "serde-yaml-to-writer":
      return "let mut bytes = Vec::new(); serde_yaml_ng::to_writer(&mut bytes, authored).unwrap(); let value = String::from_utf8(bytes).unwrap();";
    case "toml-to-string":
      return "let value = toml::to_string(authored).unwrap();";
    case "toml-to-string-pretty":
      return "let value = toml::to_string_pretty(authored).unwrap();";
    case "toml-value-to-string":
      return "let value = toml::Value::try_from(authored).unwrap().to_string();";
    case "toml-serializer":
      return "let value = authored.serialize(toml::Serializer::new(String::new())).unwrap();";
    case "serde-wasm-bindgen-to-value":
      return "let value = serde_wasm_bindgen::to_value(authored).unwrap();";
    case "serialize-method-call":
      return "let serializer = product_serializer(); let value = authored.serialize(serializer).unwrap().to_string();";
    case "napi-serde-egress":
      return "let value = napi_serde_egress(authored);";
    case "serializer-impl":
      return "let value = product_serializer_impl(authored);";
    case "debug-format-spec":
      return 'let value = format!("{authored:#?}");';
    case "debug-fmt-ufcs":
      return "let mut value = String::new(); std::fmt::Debug::fmt(authored, &mut value).unwrap();";
    case "dbg-macro":
      return 'let value = format!("{:?}", dbg!(authored));';
    case "format-args-debug":
      return 'let value = std::fmt::format(format_args!("{authored:?}"));';
    case "tracing-debug-sigil":
      return "let value = tracing_debug_value(debug = ?authored);";
    case "aliased-escape-path":
      return "use serde_json::to_string as encode_authored; let value = encode_authored(authored).unwrap();";
    case "serde-json-serializer":
      return "let mut bytes = Vec::new(); let mut serializer = serde_json::Serializer::new(&mut bytes); authored.serialize(&mut serializer).unwrap(); let value = String::from_utf8(bytes).unwrap();";
    case "serde-json-value-serializer":
      return "let value = authored.serialize(serde_json::value::Serializer).unwrap().to_string();";
    case "serde-yaml-to-value":
      return "let value = serde_yaml_ng::to_value(authored).unwrap().to_string();";
    case "serde-yaml-serializer":
      return "let mut out = String::new(); let serializer = serde_yaml_ng::Serializer::new(&mut out); let value = authored.serialize(serializer).unwrap().to_string();";
    case "serde-yaml-value-serializer":
      return "let value = authored.serialize(serde_yaml_ng::value::Serializer).unwrap().to_string();";
    case "toml-value-serializer":
      return "let value = authored.serialize(toml::ser::ValueSerializer::new()).unwrap().to_string();";
    case "serde-wasm-bindgen-serializer":
      return "let serializer = serde_wasm_bindgen::Serializer::json_compatible(); let value = authored.serialize(&serializer).unwrap();";
    default:
      throw new Error(`unknown generated escape grammar: ${escape}`);
  }
}

function generatedCellSource(functionName, origin, carrierType, grammar, escape) {
  const operation = comparisonBody(grammar);
  const production = originBinding(origin, carrierType);
  const escaping = escape ? escapeBody(escape) : escapeBody("render-authored-helper");
  return `fn ${functionName}<T: AsRef<AuthoredPropertyTextV0>>(carrier: &${carrierType}, tuple_carrier: &(AuthoredPropertyTextV0, u8), carriers: &[${carrierType}], property: &AuthoredPropertyTextV0, fqn_property: &omena_syntax::ident::AuthoredPropertyTextV0, alias_property: &AuthoredText, generic_property: T, self_value: &${carrierType}, expected: &str) -> bool { ${production} ${escaping} ${operation} }`;
}

function generatedMatrixManifest() {
  const sources = [];
  const fullProductCells = [];
  const escapeCoveringCells = [];
  for (const [positionIndex, position] of generatedPositions.entries()) {
    const carrierName = `GeneratedPropertyCarrier${positionIndex}`;
    const carrierPath =
      position === "cross-file"
        ? "rust/crates/omena-cascade/src/generated_property_matrix_axis_order.rs"
        : `rust/crates/omena-query/src/generated_property_matrix_${positionIndex}.rs`;
    const consumerPath =
      position === "cross-file"
        ? "rust/crates/omena-cascade/src/generated_property_matrix_ranking.rs"
        : carrierPath;
    const prelude = `use omena_syntax::ident::AuthoredPropertyTextV0;\nuse omena_syntax::ident::AuthoredPropertyTextV0 as AuthoredText;\nstruct ${carrierName} { property: AuthoredPropertyTextV0 }\n`;
    if (carrierPath !== consumerPath) sources.push({ relativePath: carrierPath, source: prelude });
    const functions = [];
    for (const [originIndex, origin] of generatedOrigins.entries()) {
      for (const [grammarIndex, grammar] of generatedComparisonGrammars.entries()) {
        const functionName = `generated_property_full_${positionIndex}_${originIndex}_${grammarIndex}`;
        fullProductCells.push({ functionName, origin, comparison: grammar, position });
        functions.push(generatedCellSource(functionName, origin, carrierName, grammar));
      }
    }
    for (const [escapeIndex, escape] of generatedEscapeGrammars.entries()) {
      for (const [grammarIndex, grammar] of generatedComparisonGrammars.entries()) {
        const originIndex = grammarIndex % generatedOrigins.length;
        const targetPositionIndex = grammarIndex % generatedPositions.length;
        if (targetPositionIndex !== positionIndex) continue;
        const origin = generatedOrigins[originIndex];
        const functionName = `generated_property_escape_${positionIndex}_${escapeIndex}_${grammarIndex}`;
        escapeCoveringCells.push({
          functionName,
          escape,
          origin,
          comparison: grammar,
          position,
        });
        functions.push(generatedCellSource(functionName, origin, carrierName, grammar, escape));
      }
    }
    sources.push({
      relativePath: consumerPath,
      source: `${carrierPath === consumerPath ? prelude : `use crate::axis_order::${carrierName};\nuse omena_syntax::ident::AuthoredPropertyTextV0;\nuse omena_syntax::ident::AuthoredPropertyTextV0 as AuthoredText;\n`}\n${functions.join("\n")}`,
    });
  }
  return {
    schemaVersion: "1",
    axes: {
      origins: generatedOrigins,
      comparisons: generatedComparisonGrammars,
      positions: generatedPositions,
      escapes: generatedEscapeGrammars,
      pinnedOriginBaseline,
      pinnedComparisonBaseline,
      axisBaselines,
    },
    sources,
    fullProductCells,
    escapeCoveringCells,
  };
}

const redCases = [
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_SCAN", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_COMPARE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LEXER_CASE_COMPARE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_EXEMPTION_DRIFT", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LESS_SCANNER_CALL", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_CLASS_SCANNER", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SECOND_SELECTOR_AUTHORITY", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SELECTOR_REPORT_STRUCT_LITERAL", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SELECTOR_REPORT_AUTHORITY_SEVERANCE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACE_DOUBLE_QUOTE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACEN", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CLASSNAME_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS", ["--write"]],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_LABELLED_COMPARISON", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_EXPLICIT", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_REVERSED", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP", []],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION",
    [],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CASE_FOLD", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_DECODE_NEUTER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM", []],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITHOUT_JOIN",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITH_JOIN",
    [],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CONTAINER_PREDICATE_REVERT", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CANONICAL_VECTOR_CARRIER", []],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CANONICAL_VECTOR_CARRIER",
    ["--write"],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_ENTRY_PARAMETER_INVENTORY",
    ["--write"],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_STATIC_LITERAL_INVENTORY",
    ["--write"],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_NON_PROPERTY_INVENTORY",
    ["--write"],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_IDENTITY_CONSUMER", []],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_SANCTIONED_ESCAPE_INVENTORY",
    ["--write"],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_WRITE_INTO_INVENTORY", ["--write"]],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EXTERNAL_LEAF_INVENTORY",
    ["--write"],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_WRAPPER_ESCAPE_IDENTITY",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_CONTAINER_ESCAPE_IDENTITY",
    [],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NAME_ESCAPE_IDENTITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_DERIVED_CARRIER_IDENTITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_SELECTOR_DEFINITION_SORT", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_TRANSFORM_NODE_SORT", []],
];

const requiredMutationReceipts = new Map([
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITHOUT_JOIN\0",
    ["residualKeyedCarrierWithoutJoin=inventoried"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITH_JOIN\0",
    ["residualKeyedCarrierWithJoin=inventoried", "rawPropertyIdentitySiteCount="],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CONTAINER_PREDICATE_REVERT\0",
    ["knownAuthoredVectorCarrier=missing-after-container-predicate-revert"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CANONICAL_VECTOR_CARRIER\0",
    ["canonicalVectorCarrier=p-canonical:R4"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CANONICAL_VECTOR_CARRIER\0--write",
    [
      "canonicalVectorCarrier=p-canonical:R4",
      "residual class inventory changed for canonical-text-carrier",
    ],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_ENTRY_PARAMETER_INVENTORY\0--write",
    ["residual class inventory changed for entry-parameter"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_STATIC_LITERAL_INVENTORY\0--write",
    ["residual class inventory changed for static-standard-literal"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_NON_PROPERTY_INVENTORY\0--write",
    ["residual class inventory changed for non-property"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_SANCTIONED_ESCAPE_INVENTORY\0--write",
    ["sanctioned escape inventory changed"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_WRITE_INTO_INVENTORY\0--write",
    ["write_into site inventory changed"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EXTERNAL_LEAF_INVENTORY\0--write",
    ["external leaf type inventory changed"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_WRAPPER_ESCAPE_IDENTITY\0",
    ["authoredEscapeIdentityViolationCount=1", "authored-bearing escape result reached"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_CONTAINER_ESCAPE_IDENTITY\0",
    ["authoredEscapeIdentityViolationCount=1", "authored-bearing escape result reached"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NAME_ESCAPE_IDENTITY\0",
    ["authoredEscapeIdentityViolationCount=1", "authored-bearing escape result reached"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_DERIVED_CARRIER_IDENTITY\0",
    ["authored-bearing carrier regained derived identity"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_SELECTOR_DEFINITION_SORT\0",
    ["authoredEscapeIdentityViolationCount=1", "authored-bearing escape result reached"],
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_TRANSFORM_NODE_SORT\0",
    ["authoredEscapeIdentityViolationCount=1", "authored-bearing escape result reached"],
  ],
]);

const rawPropertyMutationVariables = new Set([
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITH_JOIN",
]);
const residualConsumerMutationVariables = new Set([
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_IDENTITY_CONSUMER",
]);

function spawnCaptured(command, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, options);
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.once("error", reject);
    child.once("close", (status, signal) => {
      resolve({ status, signal, stdout, stderr });
    });
  });
}

async function mapWithConcurrency(items, concurrency, transform) {
  const results = Array.from({ length: items.length });
  let nextIndex = 0;
  function worker() {
    const index = nextIndex;
    nextIndex += 1;
    if (index >= items.length) {
      return Promise.resolve();
    }
    return Promise.resolve(transform(items[index])).then((result) => {
      results[index] = result;
      return worker();
    });
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, () => worker()));
  return results;
}

let failures = 0;
if (!generatedMatrixOnly) {
  const redCaseResults = await mapWithConcurrency(redCases, 4, ([checker, variable, args]) =>
    spawnCaptured("node", ["--import", "tsx", checker, ...args], {
      cwd: repoRoot,
      env: { ...process.env, [variable]: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    }),
  );
  for (const [[, variable, args], result] of redCases.map((redCase, index) => [
    redCase,
    redCaseResults[index],
  ])) {
    const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
    const rawPropertySiteCount = Number(
      output.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
    );
    const residualIdentityConsumerCount = Number(
      output.match(/residualIdentityShapedConsumerCount=(\d+)/u)?.[1] ?? "0",
    );
    const requiresRawPropertySite = rawPropertyMutationVariables.has(variable);
    const requiresResidualIdentityConsumer = residualConsumerMutationVariables.has(variable);
    const requiredReceipts = requiredMutationReceipts.get(`${variable}\0${args.join(" ")}`) ?? [];
    const passed =
      result.status !== 0 &&
      (!requiresRawPropertySite || rawPropertySiteCount > 0) &&
      (!requiresResidualIdentityConsumer || residualIdentityConsumerCount > 0) &&
      requiredReceipts.every((receipt) => output.includes(receipt));
    if (!passed) failures += 1;
    const suffix = args.length > 0 ? ` ${args.join(" ")}` : "";
    const rawReceipt = requiresRawPropertySite
      ? `; rawPropertyIdentitySiteCount=${rawPropertySiteCount}`
      : "";
    const residualReceipt = requiresResidualIdentityConsumer
      ? `; residualIdentityShapedConsumerCount=${residualIdentityConsumerCount}`
      : "";
    process.stdout.write(
      `${passed ? "ok  " : "FAIL"} ${variable}${suffix} exits non-zero${rawReceipt}${residualReceipt}\n`,
    );
  }
}
const redFailures = failures;

const generatedTempRoot = mkdtempSync(path.join(tmpdir(), "omena-identifier-matrix-"));
const generatedManifestPath = path.join(generatedTempRoot, "manifest.json");
const generatedManifest = generatedMatrixManifest();
writeFileSync(generatedManifestPath, `${JSON.stringify(generatedManifest)}\n`);
const generatedResult = spawnSync(
  "node",
  ["--import", "tsx", identifierChecker, "--generated-fixture-only"],
  {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST: generatedManifestPath,
    },
    maxBuffer: 16 * 1024 * 1024,
  },
);
const generatedOutput = `${generatedResult.stdout ?? ""}\n${generatedResult.stderr ?? ""}`;
const generatedFullProductCount = Number(
  generatedOutput.match(/generatedFullProductCellCount=(\d+)/u)?.[1] ?? "0",
);
const generatedEscapeCoveringCount = Number(
  generatedOutput.match(/generatedEscapeCoveringCellCount=(\d+)/u)?.[1] ?? "0",
);
const generatedPairFamilyCount = Number(
  generatedOutput.match(/generatedPairFamilies=(\d+)\/6/u)?.[1] ?? "0",
);
const generatedPassed =
  generatedResult.status === 0 &&
  generatedFullProductCount === generatedManifest.fullProductCells.length &&
  generatedEscapeCoveringCount === generatedManifest.escapeCoveringCells.length &&
  generatedPairFamilyCount === 6;
if (!generatedPassed) {
  failures += 1;
  process.stderr.write(generatedOutput);
}
process.stdout.write(
  `${generatedPassed ? "ok  " : "FAIL"} generated authored matrix: ${generatedFullProductCount}/${generatedManifest.fullProductCells.length} full-product cells + ${generatedEscapeCoveringCount}/${generatedManifest.escapeCoveringCells.length} escape covering cells; ${generatedPairFamilyCount}/6 pair families; ${generatedOrigins.length} origins x ${generatedComparisonGrammars.length} comparisons x ${generatedPositions.length} positions; ${generatedEscapeGrammars.length} entry-derived escapes\n`,
);
const generatedMutationCases = [
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_FOR_LOOP_RESOLVER",
    "generated_property_full_0_8_0: no authored origin arm fired",
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ARGUMENT_RETURN_EDGE",
    "generated_property_full_0_0_49: no comparison arm fired",
  ],
  [
    "OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ENTRY_POINT_ID",
    "serialization entry point has no escape id or the escape grammar shrank",
  ],
];
let generatedMutationFailures = 0;
for (const [variable, receipt] of generatedMutationCases) {
  const mutation = spawnSync(
    "node",
    ["--import", "tsx", identifierChecker, "--generated-fixture-only"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        [variable]: "1",
        OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST: generatedManifestPath,
      },
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  const output = `${mutation.stdout ?? ""}\n${mutation.stderr ?? ""}`;
  const passed = mutation.status !== 0 && output.includes(receipt);
  if (!passed) {
    failures += 1;
    generatedMutationFailures += 1;
    process.stderr.write(output);
  }
  process.stdout.write(
    `${passed ? "ok  " : "FAIL"} ${variable} RED names ${JSON.stringify(receipt)}\n`,
  );
}
rmSync(generatedTempRoot, { recursive: true, force: true });
if (generatedMatrixOnly) {
  process.exit(generatedPassed && generatedMutationFailures === 0 ? 0 : 1);
}

const committedLaunderingCensusPath = path.join(
  repoRoot,
  "rust/omena-identifier-authority-census.json",
);
const exactLaunderingTempRoot = mkdtempSync(path.join(tmpdir(), "omena-identifier-laundering-"));
const exactLaunderingCensusPath = path.join(exactLaunderingTempRoot, "census.json");
writeFileSync(exactLaunderingCensusPath, readFileSync(committedLaunderingCensusPath));
const originalLaunderingCensus = readFileSync(exactLaunderingCensusPath);
const baselineAuthorityCount = JSON.parse(originalLaunderingCensus.toString("utf8"))
  .propertyIdentity.authoritySiteCount;
let exactLaunderingPassed = false;
let exactLaunderingWriteCount = 0;
let exactLaunderingRecheckCount = 0;
let exactLaunderingWriteAuthorityCount = 0;
let exactLaunderingRecheckAuthorityCount = 0;

try {
  const writeAttempt = spawnSync("node", ["--import", "tsx", identifierChecker, "--write"], {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING: "1",
      OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH: exactLaunderingCensusPath,
    },
  });
  const writeOutput = `${writeAttempt.stdout ?? ""}\n${writeAttempt.stderr ?? ""}`;
  exactLaunderingWriteCount = Number(
    writeOutput.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  exactLaunderingWriteAuthorityCount = Number(
    writeOutput.match(/propertyAuthoritySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  const censusUnchangedAfterWrite =
    readFileSync(exactLaunderingCensusPath).equals(originalLaunderingCensus);

  const recheck = spawnSync("node", ["--import", "tsx", identifierChecker], {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING: "1",
      OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH: exactLaunderingCensusPath,
    },
  });
  const recheckOutput = `${recheck.stdout ?? ""}\n${recheck.stderr ?? ""}`;
  exactLaunderingRecheckCount = Number(
    recheckOutput.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  exactLaunderingRecheckAuthorityCount = Number(
    recheckOutput.match(/propertyAuthoritySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  const censusUnchangedAfterRecheck =
    readFileSync(exactLaunderingCensusPath).equals(originalLaunderingCensus);

  exactLaunderingPassed =
    writeAttempt.status !== 0 &&
    exactLaunderingWriteCount > 0 &&
    exactLaunderingWriteAuthorityCount === baselineAuthorityCount - 1 &&
    recheck.status !== 0 &&
    exactLaunderingRecheckCount > 0 &&
    exactLaunderingRecheckAuthorityCount === baselineAuthorityCount - 1 &&
    censusUnchangedAfterWrite &&
    censusUnchangedAfterRecheck;
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
} finally {
  rmSync(exactLaunderingTempRoot, { recursive: true, force: true });
}

if (!exactLaunderingPassed) failures += 1;
process.stdout.write(
  `${exactLaunderingPassed ? "ok  " : "FAIL"} exact laundering: eq_ignore_ascii_case; authority ${baselineAuthorityCount}->${exactLaunderingWriteAuthorityCount}; --write ${exactLaunderingWriteCount > 0 ? "RED" : "MISS"} raw=${exactLaunderingWriteCount}; recheck ${exactLaunderingRecheckCount > 0 ? "RED" : "MISS"} raw=${exactLaunderingRecheckCount}; census unchanged; in-memory mutation\n`,
);

const unlabelled = spawnSync("node", ["--import", "tsx", identifierChecker], {
  cwd: repoRoot,
  encoding: "utf8",
  env: {
    ...process.env,
    OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNLABELLED_COMPARISON: "1",
  },
});
const blindSpotDisclosed = unlabelled.status === 0;
if (!blindSpotDisclosed) failures += 1;
process.stdout.write(
  `${blindSpotDisclosed ? "ok  " : "FAIL"} disclosed GREEN control: unlabelled class binding remains outside the idiom arm\n`,
);

process.stdout.write(
  `\n${redCases.length - redFailures}/${redCases.length} injected RED mutation arms; ${generatedPassed ? `${generatedFullProductCount + generatedEscapeCoveringCount}/${generatedManifest.fullProductCells.length + generatedManifest.escapeCoveringCells.length}` : `0/${generatedManifest.fullProductCells.length + generatedManifest.escapeCoveringCells.length}`} generated matrix cells with 6/6 pair families; ${exactLaunderingPassed ? "1/1" : "0/1"} exact laundering arm; ${blindSpotDisclosed ? "1/1" : "0/1"} disclosed GREEN control arm\n`,
);
process.exit(failures === 0 ? 0 : 1);
