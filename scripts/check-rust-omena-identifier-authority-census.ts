import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

type Disposition = "sanctioned" | "named-exempt" | "unclassified";
type PrimitiveId = "str-eq" | "contains" | "insert" | "map-get" | "cmp";
type ResidualPropertyProvenance = "p-authored" | "p-canonical" | "p-non-property";
type ResidualPropertyClass =
  | "entry-parameter"
  | "static-standard-literal"
  | "fixture-crate"
  | "canonical-text-carrier"
  | "non-property";

interface ProductPathMatrix {
  readonly schemaVersion: "0";
  readonly product: "omena-css.product-path-matrix";
  readonly entries: readonly {
    readonly crate: string;
    readonly role: string;
  }[];
}

interface CensusSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
  readonly disposition: Exclude<Disposition, "unclassified">;
  readonly reason?: string;
  readonly provenance?: ResidualPropertyProvenance;
  readonly provenanceDerivation?: string;
  readonly residualClass?: ResidualPropertyClass;
}

interface DiscoveredSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
  readonly provenance?: ResidualPropertyProvenance;
  readonly provenanceDerivation?: string;
  readonly residualClass?: ResidualPropertyClass;
}

type PropertyIdentitySiteRole = "authority" | "producer" | "consumer" | "carrier";

interface PropertyIdentitySite extends DiscoveredSite {
  readonly role: PropertyIdentitySiteRole;
}

type ResidualPropertyConsumerClassification = "presentation" | "egress" | "identity-shaped";

interface ResidualPropertyConsumerSite extends DiscoveredSite {
  readonly classification: ResidualPropertyConsumerClassification;
}

interface ResidualPropertyCarrierConsumerRow {
  readonly carrier: CensusSite;
  readonly consumers: readonly ResidualPropertyConsumerSite[];
}

type AuthoredEscapeOperandClass = "authored-bearing" | "key-bearing" | "non-property";
type AuthoredEscapeDisposition =
  | "presentation"
  | "egress"
  | "sanctioned-identity"
  | "identity-violation";
type WriteIntoSinkClass = "emitter-output" | "formatter" | "returned-to-emitter" | "unresolved";

interface AuthoredEscapeSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly escapeId: AuthoredEscapeId;
  readonly evidence: string;
  readonly operandClasses: readonly AuthoredEscapeOperandClass[];
  readonly operandDerivation: string;
  readonly disposition: AuthoredEscapeDisposition;
  readonly resultBindings: readonly string[];
}

interface AuthoredEscapeIdentityFlow {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly comparisonId: AuthoredComparisonId;
  readonly escapeIds: readonly AuthoredEscapeId[];
  readonly evidence: string;
  readonly operandDerivation: string;
  readonly sanctioned: boolean;
}

interface WriteIntoSinkSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly escapeId:
    | "write-into-call"
    | "write-into-ufcs"
    | "write-into-fn-pointer"
    | "render-authored-helper";
  readonly sinkClass: WriteIntoSinkClass;
  readonly sinkBinding: string;
  readonly evidence: string;
  readonly derivation: string;
}

interface AuthoredIdentityCarrierAudit {
  readonly typeName: string;
  readonly paths: readonly string[];
  readonly identityImpls: readonly string[];
  readonly discriminatingTests: readonly string[];
}

interface PropertyIdentityTypeSeal {
  readonly path: "rust/crates/omena-syntax/src/ident.rs";
  readonly authoredTextDerives: readonly [
    "Debug",
    "Clone",
    "serde::Serialize",
    "serde::Deserialize",
  ];
  readonly authoredTextIdentityDerives: readonly [];
  readonly authoredTextFieldVisibility: "private";
  readonly authoredTextPresentationMethods: readonly ["write_into", "Serialize", "Deserialize"];
  readonly propertyNameDerives: readonly ["Debug", "Clone"];
  readonly propertyNameEqualityDerives: readonly [];
  readonly propertyNameMethods: readonly [
    "from_authored",
    "new",
    "standard",
    "custom",
    "kind",
    "authored_text",
    "canonical_name",
    "same_as",
    "canonical_key",
    "as_custom_key",
    "as_standard_key",
    "canonical_custom_key",
    "canonical_standard_key",
  ];
  readonly standardKeyDerives: readonly [
    "Debug",
    "Clone",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
  ];
  readonly customKeyDerives: readonly [
    "Debug",
    "Clone",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
  ];
  readonly keyFieldVisibility: "private";
  readonly sharedDecoder: "decode_css_identifier_escapes";
  readonly standardCanonicalization: "trim-decode-ascii-lowercase";
  readonly customCanonicalization: "trim-decode-case-preserved";
  readonly equalityMethod: "PropertyNameV0::same_as";
}

interface IdentifierAuthorityCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.identifier-authority.census";
  readonly policy: {
    readonly expectedSide: "committed-authored-table";
    readonly addedSiteAdoption: "explicit-review-flag-required";
    readonly owningCheck: "rust/omena-syntax-authority-raw-scan-census";
    readonly packageScript: "check:rust-omena-syntax-authority-raw-scan-census";
  };
  readonly sourceRoots: readonly ["rust/crates"];
  readonly scope: {
    readonly scannedCrates: readonly string[];
    readonly scannedCratesDigest: string;
    readonly engineCrates: readonly string[];
    readonly engineCratesDigest: string;
  };
  readonly preflight: {
    readonly productPin: "c18606086dc77101ad162c8d0ddb36ccae7e9d85";
    readonly measuredSiteCount: 50;
    readonly primitiveCounts: {
      readonly "str-eq": 24;
      readonly contains: 9;
      readonly insert: 10;
      readonly "map-get": 5;
      readonly cmp: 2;
    };
  };
  readonly typeSeal: {
    readonly path: "rust/crates/omena-syntax/src/ident.rs";
    readonly classNameDerives: readonly ["Debug", "Clone"];
    readonly classNameEqualityDerives: readonly [];
    readonly canonicalKeyDerives: readonly [
      "Debug",
      "Clone",
      "PartialEq",
      "Eq",
      "PartialOrd",
      "Ord",
      "Hash",
    ];
    readonly canonicalKeyFieldVisibility: "private";
    readonly canonicalKeyConstructor: "ClassNameV0::canonical_key";
    readonly equalityMethod: "ClassNameV0::same_as";
    readonly mutatorCrates: readonly string[];
    readonly privacyDiscriminator: string;
  };
  readonly egress: {
    readonly direction: "decrease-only";
    readonly meaningRevisionReviews: readonly {
      readonly previousSiteCount: number;
      readonly currentSiteCount: number;
      readonly disposition: "reviewed-selector-authority-rekey";
      readonly reason: string;
    }[];
    readonly baselineSiteCount: number;
    readonly currentSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly idiom: {
    readonly granularity: "line";
    readonly meaningRevisionReviews: readonly {
      readonly disposition: "reviewed-sealed-selector-key-migration";
      readonly reason: string;
    }[];
    readonly lexemeGlobs: readonly string[];
    readonly primitiveIds: readonly PrimitiveId[];
    readonly unlabelledBindingBlindSpot: string;
    readonly currentSiteCount: number;
    readonly primitiveCounts: Readonly<Record<PrimitiveId, number>>;
    readonly sanctionedSiteCount: number;
    readonly namedExemptSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly predicateCopies: {
    readonly derivation: "direct-character-predicate-body";
    readonly blindSpots: readonly string[];
    readonly currentSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly propertyIdentity: {
    readonly derivation: "authority-token-and-property-key-operation-scan";
    readonly rawSiteDerivation: "typed-property-operand-and-raw-key-dataflow-scan";
    readonly typeSeal: PropertyIdentityTypeSeal;
    readonly authorityInventoryDecreaseReviews: readonly {
      readonly previousSiteCount: number;
      readonly currentSiteCount: number;
      readonly disposition: "reviewed-structural-refactor";
      readonly reason: string;
    }[];
    readonly authoritySiteCount: number;
    readonly sites: readonly PropertyIdentitySite[];
    readonly siteDigest: string;
    readonly residualRawCarrierDirection: "decrease-only";
    readonly residualRawCarrierSiteCount: number;
    readonly residualRawCarrierSites: readonly CensusSite[];
    readonly residualRawCarrierSiteDigest: string;
    readonly residualClassCounts: Readonly<Record<ResidualPropertyClass, number>>;
    readonly residualProvenanceCounts: Readonly<Record<ResidualPropertyProvenance, number>>;
    readonly residualCarrierConsumerDerivation: "resolved-type-access-and-parameter-use-scan";
    readonly residualCarrierConsumerRows: readonly ResidualPropertyCarrierConsumerRow[];
    readonly residualCarrierConsumerSiteCount: number;
    readonly residualIdentityShapedConsumerCount: 0;
    readonly residualCarrierConsumerDigest: string;
    readonly rawStringIdentitySiteCount: 0;
    readonly rawStringIdentitySites: readonly [];
    readonly rawStringIdentitySiteDigest: string;
    readonly egressHonestyTable: {
      readonly derivation: "final-head-byte-differential";
      readonly orderBranchSurfaces: readonly {
        readonly owner: string;
        readonly changeId: string;
        readonly surface: string;
        readonly evidenceNeedles: readonly string[];
      }[];
      readonly valueBranchSurfaces: readonly {
        readonly owner: string;
        readonly changeId: string;
        readonly surface: string;
        readonly evidenceNeedles: readonly string[];
      }[];
      readonly zeroBranchSurfaces: readonly {
        readonly surface: string;
        readonly evidenceGate: string;
      }[];
    };
    readonly authoredEscapeClosure: {
      readonly derivation: "manifest-entry-points-plus-rust-dataflow-fixpoint";
      readonly axes: {
        readonly origins: readonly AuthoredOriginId[];
        readonly comparisons: readonly AuthoredComparisonId[];
        readonly positions: readonly AuthoredPositionId[];
        readonly escapes: readonly AuthoredEscapeId[];
      };
      readonly axisDigests: {
        readonly origins: string;
        readonly comparisons: string;
        readonly positions: string;
        readonly escapes: string;
      };
      readonly axisBaselines: {
        readonly origins: "sha256:7976976895e7c7c6e6041b58cc4738c1a85b321a9fdd5828489591dc89ada27d";
        readonly comparisons: "sha256:4f0b68e1929f55090fddf2560e81fd292006d390ac1a5ec94c5ad32b3b26c19e";
        readonly positions: "sha256:ffd26a47bdfef655e9e2cafa05f544deb48354f1153ab53ced4f88285008744e";
      };
      readonly fullProductCellCount: 3900;
      readonly escapeCoveringCellCount: 1800;
      readonly pairFamilies: readonly [
        "origin-comparison",
        "origin-position",
        "comparison-position",
        "escape-origin",
        "escape-comparison",
        "escape-position",
      ];
      readonly externalLeafTypes: readonly {
        readonly typeName: string;
        readonly reason: string;
      }[];
      readonly escapeSiteCount: number;
      readonly escapeSites: readonly AuthoredEscapeSite[];
      readonly escapeSiteDigest: string;
      readonly identityFlowCount: number;
      readonly identityFlows: readonly AuthoredEscapeIdentityFlow[];
      readonly identityFlowDigest: string;
      readonly identityViolationCount: 0;
      readonly sanctionedIdentityCount: number;
      readonly writeIntoSiteCount: number;
      readonly writeIntoSites: readonly WriteIntoSinkSite[];
      readonly writeIntoSiteDigest: string;
      readonly unresolvedCallEdges: readonly CensusSite[];
      readonly unresolvedCallEdgeCount: number;
      readonly unresolvedCallEdgeDigest: string;
      readonly carrierAudit: readonly AuthoredIdentityCarrierAudit[];
      readonly carrierAuditDigest: string;
      readonly identityDeriveViolationCount: 0;
      readonly cannotSee: readonly string[];
    };
  };
}

interface ExemptionRule {
  readonly path: string;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
  readonly reason: string;
  readonly disposition?: Exclude<Disposition, "unclassified">;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = process.env.OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH
  ? path.resolve(process.env.OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH)
  : path.join(repoRoot, "rust/omena-identifier-authority-census.json");
const writeMode = process.argv.includes("--write");
const acceptInventoryChange = process.argv.includes("--accept-inventory-change");
const injectClassNameEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CLASSNAME_EQUALITY === "1";
const injectEgress = process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS === "1";
const injectLabelledComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_LABELLED_COMPARISON === "1";
const injectUnlabelledComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNLABELLED_COMPARISON === "1";
const injectPredicateCopy =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY === "1";
const injectPredicateCopyExplicit =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_EXPLICIT === "1";
const injectPredicateCopyReversed =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_REVERSED === "1";
const injectPropertyStructuralEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY === "1";
const injectPropertyRoundtripEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY === "1";
const injectPropertyRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP === "1";
const injectPropertyRawCanonicalization =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION === "1";
const injectPropertyFqnRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP === "1";
const injectPropertyValuesRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP === "1";
const injectPropertySameLineRawOperation =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION === "1";
const injectPropertyNewFileRawComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON === "1";
const injectPropertyNewFileRawCanonicalization =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION === "1";
const injectPropertyTrimChain =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN === "1";
const injectPropertyContextRawOperations =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS === "1";
const injectPropertyCaseFold =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CASE_FOLD === "1";
const injectPropertyDecodeNeuter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_DECODE_NEUTER === "1";
const injectPropertyAuthorityDecreaseLaundering =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING === "1";
const injectAuthoredUppercaseTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM === "1";
const injectAuthoredTrimMatchesTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM === "1";
const injectAuthoredStripPrefixTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM === "1";
const injectResidualKeyedCarrierWithoutJoin =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITHOUT_JOIN === "1";
const injectResidualKeyedCarrierWithJoin =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_KEYED_CARRIER_WITH_JOIN === "1";
const injectContainerPredicateRevert =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CONTAINER_PREDICATE_REVERT === "1";
const injectCanonicalVectorCarrier =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CANONICAL_VECTOR_CARRIER === "1";
const injectResidualEntryParameterInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_ENTRY_PARAMETER_INVENTORY === "1";
const injectResidualStaticLiteralInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_STATIC_LITERAL_INVENTORY === "1";
const injectResidualNonPropertyInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_NON_PROPERTY_INVENTORY === "1";
const injectResidualIdentityConsumer =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RESIDUAL_IDENTITY_CONSUMER === "1";
const injectSanctionedEscapeInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_SANCTIONED_ESCAPE_INVENTORY === "1";
const injectWriteIntoInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_WRITE_INTO_INVENTORY === "1";
const injectExternalLeafInventory =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EXTERNAL_LEAF_INVENTORY === "1";
const injectAuthoredWrapperEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_WRAPPER_ESCAPE_IDENTITY === "1";
const injectAuthoredContainerEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_CONTAINER_ESCAPE_IDENTITY === "1";
const injectPropertyNameEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NAME_ESCAPE_IDENTITY === "1";
const injectDerivedCarrierIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_DERIVED_CARRIER_IDENTITY === "1";
const injectRawSelectorDefinitionSort =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_SELECTOR_DEFINITION_SORT === "1";
const injectRawTransformNodeSort =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_RAW_TRANSFORM_NODE_SORT === "1";
const generatedFixtureManifestPath =
  process.env.OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST;
const generatedFixtureOnly = process.argv.includes("--generated-fixture-only");
const injectGeneratedForLoopResolverDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_FOR_LOOP_RESOLVER === "1";
const injectGeneratedArgumentReturnEdgeDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ARGUMENT_RETURN_EDGE === "1";
const injectGeneratedEntryPointIdDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ENTRY_POINT_ID === "1";

const authoredOriginAxis = [
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
] as const;
const authoredComparisonAxis = [
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
] as const;
const authoredPositionAxis = ["same-file", "cross-file", "authority-zero-file"] as const;
const authoredEscapeAxis = [
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
] as const;
const externalLeafTypes = injectExternalLeafInventory
  ? [
      {
        typeName: "InjectedExternalLeafV0",
        reason: "selftest mutation proving external-leaf inventory laundering is refused",
      },
    ]
  : [];

type AuthoredOriginId = (typeof authoredOriginAxis)[number];
type AuthoredComparisonId = (typeof authoredComparisonAxis)[number];
type AuthoredPositionId = (typeof authoredPositionAxis)[number];
type AuthoredEscapeId = (typeof authoredEscapeAxis)[number];

interface GeneratedFixtureCell {
  readonly functionName: string;
  readonly origin: AuthoredOriginId;
  readonly comparison: AuthoredComparisonId;
  readonly position: AuthoredPositionId;
  readonly escape?: AuthoredEscapeId;
}

interface GeneratedFixtureManifest {
  readonly schemaVersion: "1";
  readonly axes: {
    readonly origins: readonly string[];
    readonly comparisons: readonly string[];
    readonly positions: readonly string[];
    readonly escapes: readonly string[];
    readonly pinnedOriginBaseline: readonly string[];
    readonly pinnedComparisonBaseline: readonly string[];
    readonly axisBaselines: {
      readonly origins: string;
      readonly comparisons: string;
      readonly positions: string;
    };
  };
  readonly sources: readonly MutableRustSource[];
  readonly fullProductCells: readonly GeneratedFixtureCell[];
  readonly escapeCoveringCells: readonly (GeneratedFixtureCell & {
    readonly escape: AuthoredEscapeId;
  })[];
}

const generatedFixtureManifest = generatedFixtureManifestPath
  ? (JSON.parse(readFileSync(generatedFixtureManifestPath, "utf8")) as GeneratedFixtureManifest)
  : undefined;
if (generatedFixtureManifest) {
  assert.equal(generatedFixtureManifest.schemaVersion, "1", "generated fixture manifest schema");
  assert.ok(
    generatedFixtureManifest.sources.length > 0 &&
      generatedFixtureManifest.fullProductCells.length > 0 &&
      generatedFixtureManifest.escapeCoveringCells.length > 0,
    "generated fixture manifest must be non-empty",
  );
}
if (generatedFixtureOnly) {
  assert.ok(generatedFixtureManifest, "--generated-fixture-only requires a generated manifest");
  validateGeneratedFixtureManifest(generatedFixtureManifest);
  process.exit(0);
}

const sourceRoots = ["rust/crates"] as const;
const authorityInventoryDecreaseReviews = [
  {
    previousSiteCount: 705,
    currentSiteCount: 699,
    disposition: "reviewed-structural-refactor",
    reason:
      "Authored-property carrier typing removes six redundant per-consumer constructor calls; those boundaries now carry AuthoredPropertyTextV0 or sealed keys and the raw-string identity count remains zero.",
  },
  {
    previousSiteCount: 706,
    currentSiteCount: 705,
    disposition: "reviewed-indexed-custom-property-resolution",
    reason:
      "The custom-property SCC evaluator now resolves dependencies through the canonical graph node index and materializes the ordered environment once, removing one intermediate canonical-name carrier while raw-string identity and identity-shaped residual counts remain zero.",
  },
  {
    previousSiteCount: 702,
    currentSiteCount: 694,
    disposition: "reviewed-structural-refactor",
    reason:
      "The indexed custom-property dependency graph replaces eight recursive canonical-name traversal carrier lines with integer node indices; raw-string identity and identity-shaped residual counts remain zero.",
  },
] as const;
const egressMeaningRevisionReviews = [
  {
    previousSiteCount: 2,
    currentSiteCount: 3,
    disposition: "reviewed-selector-authority-rekey",
    reason:
      "The selector identity report is now a presentation projection of the sealed CST-issued selector key; the single new egress is the compatibility wire id and raw matching sites were removed.",
  },
] as const;
const idiomMeaningRevisionReviews = [
  {
    disposition: "reviewed-sealed-selector-key-migration",
    reason:
      "The matcher signature and element join changed from authored String storage to sealed decoded selector keys; the witness-only String insertion remains presentation data, and the CST authority now verifies its compound key before issuing the semantic projection key.",
  },
] as const;
const lexemeGlobs = [
  "*_class_name",
  "*_class_names",
  "*_selector_name",
  "*_selector_names",
  "*_selector_universe",
  "required_class",
  "required_classes",
] as const;
const primitivePatterns = [
  ["str-eq", /(?:==|!=)/u],
  ["contains", /\.contains\s*\(/u],
  ["insert", /\.insert\s*\(/u],
  ["map-get", /\.get(?:_key_value)?\s*\(/u],
  ["cmp", /\.cmp\s*\(/u],
] as const satisfies readonly (readonly [PrimitiveId, RegExp])[];
const classBindingLexeme =
  /\b[A-Za-z0-9_]*(?:class_names?|selector_names?|selector_universe|required_class(?:es)?)[A-Za-z0-9_]*\b/u;
const mutatorCrates = [
  "omena-cascade",
  "omena-query",
  "omena-semantic",
  "omena-transform-passes",
  "omena-bridge",
  "omena-lsp-server",
  "omena-abstract-value",
  "omena-bundler",
] as const;
const egressExemptions: readonly ExemptionRule[] = [
  {
    path: "rust/crates/omena-semantic/src/selector_identity.rs",
    function: "from_canonical_key",
    operation: "CanonicalClassKeyV0::as_str",
    evidence: "let local_name = key.as_str().to_string();",
    reason:
      "The compatibility report carrier projects its display id from the sealed selector authority key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "class_names_in_selector",
    operation: "ClassNameV0::into_raw",
    evidence: "let name = entry.name.into_raw();",
    reason:
      "The parser product preserves authored selector spelling together with its source span.",
  },
  {
    path: "rust/crates/omena-query/src/style/cascade_checker/runtime_state.rs",
    function: "query_selector_class_names",
    operation: "ClassNameV0::into_raw",
    evidence: "names.insert(entry.name.into_raw());",
    reason:
      "Cascade diagnostics preserve authored selector spelling after shared lexical extraction.",
  },
] as const;

const idiomExemptions: readonly ExemptionRule[] = [
  ...[
    ["rust/crates/omena-bundler/src/lib.rs", "&& self.class_names == other.class_names"],
    ["rust/crates/omena-lsp-server/src/state.rs", "&& self.selector_names == other.selector_names"],
    [
      "rust/crates/omena-parser/src/closed_world/contract.rs",
      "&& self.class_names == other.class_names",
    ],
    [
      "rust/crates/omena-parser/src/summaries.rs",
      "&& self.class_selector_names == other.class_selector_names",
    ],
    [
      "rust/crates/omena-parser/src/summaries.rs",
      "&& self.id_selector_names == other.id_selector_names",
    ],
    [
      "rust/crates/omena-parser/src/summaries.rs",
      "&& self.placeholder_selector_names == other.placeholder_selector_names",
    ],
    ["rust/crates/omena-query/src/types.rs", "&& self.selector_names == other.selector_names"],
    [
      "rust/crates/omena-query/src/types.rs",
      "&& self.class_selector_names == other.class_selector_names",
    ],
    [
      "rust/crates/omena-query/src/types.rs",
      "&& self.id_selector_names == other.id_selector_names",
    ],
    [
      "rust/crates/omena-query/src/types.rs",
      "&& self.placeholder_selector_names == other.placeholder_selector_names",
    ],
    [
      "rust/crates/omena-semantic/src/lib.rs",
      "&& self.class_selector_names == other.class_selector_names",
    ],
    [
      "rust/crates/omena-transform-passes/src/model.rs",
      "&& self.reachable_class_names == other.reachable_class_names",
    ],
    [
      "rust/crates/omena-transform-passes/src/model.rs",
      "&& self.class_name_rewrites == other.class_name_rewrites",
    ],
  ].map(([path, evidence]) => ({
    path,
    function: "eq",
    operation: "str-eq" as const,
    evidence,
    reason:
      "Manual property-carrier equality preserves this non-property selector/class field's prior structural comparison; property identity is delegated separately to sealed property keys.",
    disposition: "sanctioned" as const,
  })),
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "eq",
    operation: "str-eq",
    evidence: "self.kind == other.kind",
    reason:
      "The compared static tag selects the hover-candidate wire variant; property identity is delegated separately to the sealed custom-property key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "eq",
    operation: "str-eq",
    evidence: "&& self.selector_class_names == other.selector_class_names",
    reason:
      "The compared vector is selector-class presentation evidence and is outside the property-name identity plane.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-syntax/src/selector.rs",
    function: "canonical_class_key_for_source_span",
    operation: "contains",
    evidence: "&& compound.required_classes.contains(&key)",
    reason:
      "The sole selector authority verifies that a parser definition span names a class key carried by the same CST compound before issuing that key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-cascade/src/selector.rs",
    function: "selector_match_branch_witness",
    operation: "insert",
    evidence: ".insert(required_class.as_str().to_string());",
    reason:
      "A failed sealed-key match is converted to authored-facing witness text only after the identity decision.",
    disposition: "named-exempt",
  },
  {
    path: "rust/crates/omena-cascade/src/selector.rs",
    function: "parse_simple_selector_signature_inner",
    operation: "insert",
    evidence: "required_classes.insert(ClassNameV0::new(name).canonical_key());",
    reason:
      "Selector signatures store the shared decoder's sealed class key rather than authored spelling.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-cascade-proof/src/proof_kernel.rs",
    function: "module_export_observation_map_v0",
    operation: "str-eq",
    evidence: "|| canonical_class_name.decoded() != observation.key.canonical_class_name",
    reason:
      "The proof kernel compares a certificate identity with the spelling decoded by the shared syntax authority.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-bundler/src/lib.rs",
    function: "instance_reachability_inputs_closed_over_composes",
    operation: "str-eq",
    evidence: "changed |= target.class_names.len() != before;",
    reason: "This compares set cardinalities to detect fixed-point convergence, not identifiers.",
  },
  {
    path: "rust/crates/omena-parser/src/facts/selectors.rs",
    function: "resolve_selector_group",
    operation: "str-eq",
    evidence: "&& class_names.len() == 1",
    reason: "This is a cardinality check, not class-name equality.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/code_actions.rs",
    function: "resolve_lsp_code_actions",
    operation: "map-get",
    evidence: 'let selector_name = payload.get("selectorName").and_then(Value::as_str)?;',
    reason: "This reads a JSON field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-query-core/src/lib.rs",
    function: "selector_universe_for_targets",
    operation: "map-get",
    evidence: "if let Some(selector_names) = style_selectors_by_path.get(target_style_path) {",
    reason: "This lookup is keyed by a normalized style path.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_local_composes: local_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_imported_composes: imported_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_global_composes: global_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/omena-semantic/src/css_modules_cross_file.rs",
    function: "collect_css_modules_composes_adjacency",
    operation: "str-eq",
    evidence: "let target_class_names = if target_style_path == *style_path {",
    reason: "This compares normalized style paths before selecting a class-name set.",
  },
  {
    path: "rust/crates/omena-semantic/src/source_evidence.rs",
    function: "selector_certainty_reason",
    operation: "str-eq",
    evidence: "if payload.selector_names.len() == 1 {",
    reason: "This is a cardinality check, not selector-name equality.",
  },
  {
    path: "rust/crates/omena-bridge/src/source_syntax.rs",
    function: "retire_template_prefix_reference",
    operation: "str-eq",
    evidence: "|| reference.selector_name.as_deref() != Some(prefix)",
    reason: "This retires an exact template prefix rather than joining complete class names.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "validate_authored_liveness_expectations_v0",
    operation: "contains",
    evidence: ".contains(expectation.class_name);",
    reason: "The differential harness compares authored expectation data with measured output.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "validate_authored_liveness_expectations_v0",
    operation: "contains",
    evidence: ".is_some_and(|names| names.contains(expectation.class_name));",
    reason: "The differential harness compares authored expectation data with measured output.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "rule_declaration_counts_for_class_v0",
    operation: "str-eq",
    evidence: "&& selector.name == class_name",
    reason: "The differential harness compares two already-emitted selector spellings.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_document_cache.rs",
    function: "class_value_universes_from_value",
    operation: "map-get",
    evidence: 'class_names: strings_from_value(universe.get("classNames")?)?,',
    reason: "This reads a serialized field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_document_cache.rs",
    function: "source_elements_from_value",
    operation: "map-get",
    evidence: 'static_class_names: match fact.get("staticClassNames") {',
    reason: "This reads a serialized field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_type_facts.rs",
    function: "take_type_fact_prefix_references",
    operation: "str-eq",
    evidence: "&& reference.selector_name.as_deref() == Some(prefix)",
    reason: "This consumes an exact prefix reference rather than joining complete class names.",
  },
  {
    path: "rust/crates/omena-parser/src/closed_world/contract.rs",
    function: "composes_origin_symbol_is_reachable",
    operation: "str-eq",
    evidence: ".is_some_and(|class_names| class_names.iter().any(|name| name == symbol)),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_local_composes: local_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_imported_composes: imported_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_global_composes: global_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "summarize_omena_query_css_modules_interface_bundle_inner",
    operation: "insert",
    evidence: "emitted_class_names.insert(",
    reason:
      "The interface bundle indexes each emitted token by module path and exact authored spelling because distinct raw spellings have distinct module-scoped token preimages.",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "str-eq",
    evidence: "let target_class_names = if target_style_path == *style_path {",
    reason: "This compares normalized style paths before selecting a class-name set.",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "contains",
    evidence: "if !class_names.contains(&canonical_class_key(owner_selector_name)) {",
    reason: "Both operands are normalized through the shared canonical class-name key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "contains",
    evidence: "if !target_class_names.contains(&canonical_class_key(target_selector_name)) {",
    reason: "Both operands are normalized through the shared canonical class-name key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts",
    operation: "insert",
    evidence: "if seen.insert((range_span.start, range_span.end, selector_name.clone())) {",
    reason: "This tuple insertion deduplicates parser facts by span and authored spelling.",
  },
  {
    path: "rust/crates/omena-query/src/style/transform.rs",
    function: "closed_world_bound_reachability_precision",
    operation: "contains",
    evidence: ".any(|name| !closed_world_class_names.contains(name.as_str()))",
    reason: "The closed-world carrier and query projection share the canonical class-name domain.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style/transform/css_modules.rs",
    function: "derive_class_name_rewrites_for_module_instance",
    operation: "contains",
    evidence: "if !unique_class_names.contains(name) {",
    reason:
      "Token preimages preserve exact authored spelling so two spellings that decode alike can still receive distinct module-scoped rewrites.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "flat_class_names_for_style_paths",
    operation: "contains",
    evidence: ".filter(|(_, names)| names.contains(class_name))",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "from_style_paths",
    operation: "contains",
    evidence: "if directly_attributed_class_names.contains(class_name) {",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "from_style_paths",
    operation: "contains",
    evidence: ".filter(|(_, names)| names.contains(class_name))",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/runtime/executor.rs",
    function: "module_export_observations_from_emitted_css_v0",
    operation: "contains",
    evidence: ".find(|(canonical, _)| observed_class_names.contains(*canonical))",
    reason:
      "Both operands are decoded through the shared syntax authority before emitted export observations are admitted.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-transform-passes/src/runtime/executor.rs",
    function: "run_hash_css_module_class_names_structural",
    operation: "str-eq",
    evidence: "if mutation_count == 0 && class_name_rewrites.is_empty() {",
    reason: "This checks whether transform outputs are empty rather than comparing identifiers.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/domains/css_modules_classes.rs",
    function: "rewritten_class_name_for",
    operation: "str-eq",
    evidence: "(original_name == class_name).then_some(rewritten_name)",
    reason:
      "Exact authored spelling selects its distinct module-scoped rewrite before the canonical CSS identifier fallback is consulted.",
  },
] as const;

const productPathMatrix = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/omena-product-path-matrix.json"), "utf8"),
) as ProductPathMatrix;
assert.equal(productPathMatrix.schemaVersion, "0", "product-path matrix schemaVersion");
assert.equal(productPathMatrix.product, "omena-css.product-path-matrix", "product-path matrix");
const engineCrates = productPathMatrix.entries
  .filter((entry) => entry.role === "R1" || entry.role === "R2")
  .map((entry) => entry.crate)
  .toSorted();
assert.ok(engineCrates.length > 0, "product-path matrix must identify engine crates");

const productionSources = trackedProductionSources();
const scannedCrates = [
  ...new Set(productionSources.map((sourcePath) => sourcePath.split("/")[2])),
].toSorted();
assert.ok(scannedCrates.length > 0, "identifier census source scope must be non-empty");
for (const crateName of engineCrates) {
  assert.ok(scannedCrates.includes(crateName), `engine crate is outside scan scope: ${crateName}`);
}

const existing = readExistingCensus();
const typeSeal = inspectTypeSeal();
const egressDiscovered = discoverEgressSites();
const idiomDiscovered = discoverIdiomSites();
const predicateDiscovered = discoverPredicateCopies();
const propertyTypeSeal = inspectPropertyTypeSeal();
const propertyIdentitySites = discoverPropertyIdentitySites();
const residualSweepSources: MutableRustSource[] = productionSources.map((relativePath) => ({
  relativePath,
  source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
}));
applyResidualCarrierProbeMutations(residualSweepSources);
const residualRawPropertyCarrierDiscovered =
  discoverResidualRawPropertyCarrierSites(residualSweepSources);
assertResidualCarrierProbeOutcomes(residualRawPropertyCarrierDiscovered);
const residualRawPropertyCarrierSites = classifyResidualRawPropertyCarrierSites(
  residualRawPropertyCarrierDiscovered,
  existing?.propertyIdentity.residualRawCarrierSites,
);
const rawPropertyIdentitySites = discoverRawPropertyIdentitySites();
const authoredEscapeClosureAudit = discoverAuthoredEscapeClosureAudit();
const authoredEscapeIdentityViolations = authoredEscapeClosureAudit.identityFlows.filter(
  (flow) => !flow.sanctioned,
);
assertReferenceSanctionedEscapeSites(authoredEscapeClosureAudit.escapeSites);
process.stderr.write(`authoredEscapeSiteCount=${authoredEscapeClosureAudit.escapeSites.length}\n`);
process.stderr.write(
  `authoredEscapeIdentityFlowCount=${authoredEscapeClosureAudit.identityFlows.length}\n`,
);
process.stderr.write(
  `authoredEscapeIdentityViolationCount=${authoredEscapeIdentityViolations.length}\n`,
);
process.stderr.write(`writeIntoSiteCount=${authoredEscapeClosureAudit.writeIntoSites.length}\n`);
process.stderr.write(
  `authoredIdentityCarrierAuditCount=${authoredEscapeClosureAudit.carrierAudit.length}\n`,
);
assert.deepEqual(
  authoredEscapeIdentityViolations,
  [],
  "authored-bearing escape result reached a property identity operation",
);
assert.equal(
  authoredEscapeClosureAudit.identityDeriveViolationCount,
  0,
  `authored-bearing carrier regained derived identity: ${authoredEscapeClosureAudit.identityDeriveViolations.join(
    "; ",
  )}`,
);
const unresolvedWriteIntoSites = authoredEscapeClosureAudit.writeIntoSites.filter(
  (site) => site.sinkClass === "unresolved",
);
assert.deepEqual(
  unresolvedWriteIntoSites,
  [],
  `write_into escape reached an unresolved sink: ${unresolvedWriteIntoSites
    .map((site) => `${site.path}:${site.line}:${site.function}`)
    .join(", ")}`,
);
const egressSites = classifySites(
  egressDiscovered,
  existing?.egress.sites,
  egressExemptions,
  "egress",
);
const idiomSites = classifySites(idiomDiscovered, existing?.idiom.sites, idiomExemptions, "idiom");
const predicateSites = classifyPredicateSites(predicateDiscovered, existing?.predicateCopies.sites);

const unclassified = [
  ...egressSites,
  ...idiomSites,
  ...predicateSites,
  ...residualRawPropertyCarrierSites,
].filter((site) => site.disposition === "unclassified");
if (rawPropertyIdentitySites.length > 0) {
  process.stderr.write(`rawPropertyIdentitySiteCount=${rawPropertyIdentitySites.length}\n`);
}
if (injectPropertyAuthorityDecreaseLaundering) {
  process.stderr.write(`propertyAuthoritySiteCount=${propertyIdentitySites.length}\n`);
}
assert.deepEqual(
  unclassified,
  [],
  "identifier authority census found unclassified sites; route or exempt each site explicitly",
);

const classifiedEgressSites = egressSites as CensusSite[];
const classifiedIdiomSites = idiomSites as CensusSite[];
const classifiedPredicateSites = predicateSites as CensusSite[];
const classifiedResidualRawPropertyCarrierSites = residualRawPropertyCarrierSites as CensusSite[];
const residualCarrierConsumerRows = discoverResidualPropertyCarrierConsumers(
  classifiedResidualRawPropertyCarrierSites,
  residualSweepSources,
);
const residualCarrierConsumerSites = residualCarrierConsumerRows.flatMap((row) => row.consumers);
const residualIdentityShapedConsumerCount = residualCarrierConsumerRows
  .filter((row) =>
    ["entry-parameter", "static-standard-literal"].includes(row.carrier.residualClass ?? ""),
  )
  .flatMap((row) => row.consumers)
  .filter((site) => site.classification === "identity-shaped").length;
if (acceptInventoryChange) {
  assert.equal(
    rawPropertyIdentitySites.length,
    0,
    "--accept-inventory-change requires rawPropertyIdentitySiteCount=0",
  );
  process.stderr.write(
    "inventoryChangeAcceptance=enabled; rawPropertyIdentitySiteCount=0; review-required=true\n",
  );
}
if (residualIdentityShapedConsumerCount > 0) {
  process.stderr.write(
    `residualIdentityShapedConsumerCount=${residualIdentityShapedConsumerCount}\n`,
  );
  process.stderr.write(
    `residualIdentityShapedConsumers=${JSON.stringify(
      residualCarrierConsumerRows
        .filter((row) => row.consumers.some((site) => site.classification === "identity-shaped"))
        .map((row) => ({
          carrier: row.carrier.operation,
          path: row.carrier.path,
          consumers: row.consumers.filter((site) => site.classification === "identity-shaped"),
        })),
    )}\n`,
  );
}
assert.deepEqual(
  rawPropertyIdentitySites,
  [],
  "property identity census found raw-string identity sites; convert each boundary to PropertyNameV0 or a sealed canonical key",
);
assert.equal(
  residualIdentityShapedConsumerCount,
  0,
  "residual raw-property carriers have identity-shaped consumers; add a sealed key to each carrier and migrate those consumers",
);
const egressMeaningRevision = existing
  ? egressMeaningRevisionReviews.find(
      (review) =>
        review.previousSiteCount === existing.egress.currentSiteCount &&
        review.currentSiteCount === classifiedEgressSites.length,
    )
  : undefined;
const baselineEgressSiteCount = egressMeaningRevision
  ? egressMeaningRevision.currentSiteCount
  : (existing?.egress.baselineSiteCount ?? classifiedEgressSites.length);
assert.ok(
  classifiedEgressSites.length <= baselineEgressSiteCount,
  `identifier egress count increased: baseline=${baselineEgressSiteCount} current=${classifiedEgressSites.length}`,
);

if (existing && writeMode) {
  if (propertyIdentitySites.length < existing.propertyIdentity.authoritySiteCount) {
    assert.ok(
      authorityInventoryDecreaseReviews.some(
        (review) =>
          review.previousSiteCount === existing.propertyIdentity.authoritySiteCount &&
          review.currentSiteCount === propertyIdentitySites.length,
      ),
      `authority inventory decrease cannot be adopted by --write: previous=${existing.propertyIdentity.authoritySiteCount} current=${propertyIdentitySites.length}`,
    );
  }
  if (egressMeaningRevision) {
    const previousKeys = new Set(existing.egress.sites.map(stableSiteKey));
    const additions = classifiedEgressSites.filter(
      (site) => !previousKeys.has(stableSiteKey(site)),
    );
    assert.deepEqual(
      additions.map((site) => [site.path, site.function, site.operation]),
      [
        [
          "rust/crates/omena-semantic/src/selector_identity.rs",
          "from_canonical_key",
          "CanonicalClassKeyV0::as_str",
        ],
      ],
      "the reviewed selector-authority meaning revision permits only the report projection egress",
    );
  } else {
    assertNoAddedSites(existing.egress.sites, classifiedEgressSites, "identifier egress");
  }
  const previousIdiomSites = existing.idiom.sites;
  const previousIdiomKeys = new Set(previousIdiomSites.map(stableSiteKey));
  const idiomAdditions = classifiedIdiomSites.filter(
    (site) => !previousIdiomKeys.has(stableSiteKey(site)),
  );
  if (idiomAdditions.length > 0) {
    const reviewedIdiomAdditionKeys = [
      ["rust/crates/omena-bundler/src/lib.rs", "eq", "str-eq"],
      ["rust/crates/omena-lsp-server/src/state.rs", "eq", "str-eq"],
      ["rust/crates/omena-parser/src/closed_world/contract.rs", "eq", "str-eq"],
      ["rust/crates/omena-parser/src/summaries.rs", "eq", "str-eq"],
      ["rust/crates/omena-query/src/types.rs", "eq", "str-eq"],
      ["rust/crates/omena-semantic/src/lib.rs", "eq", "str-eq"],
      ["rust/crates/omena-transform-passes/src/model.rs", "eq", "str-eq"],
      ["rust/crates/omena-cascade/src/selector.rs", "selector_match_branch_witness", "insert"],
      [
        "rust/crates/omena-cascade/src/selector.rs",
        "parse_simple_selector_signature_inner",
        "insert",
      ],
      [
        "rust/crates/omena-syntax/src/selector.rs",
        "canonical_class_key_for_source_span",
        "contains",
      ],
    ].map((key) => key.join("\0"));
    const reviewedIdiomAdditionKeySet = new Set(reviewedIdiomAdditionKeys);
    const actualIdiomAdditionKeys = idiomAdditions.map((site) =>
      [site.path, site.function, site.operation].join("\0"),
    );
    assert.deepEqual(
      actualIdiomAdditionKeys.filter((key) => !reviewedIdiomAdditionKeySet.has(key)),
      [],
      "the reviewed selector-key meaning revision permits only matcher storage, witness, CST authority issuance, and the non-property hover variant tag rows",
    );
  } else {
    assertNoAddedSites(existing.idiom.sites, classifiedIdiomSites, "identifier idiom");
  }
  assertNoAddedSites(
    existing.predicateCopies.sites,
    classifiedPredicateSites,
    "identifier predicate copy",
  );
  const residualClassInventoryTerms = [
    "entry-parameter",
    "static-standard-literal",
    "canonical-text-carrier",
    "non-property",
  ] as const;
  for (const residualClass of residualClassInventoryTerms) {
    const previousCount = existing.propertyIdentity.residualClassCounts?.[residualClass] ?? 0;
    const currentCount = classifiedResidualRawPropertyCarrierSites.filter(
      (site) => site.residualClass === residualClass,
    ).length;
    assert.ok(
      previousCount === currentCount || acceptInventoryChange,
      `residual class inventory changed for ${residualClass}; rerun only after review with --accept-inventory-change`,
    );
  }
  const previousEscapeClosure = existing.propertyIdentity.authoredEscapeClosure;
  if (previousEscapeClosure) {
    const sanctionedIdentityCount = authoredEscapeClosureAudit.identityFlows.filter(
      (flow) => flow.sanctioned,
    ).length;
    assert.ok(
      previousEscapeClosure.sanctionedIdentityCount === sanctionedIdentityCount ||
        acceptInventoryChange,
      "sanctioned escape inventory changed; rerun only after review with --accept-inventory-change",
    );
    assert.ok(
      JSON.stringify(previousEscapeClosure.externalLeafTypes) ===
        JSON.stringify(externalLeafTypes) || acceptInventoryChange,
      "external leaf type inventory changed; rerun only after review with --accept-inventory-change",
    );
    assert.ok(
      digest(previousEscapeClosure.writeIntoSites) ===
        digest(authoredEscapeClosureAudit.writeIntoSites) || acceptInventoryChange,
      "write_into site inventory changed; rerun only after review with --accept-inventory-change",
    );
  }
  if (existing.propertyIdentity.residualRawCarrierSites) {
    const previousInventory = existing.propertyIdentity.residualRawCarrierSites
      .map(residualCarrierInventoryKey)
      .toSorted();
    const currentInventory = (residualRawPropertyCarrierSites as CensusSite[])
      .map(residualCarrierInventoryKey)
      .toSorted();
    assert.ok(
      JSON.stringify(previousInventory) === JSON.stringify(currentInventory) ||
        acceptInventoryChange,
      "raw property carrier inventory changed; rerun only after review with --accept-inventory-change",
    );
  }
}

const census: IdentifierAuthorityCensus = {
  schemaVersion: "0",
  product: "omena.identifier-authority.census",
  policy: {
    expectedSide: "committed-authored-table",
    addedSiteAdoption: "explicit-review-flag-required",
    owningCheck: "rust/omena-syntax-authority-raw-scan-census",
    packageScript: "check:rust-omena-syntax-authority-raw-scan-census",
  },
  sourceRoots,
  scope: {
    scannedCrates,
    scannedCratesDigest: digest(scannedCrates),
    engineCrates,
    engineCratesDigest: digest(engineCrates),
  },
  preflight: {
    productPin: "c18606086dc77101ad162c8d0ddb36ccae7e9d85",
    measuredSiteCount: 50,
    primitiveCounts: {
      "str-eq": 24,
      contains: 9,
      insert: 10,
      "map-get": 5,
      cmp: 2,
    },
  },
  typeSeal,
  egress: {
    direction: "decrease-only",
    meaningRevisionReviews: egressMeaningRevisionReviews,
    baselineSiteCount: baselineEgressSiteCount,
    currentSiteCount: classifiedEgressSites.length,
    sites: classifiedEgressSites,
    siteDigest: digest(classifiedEgressSites),
  },
  idiom: {
    granularity: "line",
    meaningRevisionReviews: idiomMeaningRevisionReviews,
    lexemeGlobs,
    primitiveIds: primitivePatterns.map(([id]) => id),
    unlabelledBindingBlindSpot:
      "Bindings without a class or selector label are intentionally invisible; typed carriers and API egresses are the backstops.",
    currentSiteCount: classifiedIdiomSites.length,
    primitiveCounts: Object.fromEntries(
      primitivePatterns.map(([id]) => [
        id,
        classifiedIdiomSites.filter((site) => site.operation === id).length,
      ]),
    ) as Record<PrimitiveId, number>,
    sanctionedSiteCount: classifiedIdiomSites.filter((site) => site.disposition === "sanctioned")
      .length,
    namedExemptSiteCount: classifiedIdiomSites.filter((site) => site.disposition === "named-exempt")
      .length,
    sites: classifiedIdiomSites,
    siteDigest: digest(classifiedIdiomSites),
  },
  predicateCopies: {
    derivation: "direct-character-predicate-body",
    blindSpots: [
      "Methods and functions whose character input is not the sole typed parameter are outside this syntactic arm.",
      "Character membership hidden behind a constant, helper call, or other indirection is outside this syntactic arm.",
    ],
    currentSiteCount: classifiedPredicateSites.length,
    sites: classifiedPredicateSites,
    siteDigest: digest(classifiedPredicateSites),
  },
  propertyIdentity: {
    derivation: "authority-token-and-property-key-operation-scan",
    rawSiteDerivation: "typed-property-operand-and-raw-key-dataflow-scan",
    typeSeal: propertyTypeSeal,
    authorityInventoryDecreaseReviews,
    authoritySiteCount: propertyIdentitySites.length,
    sites: propertyIdentitySites,
    siteDigest: digest(propertyIdentitySites),
    residualRawCarrierDirection: "decrease-only",
    residualRawCarrierSiteCount: classifiedResidualRawPropertyCarrierSites.length,
    residualRawCarrierSites: classifiedResidualRawPropertyCarrierSites,
    residualRawCarrierSiteDigest: digest(residualRawPropertyCarrierSites),
    residualClassCounts: Object.fromEntries(
      [
        "entry-parameter",
        "static-standard-literal",
        "fixture-crate",
        "canonical-text-carrier",
        "non-property",
      ].map((residualClass) => [
        residualClass,
        classifiedResidualRawPropertyCarrierSites.filter(
          (site) => site.residualClass === residualClass,
        ).length,
      ]),
    ) as Record<ResidualPropertyClass, number>,
    residualProvenanceCounts: Object.fromEntries(
      ["p-authored", "p-canonical", "p-non-property"].map((provenance) => [
        provenance,
        classifiedResidualRawPropertyCarrierSites.filter((site) => site.provenance === provenance)
          .length,
      ]),
    ) as Record<ResidualPropertyProvenance, number>,
    residualCarrierConsumerDerivation: "resolved-type-access-and-parameter-use-scan",
    residualCarrierConsumerRows,
    residualCarrierConsumerSiteCount: residualCarrierConsumerSites.length,
    residualIdentityShapedConsumerCount: 0,
    residualCarrierConsumerDigest: digest(residualCarrierConsumerRows),
    rawStringIdentitySiteCount: 0,
    rawStringIdentitySites: [],
    rawStringIdentitySiteDigest: digest([]),
    egressHonestyTable: {
      derivation: "final-head-byte-differential",
      orderBranchSurfaces: [],
      valueBranchSurfaces: [],
      zeroBranchSurfaces: [
        {
          surface: "omena facts --json",
          evidenceGate: "rust/omena-cli-json-output-census",
        },
        {
          surface: "style-hover-candidates",
          evidenceGate: "rust/omena-lsp-server/style-provider-parity",
        },
        {
          surface: "N-API JSON fixtures",
          evidenceGate: "contract/parity-v2-golden",
        },
        {
          surface: "WASM JSON fixtures",
          evidenceGate: "contract/parity-v2-golden",
        },
        {
          surface: "omena CLI text",
          evidenceGate: "rust/omena-cli-json-output-census",
        },
        {
          surface: "LSP wire payloads",
          evidenceGate: "rust/omena-lsp-server/style-provider-parity",
        },
      ],
    },
    authoredEscapeClosure: {
      derivation: "manifest-entry-points-plus-rust-dataflow-fixpoint",
      axes: {
        origins: authoredOriginAxis,
        comparisons: authoredComparisonAxis,
        positions: authoredPositionAxis,
        escapes: authoredEscapeAxis,
      },
      axisDigests: {
        origins: digest(authoredOriginAxis),
        comparisons: digest(authoredComparisonAxis),
        positions: digest(authoredPositionAxis),
        escapes: digest(authoredEscapeAxis),
      },
      axisBaselines: {
        origins: "sha256:7976976895e7c7c6e6041b58cc4738c1a85b321a9fdd5828489591dc89ada27d",
        comparisons: "sha256:4f0b68e1929f55090fddf2560e81fd292006d390ac1a5ec94c5ad32b3b26c19e",
        positions: "sha256:ffd26a47bdfef655e9e2cafa05f544deb48354f1153ab53ced4f88285008744e",
      },
      fullProductCellCount: 3900,
      escapeCoveringCellCount: 1800,
      pairFamilies: [
        "origin-comparison",
        "origin-position",
        "comparison-position",
        "escape-origin",
        "escape-comparison",
        "escape-position",
      ],
      externalLeafTypes,
      escapeSiteCount: authoredEscapeClosureAudit.escapeSites.length,
      escapeSites: authoredEscapeClosureAudit.escapeSites,
      escapeSiteDigest: digest(authoredEscapeClosureAudit.escapeSites),
      identityFlowCount: authoredEscapeClosureAudit.identityFlows.length,
      identityFlows: authoredEscapeClosureAudit.identityFlows,
      identityFlowDigest: digest(authoredEscapeClosureAudit.identityFlows),
      identityViolationCount: 0,
      sanctionedIdentityCount: authoredEscapeClosureAudit.identityFlows.filter(
        (flow) => flow.sanctioned,
      ).length,
      writeIntoSiteCount: authoredEscapeClosureAudit.writeIntoSites.length,
      writeIntoSites: authoredEscapeClosureAudit.writeIntoSites,
      writeIntoSiteDigest: digest(authoredEscapeClosureAudit.writeIntoSites),
      unresolvedCallEdges: authoredEscapeClosureAudit.unresolvedCallEdges,
      unresolvedCallEdgeCount: authoredEscapeClosureAudit.unresolvedCallEdges.length,
      unresolvedCallEdgeDigest: digest(authoredEscapeClosureAudit.unresolvedCallEdges),
      carrierAudit: authoredEscapeClosureAudit.carrierAudit,
      carrierAuditDigest: digest(authoredEscapeClosureAudit.carrierAudit),
      identityDeriveViolationCount: 0,
      cannotSee: authoredEscapeCannotSee(authoredEscapeClosureAudit.unresolvedCallEdges.length),
    },
  },
};

const expected = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  assert.ok(
    !injectClassNameEquality &&
      !injectEgress &&
      !injectLabelledComparison &&
      !injectUnlabelledComparison &&
      !injectPredicateCopy &&
      !injectPredicateCopyExplicit &&
      !injectPredicateCopyReversed &&
      !injectPropertyStructuralEquality &&
      !injectPropertyRoundtripEquality &&
      !injectPropertyRawMap &&
      !injectPropertyRawCanonicalization &&
      !injectPropertyFqnRawMap &&
      !injectPropertyValuesRawMap &&
      !injectPropertySameLineRawOperation &&
      !injectPropertyNewFileRawComparison &&
      !injectPropertyNewFileRawCanonicalization &&
      !injectPropertyTrimChain &&
      !injectPropertyContextRawOperations &&
      !injectPropertyCaseFold &&
      !injectPropertyDecodeNeuter &&
      !injectPropertyAuthorityDecreaseLaundering &&
      !injectAuthoredUppercaseTransform &&
      !injectAuthoredTrimMatchesTransform &&
      !injectAuthoredStripPrefixTransform &&
      !injectResidualKeyedCarrierWithoutJoin &&
      !injectResidualKeyedCarrierWithJoin &&
      !injectContainerPredicateRevert &&
      !injectCanonicalVectorCarrier &&
      !injectResidualEntryParameterInventory &&
      !injectResidualStaticLiteralInventory &&
      !injectResidualNonPropertyInventory &&
      !injectResidualIdentityConsumer &&
      !injectSanctionedEscapeInventory &&
      !injectWriteIntoInventory &&
      !injectExternalLeafInventory &&
      !injectAuthoredWrapperEscapeIdentity &&
      !injectAuthoredContainerEscapeIdentity &&
      !injectPropertyNameEscapeIdentity &&
      !injectDerivedCarrierIdentity &&
      !injectRawSelectorDefinitionSort &&
      !injectRawTransformNodeSort &&
      !generatedFixtureManifest,
    "test injection cannot be combined with --write",
  );
  writeFileSync(censusPath, expected);
  const formatResult = spawnSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(
    formatResult.status,
    0,
    `failed to format generated census: ${(formatResult.stderr ?? "").trim()}`,
  );
} else {
  assert.ok(
    existsSync(censusPath),
    "identifier authority census is missing; run the package update script",
  );
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "identifier authority census is stale; review new sites before regeneration",
  );
}

function residualCarrierInventoryKey(
  site: Pick<
    CensusSite,
    "path" | "function" | "operation" | "provenance" | "provenanceDerivation" | "residualClass"
  >,
): string {
  const siteIdentity = site.operation.startsWith("residual-property-field:")
    ? `${site.path}\u0000${site.operation}`
    : `${site.path}\u0000${site.function}\u0000${site.operation}`;
  return `${siteIdentity}\u0000${site.provenance ?? "missing"}\u0000${site.residualClass ?? "missing"}\u0000${site.provenanceDerivation ?? "missing"}`;
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      scannedCrateCount: census.scope.scannedCrates.length,
      engineCrateCount: census.scope.engineCrates.length,
      classNameDerives: census.typeSeal.classNameDerives,
      classNameEqualityDerives: census.typeSeal.classNameEqualityDerives,
      egressSiteCount: census.egress.currentSiteCount,
      idiomSiteCount: census.idiom.currentSiteCount,
      idiomSanctionedSiteCount: census.idiom.sanctionedSiteCount,
      idiomNamedExemptSiteCount: census.idiom.namedExemptSiteCount,
      predicateCopySiteCount: census.predicateCopies.currentSiteCount,
      propertyAuthoritySiteCount: census.propertyIdentity.authoritySiteCount,
      residualRawPropertyCarrierSiteCount: census.propertyIdentity.residualRawCarrierSiteCount,
      residualCarrierConsumerSiteCount: census.propertyIdentity.residualCarrierConsumerSiteCount,
      residualIdentityShapedConsumerCount:
        census.propertyIdentity.residualIdentityShapedConsumerCount,
      rawPropertyIdentitySiteCount: census.propertyIdentity.rawStringIdentitySiteCount,
    },
    null,
    2,
  )}\n`,
);

function inspectTypeSeal(): IdentifierAuthorityCensus["typeSeal"] {
  const relativePath = "rust/crates/omena-syntax/src/ident.rs" as const;
  let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (injectClassNameEquality) {
    source = source.replace(
      "#[derive(Debug, Clone)]\npub struct ClassNameV0",
      "#[derive(Debug, Clone, PartialEq)]\npub struct ClassNameV0",
    );
  }
  const classNameDerives = derivesForStruct(source, "ClassNameV0");
  const canonicalKeyDerives = derivesForStruct(source, "CanonicalClassKeyV0");
  assert.deepEqual(
    classNameDerives,
    ["Debug", "Clone"],
    "ClassNameV0 derives must not expose structural equality or ordering",
  );
  assert.deepEqual(
    canonicalKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalClassKeyV0 must remain the equality, ordering, and hash carrier",
  );
  assert.match(
    source,
    /pub struct CanonicalClassKeyV0\s*\(\s*String\s*,\s*CanonicalClassKeySealV0\s*\)\s*;/u,
    "canonical class key fields must remain private and sealed",
  );
  assert.match(
    source,
    /pub fn same_as\s*\(&self,\s*other:\s*&Self\)\s*->\s*bool/u,
    "ClassNameV0 equality must remain behind same_as",
  );
  assert.match(
    source,
    /pub fn canonical_key\s*\(self\)\s*->\s*CanonicalClassKeyV0/u,
    "ClassNameV0 key construction must remain behind canonical_key",
  );
  const canonicalKeyBody = source.match(
    /pub fn canonical_key\s*\(self\)\s*->\s*CanonicalClassKeyV0\s*\{(?<body>[\s\S]*?)\n\s{4}\}/u,
  )?.groups?.body;
  assert.ok(canonicalKeyBody, "canonical_key body must remain inspectable");
  assert.match(
    canonicalKeyBody,
    /self\.decoded\.unwrap_or\(self\.raw\)/u,
    "canonical class key must consume the decoded spelling or unchanged raw spelling",
  );
  const constructorMatches = [...canonicalKeyBody.matchAll(/CanonicalClassKeyV0\s*\(/gu)];
  assert.equal(
    constructorMatches.length,
    1,
    "canonical class key must have one authority-owned construction site",
  );
  for (const crateName of mutatorCrates) {
    assert.notEqual(crateName, "omena-syntax", "mutator crate must be outside authority crate");
    assert.ok(
      existsSync(path.join(repoRoot, "rust/crates", crateName, "Cargo.toml")),
      `measured mutator crate is missing: ${crateName}`,
    );
  }
  return {
    path: relativePath,
    classNameDerives: ["Debug", "Clone"],
    classNameEqualityDerives: [],
    canonicalKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    canonicalKeyFieldVisibility: "private",
    canonicalKeyConstructor: "ClassNameV0::canonical_key",
    equalityMethod: "ClassNameV0::same_as",
    mutatorCrates,
    privacyDiscriminator:
      "Rust field privacy is module-scoped, and every measured mutator is in a crate other than omena-syntax.",
  };
}

function inspectPropertyTypeSeal(): PropertyIdentityTypeSeal {
  const relativePath = "rust/crates/omena-syntax/src/ident.rs" as const;
  let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (injectPropertyCaseFold) {
    source = source.replace(
      /CanonicalCustomPropertyNameV0\(\s*decoded\.clone\(\),/u,
      "CanonicalCustomPropertyNameV0(decoded.to_ascii_lowercase(),",
    );
  }
  if (injectPropertyDecodeNeuter) {
    const decoderNeedle = "let decoded = decode_css_identifier_escapes(&authored).into_owned();";
    assert.equal(
      source.split(decoderNeedle).length - 1,
      2,
      "property decoder mutation must cover both explicit and inferred-kind constructors",
    );
    source = source.replaceAll(decoderNeedle, "let decoded = authored.clone();");
  }

  const propertyNameDerives = derivesForEnum(source, "PropertyNameV0");
  const authoredTextDerives = derivesForStruct(source, "AuthoredPropertyTextV0");
  const standardKeyDerives = derivesForStruct(source, "CanonicalStandardPropertyNameV0");
  const customKeyDerives = derivesForStruct(source, "CanonicalCustomPropertyNameV0");
  assert.deepEqual(
    authoredTextDerives,
    ["Debug", "Clone", "serde::Serialize", "serde::Deserialize"],
    "AuthoredPropertyTextV0 must remain presentation-only and structurally non-comparable",
  );
  assert.match(
    source,
    /pub struct AuthoredPropertyTextV0\s*\(\s*String\s*\)\s*;/u,
    "authored property text storage must remain private",
  );
  assert.doesNotMatch(
    source,
    /impl fmt::Display for AuthoredPropertyTextV0/u,
    "authored property text must not expose presentation through Display",
  );
  const authoredImplStart = source.indexOf("impl AuthoredPropertyTextV0");
  const authoredImplOpen = source.indexOf("{", authoredImplStart);
  const authoredImplClose = matchingBrace(source, authoredImplOpen);
  assert.ok(
    authoredImplStart >= 0 && authoredImplOpen >= 0 && authoredImplClose !== undefined,
    "AuthoredPropertyTextV0 impl must remain inspectable",
  );
  const authoredImpl = source.slice(authoredImplOpen + 1, authoredImplClose);
  assert.deepEqual(
    [...authoredImpl.matchAll(/\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)].map(
      (match) => match[1],
    ),
    ["new", "write_into", "is_empty", "to_property_name", "to_custom_key", "to_standard_key"],
    "AuthoredPropertyTextV0 must expose only construction, bounded presentation, and sealed projections",
  );
  assert.doesNotMatch(
    authoredImpl,
    /pub fn as_str\s*\(&self\)/u,
    "authored property text must not expose a borrowable raw string",
  );
  assert.match(
    authoredImpl,
    /pub fn write_into\s*\(&self,\s*out:\s*&mut impl fmt::Write\)\s*->\s*fmt::Result/u,
    "authored property text must expose bounded writer-based presentation",
  );
  assert.match(
    source,
    /pub fn render_authored\s*\(\s*authored:\s*&AuthoredPropertyTextV0,\s*output:\s*&mut String\s*\)\s*->\s*fmt::Result/u,
    "authored property text must expose exactly the named owned-buffer presentation helper",
  );
  const propertyImplStart = source.indexOf("impl PropertyNameV0");
  const propertyImplOpen = source.indexOf("{", propertyImplStart);
  const propertyImplClose = matchingBrace(source, propertyImplOpen);
  assert.ok(
    propertyImplStart >= 0 && propertyImplOpen >= 0 && propertyImplClose !== undefined,
    "PropertyNameV0 impl must remain inspectable",
  );
  const propertyImpl = source.slice(propertyImplOpen + 1, propertyImplClose);
  assert.doesNotMatch(
    propertyImpl,
    /pub fn authored\s*\(&self\)\s*->\s*&str/u,
    "PropertyNameV0 must not reintroduce the authored raw-string borrow",
  );
  assert.doesNotMatch(
    propertyImpl,
    /pub fn decoded\s*\(/u,
    "PropertyNameV0 must not expose decoded authored text",
  );
  assert.deepEqual(
    [...propertyImpl.matchAll(/\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)].map(
      (match) => match[1],
    ),
    [
      "from_authored",
      "new",
      "standard",
      "custom",
      "kind",
      "authored_text",
      "canonical_name",
      "same_as",
      "canonical_key",
      "as_custom_key",
      "as_standard_key",
      "canonical_custom_key",
      "canonical_standard_key",
    ],
    "PropertyNameV0 public method set must remain exact",
  );
  assert.match(
    source,
    /pub enum PropertyNameV0\s*\{\s*Standard\(StandardPropertyNamePayloadV0\),\s*Custom\(CustomPropertyNamePayloadV0\),\s*\}/su,
    "PropertyNameV0 variants must carry private payload types",
  );
  assert.match(
    source,
    /struct StandardPropertyNamePayloadV0\s*\{/u,
    "standard property payload must remain private",
  );
  assert.match(
    source,
    /struct CustomPropertyNamePayloadV0\s*\{/u,
    "custom property payload must remain private",
  );
  assert.doesNotMatch(
    source,
    /pub struct (?:Standard|Custom)PropertyNamePayloadV0\b/u,
    "property payload structs must not be public",
  );
  assert.deepEqual(
    propertyNameDerives,
    ["Debug", "Clone"],
    "PropertyNameV0 derives must forbid raw structural equality and ordering",
  );
  assert.deepEqual(
    standardKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalStandardPropertyNameV0 must remain a sealed equality carrier",
  );
  assert.deepEqual(
    customKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalCustomPropertyNameV0 must remain a sealed equality carrier",
  );
  assert.match(
    source,
    /pub struct CanonicalStandardPropertyNameV0\s*\(\s*String\s*,\s*CanonicalStandardPropertyNameSealV0\s*\)\s*;/u,
    "standard property key fields must remain private and sealed",
  );
  assert.match(
    source,
    /pub struct CanonicalCustomPropertyNameV0\s*\(\s*String\s*,\s*CanonicalCustomPropertyNameSealV0\s*\)\s*;/u,
    "custom property key fields must remain private and sealed",
  );
  assert.match(
    source,
    /let decoded = decode_css_identifier_escapes\(&authored\)\.into_owned\(\);/u,
    "property identity must share the CSS identifier escape decoder",
  );
  assert.match(
    source,
    /CanonicalStandardPropertyNameV0\(\s*decoded\.to_ascii_lowercase\(\),/u,
    "standard property canonicalization must remain ASCII-case-insensitive",
  );
  assert.match(
    source,
    /CanonicalCustomPropertyNameV0\(\s*decoded\.clone\(\),/u,
    "custom property canonicalization must preserve decoded case",
  );
  assert.match(
    source,
    /pub fn property_names_same\([^)]*\)[^{]*\{\s*PropertyNameV0::from_authored\(left\)\.same_as\(&PropertyNameV0::from_authored\(right\)\)/su,
    "property comparison helper must delegate to PropertyNameV0::same_as",
  );

  return {
    path: relativePath,
    authoredTextDerives: ["Debug", "Clone", "serde::Serialize", "serde::Deserialize"],
    authoredTextIdentityDerives: [],
    authoredTextFieldVisibility: "private",
    authoredTextPresentationMethods: ["write_into", "Serialize", "Deserialize"],
    propertyNameDerives: ["Debug", "Clone"],
    propertyNameEqualityDerives: [],
    propertyNameMethods: [
      "from_authored",
      "new",
      "standard",
      "custom",
      "kind",
      "authored_text",
      "canonical_name",
      "same_as",
      "canonical_key",
      "as_custom_key",
      "as_standard_key",
      "canonical_custom_key",
      "canonical_standard_key",
    ],
    standardKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    customKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    keyFieldVisibility: "private",
    sharedDecoder: "decode_css_identifier_escapes",
    standardCanonicalization: "trim-decode-ascii-lowercase",
    customCanonicalization: "trim-decode-case-preserved",
    equalityMethod: "PropertyNameV0::same_as",
  };
}

function discoverPropertyIdentitySites(): PropertyIdentitySite[] {
  const tokenPatterns = [
    ["PropertyNameV0::from_authored", /PropertyNameV0::from_authored\s*\(/u],
    ["PropertyNameV0::custom", /PropertyNameV0::custom\s*\(/u],
    ["PropertyNameV0::standard", /PropertyNameV0::standard\s*\(/u],
    ["PropertyNameV0::canonical_custom_key", /PropertyNameV0::canonical_custom_key\s*\(/u],
    ["PropertyNameV0::canonical_standard_key", /PropertyNameV0::canonical_standard_key\s*\(/u],
    [
      "PropertyNameV0::canonical_key",
      /\b(?:property|property_name|property_identity|canonical_property)\.canonical_key\s*\(\s*\)/u,
    ],
    ["sealed-property-key", /\b(?:[A-Za-z_][A-Za-z0-9_]*\.)?property_key\b/u],
    [
      "PropertyNameV0::same_as",
      /(?:\b(?:property|property_name|property_identity|canonical_property)\.same_as\s*\(|\.same_as\s*\(\s*&PropertyNameV0)/u,
    ],
    ["property_names_same", /\bproperty_names_same\s*\(/u],
    ["CanonicalPropertyKeyV0", /\bCanonicalPropertyKeyV0\b/u],
    ["CanonicalCustomPropertyNameV0", /\bCanonicalCustomPropertyNameV0\b/u],
    ["CanonicalStandardPropertyNameV0", /\bCanonicalStandardPropertyNameV0\b/u],
  ] as const;
  const sites: PropertyIdentitySite[] = [];
  for (const relativePath of productionSources) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (
      injectPropertyAuthorityDecreaseLaundering &&
      relativePath === "rust/crates/omena-lsp-server/src/lib.rs"
    ) {
      const authorityJoin = "target.property_key == candidate.property_key";
      assert.equal(
        source.split(authorityJoin).length - 1,
        1,
        "authority census decrease mutation must replace exactly one sealed join",
      );
      source = source.replace(
        authorityJoin,
        "target.name.to_string().eq_ignore_ascii_case(&candidate.name.to_string())",
      );
    }
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const sourceLines = source.split(/\r?\n/u);
    for (const [index, line] of scannable.split(/\r?\n/u).entries()) {
      const match = tokenPatterns.find(([, pattern]) => pattern.test(line));
      if (!match) continue;
      const evidence = sourceLines[index]?.trim().replace(/\s+/gu, " ") ?? "";
      const functionName = enclosingFunctionName(scannable, offsetForLine(scannable, index + 1));
      const operation = match[0];
      const role: PropertyIdentitySiteRole =
        relativePath === "rust/crates/omena-syntax/src/ident.rs"
          ? "authority"
          : /property_key\s*:|canonical_custom_key|canonical_key/u.test(evidence)
            ? "producer"
            : /property_names_same|same_as|\.get\s*\(|contains_key|\.entry\s*\(/u.test(evidence)
              ? "consumer"
              : /Canonical(?:Custom|Standard)?Property/u.test(evidence)
                ? "carrier"
                : "consumer";
      sites.push({
        path: relativePath,
        line: index + 1,
        function: functionName,
        operation,
        evidence,
        role,
      });
    }
  }
  return uniqueSites(sites);
}

function discoverRawPropertyIdentitySites(): DiscoveredSite[] {
  return uniqueSites([
    ...discoverTypedRawPropertyIdentitySites(),
    ...discoverAuthoredPropertyIdentityLaunderingSites(),
  ]);
}

function discoverAuthoredPropertyIdentityLaunderingSites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  sources.push(
    ...(generatedFixtureManifest?.sources.map((source) => ({
      relativePath: source.relativePath,
      source: source.source,
    })) ?? []),
  );
  applyResidualCarrierProbeMutations(sources);
  applyPropertyAuthorityDecreaseMutation(sources);
  const carrierFields = discoverAuthoredPropertyCarrierFields(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const authoredReturningFunctions = authoredReturningFunctionNames(sources, carrierFields);
  const functionResultKinds = rustFunctionResultKinds(sources);
  const sites: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const classifier = propertyOperandClassifier(
        functionSlice,
        carrierFields,
        false,
        aliasesByPath.get(relativePath),
        authoredReturningFunctions,
      );
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text !== "==" && token.text !== "!=") continue;
        const left = maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "left"));
        const right = maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "right"));
        if (
          !classifier.containsRawPropertyOrigin(left) &&
          !classifier.containsRawPropertyOrigin(right)
        )
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-identity-laundering",
          ),
        );
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          !/^(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)$/u.test(token.text) ||
          tokens[tokenIndex - 1]?.text !== "." ||
          tokens[tokenIndex + 1]?.text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 1, "(", ")");
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        const argumentTokens =
          closeParen === undefined ? [] : tokens.slice(tokenIndex + 2, closeParen);
        if (
          !classifier.containsRawPropertyOrigin(receiver) &&
          !classifier.containsRawPropertyOrigin(argumentTokens)
        ) {
          continue;
        }
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-identity-method-laundering",
          ),
        );
      }
      for (let tokenIndex = 0; tokenIndex + 3 < tokens.length; tokenIndex += 1) {
        const ufcsEquality =
          (tokens[tokenIndex].text === "str" || tokens[tokenIndex].text === "PartialEq") &&
          tokens[tokenIndex + 1]?.text === "::" &&
          tokens[tokenIndex + 2]?.text === "eq" &&
          tokens[tokenIndex + 3]?.text === "(";
        if (!ufcsEquality) continue;
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 3, "(", ")");
        if (closeParen === undefined) continue;
        if (!classifier.containsRawPropertyOrigin(tokens.slice(tokenIndex + 4, closeParen)))
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + tokens[tokenIndex + 2].start,
            "authored-property-ufcs-identity-laundering",
          ),
        );
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          token.text !== "to_ascii_lowercase" &&
          token.text !== "make_ascii_lowercase" &&
          token.text !== "to_lowercase" &&
          token.text !== "to_ascii_uppercase" &&
          token.text !== "make_ascii_uppercase" &&
          token.text !== "to_uppercase" &&
          token.text !== "trim_matches" &&
          token.text !== "strip_prefix"
        )
          continue;
        if (tokens[tokenIndex - 1]?.text !== "." || tokens[tokenIndex + 1]?.text !== "(") continue;
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        if (!classifier.containsRawPropertyOrigin(receiver)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-canonicalization-laundering",
          ),
        );
      }
      for (const collection of rawStringCollectionDeclarations(functionSlice)) {
        if (!classifier.collectionReceivesPropertyKey(collection.binding)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + collection.offset,
            "authored-property-map-laundering",
          ),
        );
        if (classifier.collectionUsesIdentityOperation(collection.binding)) {
          sites.push(
            siteAt(
              relativePath,
              source,
              scannable,
              functionSlice.bodyStart + collection.offset,
              "authored-property-collection-identity-laundering",
            ),
          );
        }
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text === "match") {
          const openBrace = tokens.findIndex(
            (candidate, index) => index > tokenIndex && candidate.text === "{",
          );
          if (
            openBrace > tokenIndex &&
            classifier.containsRawPropertyOrigin(
              maskNonRawResultCalls(tokens.slice(tokenIndex + 1, openBrace), functionResultKinds),
            )
          ) {
            sites.push(
              siteAt(
                relativePath,
                source,
                scannable,
                functionSlice.bodyStart + token.start,
                "authored-property-match-identity-laundering",
              ),
            );
          }
        }
        if (
          token.text === "matches" &&
          tokens[tokenIndex + 1]?.text === "!" &&
          tokens[tokenIndex + 2]?.text === "("
        ) {
          const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 2, "(", ")");
          if (closeParen === undefined) continue;
          const firstArgument = maskNonRawResultCalls(
            firstArgumentTokens(tokens.slice(tokenIndex + 3, closeParen)),
            functionResultKinds,
          );
          if (!classifier.containsRawPropertyOrigin(firstArgument)) continue;
          sites.push(
            siteAt(
              relativePath,
              source,
              scannable,
              functionSlice.bodyStart + token.start,
              "authored-property-matches-identity-laundering",
            ),
          );
        }
      }
    }
  }
  return uniqueSites(sites);
}

function discoverAuthoredPropertyCarrierFields(
  sources: readonly { source: string }[],
): ReadonlyMap<string, ReadonlySet<string>> {
  const fields = new Map<string, Set<string>>();
  for (const { source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of rustStructFields(body)) {
        if (!/\bAuthoredPropertyTextV0\b/u.test(field.rustType)) continue;
        const typeFields = fields.get(typeName) ?? new Set<string>();
        typeFields.add(field.name);
        fields.set(typeName, typeFields);
      }
    }
  }
  return fields;
}

function discoverResidualRawPropertyCarrierSites(
  sources: readonly MutableRustSource[],
): DiscoveredSite[] {
  const sites: DiscoveredSite[] = [];
  const fieldProvenance = deriveRawPropertyFieldClassifications(sources);
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of rustStructFields(body)) {
        const fieldName = field.name;
        const rustType = field.rustType.trim().replace(/\s+/gu, " ");
        if (!rawPropertyCarrierType(rustType)) continue;
        const classification = fieldProvenance.get(
          rawPropertyFieldKey(relativePath, typeName, fieldName),
        );
        if (!classification) continue;
        sites.push({
          ...siteAt(
            relativePath,
            source,
            scannable,
            openBrace + 1 + field.offset,
            `residual-property-field:${typeName}.${fieldName}:${rustType}`,
          ),
          function: "<module>",
          ...classification,
        });
      }
    }

    for (const functionSlice of rustFunctionSlices(scannable)) {
      for (const parameter of functionSlice.signature.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str|String)\b/gu,
      )) {
        const parameterName = parameter[1];
        if (
          !isPropertySemanticIdentifier(parameterName) &&
          !(
            functionSlice.propertyContext &&
            /^(?:name|left|right|actual|expected|target|candidate|declared)$/u.test(parameterName)
          )
        ) {
          continue;
        }
        sites.push({
          ...siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart - functionSlice.signature.length + parameter.index,
            `residual-property-parameter:${functionSlice.name}.${parameterName}:${parameter[2].replace(/\s+/gu, " ")}`,
          ),
          ...classifyResidualPropertyParameter(relativePath, parameter[2]),
        });
      }
    }
  }
  return uniqueSites(sites);
}

function rawPropertyCarrierMember(typeName: string, fieldName: string, _source?: string): boolean {
  if (isPropertySemanticIdentifier(fieldName)) return true;
  if (
    isPropertySemanticIdentifier(typeName) &&
    /^(?:name|names|text|key|keys|first|second|left|right|actual|expected|target|candidate|declared)$/u.test(
      fieldName,
    )
  ) {
    return true;
  }
  return false;
}

interface ResidualPropertyClassification {
  readonly provenance: ResidualPropertyProvenance;
  readonly provenanceDerivation: string;
  readonly residualClass?: ResidualPropertyClass;
}

function deriveRawPropertyFieldClassifications(
  sources: readonly { relativePath: string; source: string }[],
): ReadonlyMap<string, ResidualPropertyClassification> {
  const classifications = new Map<string, ResidualPropertyClassification>();
  const initializerExpressions = rustStructFieldInitializerExpressionIndex(sources);
  const workspaceStaticSmugglingSiteCount = sources.filter(({ source }) =>
    /(?:Box\s*::\s*leak|\.leak\s*\()/u.test(maskCommentsStringsAndTestItems(source, false)),
  ).length;
  for (const declaration of rawRustStructFieldDeclarations(sources)) {
    const { relativePath, typeName, fieldName, rustType } = declaration;
    if (!rawPropertyCarrierType(rustType)) continue;
    const crateName = relativePath.split("/")[2] ?? "";
    const crateSources = sources.filter(
      (candidate) => candidate.relativePath.split("/")[2] === crateName,
    );
    const expressions =
      initializerExpressions.get(`${crateName}\u0000${typeName}\u0000${fieldName}`) ?? [];
    const expressionKinds = expressions.map(classifyRawPropertyExpression);
    const semanticCandidate = rawPropertyCarrierMember(typeName, fieldName);
    const fixtureCrate = fixturePropertyCrate(relativePath);
    const hasPropertyProvenance = expressionKinds.some(
      (kind) => kind === "p-authored" || kind === "p-canonical",
    );
    const allCanonicalOrNeutral =
      expressionKinds.includes("p-canonical") &&
      expressionKinds.every(
        (kind, index) =>
          kind === "p-canonical" ||
          (kind === undefined && neutralCanonicalTextExpression(expressions[index] ?? "")),
      );
    if (!semanticCandidate && !hasPropertyProvenance) continue;
    const consumerDerivation =
      semanticCandidate && !hasPropertyProvenance
        ? (rawPropertyNonPropertyNameDerivation(fieldName) ??
          rawPropertyNonPropertyConsumerDerivation(crateSources, typeName, fieldName))
        : undefined;

    let provenance: ResidualPropertyProvenance;
    let provenanceDerivation: string;
    let residualClass: ResidualPropertyClass | undefined;
    if (fixtureCrate !== undefined) {
      provenance = expressionKinds.includes("p-canonical") ? "p-canonical" : "p-authored";
      provenanceDerivation = `fixture-crate:${fixtureCrate};initializer-kinds:${renderProvenanceKinds(expressionKinds)}`;
      residualClass = "fixture-crate";
    } else if (
      semanticCandidate &&
      staticStringType(rustType) &&
      workspaceStaticSmugglingSiteCount === 0 &&
      expressions.every((expression) => !/\b(?:include_str|include_bytes)\s*!/u.test(expression))
    ) {
      provenance = "p-canonical";
      provenanceDerivation = `static-type;direct-literal-initializers:${expressions.filter(rustStringLiteralExpression).length};workspace-static-smuggling-sites:0`;
      residualClass = "static-standard-literal";
    } else if (consumerDerivation !== undefined && !hasPropertyProvenance) {
      provenance = "p-non-property";
      provenanceDerivation = consumerDerivation;
      residualClass = "non-property";
    } else if (expressions.length > 0 && expressions.every(rustStringLiteralExpression)) {
      provenance = "p-canonical";
      provenanceDerivation = `literal-static-table-initializers:${expressions.length}`;
      residualClass = "canonical-text-carrier";
    } else if (allCanonicalOrNeutral) {
      provenance = "p-canonical";
      provenanceDerivation = `canonical-initializers:${expressions.length}`;
      residualClass = "canonical-text-carrier";
    } else {
      provenance = "p-authored";
      provenanceDerivation =
        expressionKinds.length === 0
          ? "unresolved-field-origin-defaults-to-p-authored"
          : `initializer-kinds:${renderProvenanceKinds(expressionKinds)};unresolved-default:p-authored`;
    }
    classifications.set(rawPropertyFieldKey(relativePath, typeName, fieldName), {
      provenance,
      provenanceDerivation,
      residualClass,
    });
  }
  return classifications;
}

function rawRustStructFieldDeclarations(
  sources: readonly { relativePath: string; source: string }[],
): readonly {
  readonly relativePath: string;
  readonly typeName: string;
  readonly fieldName: string;
  readonly rustType: string;
}[] {
  const declarations: {
    relativePath: string;
    typeName: string;
    fieldName: string;
    rustType: string;
  }[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of rustStructFields(body)) {
        declarations.push({
          relativePath,
          typeName,
          fieldName: field.name,
          rustType: field.rustType.trim().replace(/\s+/gu, " "),
        });
      }
    }
  }
  return declarations;
}

function rustStructFieldInitializerExpressionIndex(
  sources: readonly { relativePath: string; source: string }[],
): ReadonlyMap<string, readonly string[]> {
  const expressions = new Map<string, string[]>();
  for (const { relativePath, source } of sources) {
    const crateName = relativePath.split("/")[2] ?? "";
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\b([A-Z][A-Za-z0-9_]*)\s*\{/gu)) {
      const prefix = scannable.slice(Math.max(0, match.index - 16), match.index);
      if (/\b(?:struct|enum)\s*$/u.test(prefix)) continue;
      const typeName = match[1];
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const scannableBody = scannable.slice(openBrace + 1, closeBrace);
      const sourceBody = source.slice(openBrace + 1, closeBrace);
      for (const segment of topLevelSegments(scannableBody, ",")) {
        const assignment = segment.text.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:/u);
        if (!assignment) continue;
        const key = `${crateName}\u0000${typeName}\u0000${assignment[1]}`;
        const fieldExpressions = expressions.get(key) ?? [];
        fieldExpressions.push(
          sourceBody
            .slice(segment.offset + assignment[0].length, segment.offset + segment.text.length)
            .trim(),
        );
        expressions.set(key, fieldExpressions);
      }
    }
  }
  return expressions;
}

function classifyRawPropertyExpression(expression: string): ResidualPropertyProvenance | undefined {
  const scannable = maskCommentsStringsAndTestItems(expression, false);
  if (
    /\b(?:canonical_name|canonical_key|canonical_custom_key|canonical_standard_key|to_custom_key|to_standard_key|as_custom_key|as_standard_key)\b/u.test(
      scannable,
    ) &&
    /\b(?:PropertyNameV0|Canonical(?:Custom|Standard)?Property(?:Name)?V0|property|properties|custom_property|longhand|longhands|shorthand)\b/u.test(
      scannable,
    )
  ) {
    return "p-canonical";
  }
  if (
    !/\b(?:format|format_args|concat|join)\s*!?\s*\(/u.test(scannable) &&
    (/(?:^|\.)authored_text\s*\(/u.test(scannable) ||
      /\bAuthoredPropertyTextV0\b/u.test(scannable) ||
      /^\s*(?:Some\s*\(\s*)?&?(?:[A-Za-z_][A-Za-z0-9_]*\.)*(?:property|property_name|custom_property_name)(?:\.(?:clone|to_owned|to_string|as_str)\s*\(\s*\))*\s*\)?\s*$/u.test(
        scannable,
      ))
  ) {
    return "p-authored";
  }
  if (/\b(?:declaration_?id|winner\.id|digest|hash|selector|class_name|value)\b/u.test(scannable)) {
    return "p-non-property";
  }
  return undefined;
}

function neutralCanonicalTextExpression(expression: string): boolean {
  const scannable = maskCommentsStringsAndTestItems(expression, false);
  return /^\s*(?:[A-Za-z_][A-Za-z0-9_]*\.)*[A-Za-z_][A-Za-z0-9_]*\.to_ascii_lowercase\s*\(\s*\)\s*$/u.test(
    scannable,
  );
}

function rawPropertyNonPropertyConsumerDerivation(
  sources: readonly { relativePath: string; source: string }[],
  typeName: string,
  fieldName: string,
): string | undefined {
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const bindings = [
      ...scannable.matchAll(
        new RegExp(
          String.raw`\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*(?:mut\s+)?${escapeRegExp(typeName)}\b`,
          "gu",
        ),
      ),
    ].map((match) => match[1]);
    for (const binding of bindings) {
      const access = new RegExp(
        String.raw`\b${escapeRegExp(binding)}\.${escapeRegExp(fieldName)}\s*\.\s*(?:get|contains_key|remove)\s*\(\s*&?\s*([^),\n]+)`,
        "gu",
      );
      for (const match of scannable.matchAll(access)) {
        const argument = match[1].trim();
        if (
          /\b(?:id|declaration_id|digest|hash|selector|class_name|value)\b/u.test(argument) &&
          !isPropertySemanticIdentifier(argument)
        ) {
          return `non-property-key-consumer:${relativePath}:${lineNumberAt(source, match.index)}:${argument}`;
        }
      }
    }
  }
  return undefined;
}

function rawPropertyNonPropertyNameDerivation(fieldName: string): string | undefined {
  const words = semanticIdentifierWords(fieldName);
  const discriminator = words.find((word) =>
    [
      "access",
      "diagnostic",
      "message",
      "product",
      "provenance",
      "reason",
      "selector",
      "selectors",
      "status",
      "value",
      "values",
      "verdict",
      "verdicts",
    ].includes(word),
  );
  return discriminator === undefined
    ? undefined
    : `field-semantic-non-property-domain:${discriminator}`;
}

function staticStringType(typeSource: string): boolean {
  return /^&\s*'static\s+str\b/u.test(typeSource.trim());
}

function rustStringLiteralExpression(expression: string): boolean {
  const value = expression.trim();
  return /^(?:b?r(?:#[#]*)?"[\s\S]*"[#]*|b?"(?:\\.|[^"\\])*"|String::from\s*\(\s*"(?:\\.|[^"\\])*"\s*\)|"(?:\\.|[^"\\])*"\.to_string\s*\(\s*\))$/u.test(
    value,
  );
}

function renderProvenanceKinds(kinds: readonly (ResidualPropertyProvenance | undefined)[]): string {
  return (
    kinds
      .map((kind) => kind ?? "unresolved")
      .toSorted()
      .join(",") || "none"
  );
}

function fixturePropertyCrate(relativePath: string): string | undefined {
  const crateName = relativePath.split("/")[2];
  if (crateName === undefined) return undefined;
  return ["engine-style-parser", "omena-diff-test", "omena-benchmarks", "examples"].includes(
    crateName,
  )
    ? crateName
    : undefined;
}

function rawPropertyFieldKey(relativePath: string, typeName: string, fieldName: string): string {
  return `${relativePath}\u0000${typeName}\u0000${fieldName}`;
}

function classifyResidualPropertyParameter(
  relativePath: string,
  rustType: string,
): ResidualPropertyClassification {
  const fixtureCrate = fixturePropertyCrate(relativePath);
  if (fixtureCrate !== undefined) {
    return {
      provenance: "p-authored",
      provenanceDerivation: `fixture-crate:${fixtureCrate};borrowed-or-owned-boundary-parameter`,
      residualClass: "fixture-crate",
    };
  }
  if (staticStringType(rustType)) {
    return {
      provenance: "p-canonical",
      provenanceDerivation: "static-borrowed-entry-parameter",
      residualClass: "static-standard-literal",
    };
  }
  return {
    provenance: "p-authored",
    provenanceDerivation: /^\s*&/u.test(rustType)
      ? "borrowed-entry-parameter;usage-validated-by-residual-consumer-scan"
      : "owned-authored-parameter-forbidden",
    residualClass: /^\s*&/u.test(rustType) ? "entry-parameter" : undefined,
  };
}

function classifyResidualRawPropertyCarrierSites(
  discovered: readonly DiscoveredSite[],
  _previous: readonly CensusSite[] | undefined,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  return discovered.map((site) => {
    if (site.provenance && site.provenanceDerivation && site.residualClass)
      return { ...site, disposition: "named-exempt" } as CensusSite;
    return { ...site, disposition: "unclassified" } as const;
  });
}

function discoverResidualPropertyCarrierConsumers(
  carriers: readonly CensusSite[],
  sources: readonly MutableRustSource[],
): ResidualPropertyCarrierConsumerRow[] {
  const carrierByField = new Map<string, CensusSite>();
  const carrierByParameter = new Map<string, CensusSite>();
  for (const carrier of carriers) {
    const field = carrier.operation.match(
      /^residual-property-field:([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*):/u,
    );
    if (field) carrierByField.set(`${field[1]}.${field[2]}`, carrier);
    const parameter = carrier.operation.match(
      /^residual-property-parameter:([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*):/u,
    );
    if (parameter)
      carrierByParameter.set(`${carrier.path}\u0000${parameter[1]}.${parameter[2]}`, carrier);
  }

  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFieldTypes = rustStructFieldTypes(sources, aliasesByPath);
  const consumersByCarrier = new Map<CensusSite, ResidualPropertyConsumerSite[]>();
  for (const carrier of carriers) consumersByCarrier.set(carrier, []);

  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const bindings = resolvedRustBindingsForFunction(
        functionSlice,
        scannable,
        aliases,
        structFieldTypes,
      );
      for (const access of functionSlice.scannable.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\b/gu,
      )) {
        const receiverType = bindings.get(access[1]);
        if (!receiverType) continue;
        const carrier = carrierByField.get(`${receiverType}.${access[2]}`);
        if (!carrier) continue;
        const offset = functionSlice.bodyStart + access.index;
        consumersByCarrier
          .get(carrier)
          ?.push(
            residualConsumerSiteAt(
              relativePath,
              source,
              scannable,
              offset,
              functionSlice,
              `${access[1]}.${access[2]}`,
            ),
          );
      }

      for (const [key, carrier] of carrierByParameter) {
        const [carrierPath, functionAndParameter] = key.split("\u0000");
        if (carrierPath !== relativePath) continue;
        const separator = functionAndParameter.lastIndexOf(".");
        const functionName = functionAndParameter.slice(0, separator);
        const parameterName = functionAndParameter.slice(separator + 1);
        if (functionName !== functionSlice.name) continue;
        const parameterUse = new RegExp(`\\b${escapeRegExp(parameterName)}\\b`, "gu");
        const authorityShadow = functionSlice.scannable.match(
          new RegExp(
            String.raw`\blet\s+${escapeRegExp(parameterName)}\s*=\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*PropertyNameV0\s*::[^;]*\b${escapeRegExp(parameterName)}\b[^;]*;`,
            "u",
          ),
        );
        const authorityShadowEnd = authorityShadow
          ? (authorityShadow.index ?? 0) + authorityShadow[0].length
          : undefined;
        for (const use of functionSlice.scannable.matchAll(parameterUse)) {
          if (authorityShadowEnd !== undefined && use.index >= authorityShadowEnd) continue;
          const offset = functionSlice.bodyStart + use.index;
          consumersByCarrier
            .get(carrier)
            ?.push(
              residualConsumerSiteAt(
                relativePath,
                source,
                scannable,
                offset,
                functionSlice,
                parameterName,
              ),
            );
        }
      }
    }
  }

  return carriers.map((carrier) => ({
    carrier,
    consumers: uniqueSites(consumersByCarrier.get(carrier) ?? []),
  }));
}

function residualConsumerSiteAt(
  relativePath: string,
  source: string,
  scannable: string,
  offset: number,
  functionSlice: RustFunctionSlice,
  expression: string,
): ResidualPropertyConsumerSite {
  const localOffset = offset - functionSlice.bodyStart;
  const contextStart = Math.max(
    0,
    Math.max(
      functionSlice.scannable.lastIndexOf(";", localOffset - 1),
      functionSlice.scannable.lastIndexOf("{", localOffset - 1),
      functionSlice.scannable.lastIndexOf("}", localOffset - 1),
    ) + 1,
  );
  const nextSemicolon = functionSlice.scannable.indexOf(";", localOffset);
  const contextEnd =
    nextSemicolon < 0
      ? Math.min(functionSlice.scannable.length, localOffset + 320)
      : Math.min(functionSlice.scannable.length, nextSemicolon + 1);
  const context = functionSlice.scannable.slice(contextStart, contextEnd);
  const sourceLine = source.split(/\r?\n/u)[lineNumberAt(source, offset) - 1] ?? "";
  const expressionCore = escapeRegExp(expression).replaceAll("\\.", String.raw`\s*\.\s*`);
  const expressionPattern = String.raw`(?<![A-Za-z0-9_.])${expressionCore}(?![A-Za-z0-9_])`;
  const authorityCall = new RegExp(
    String.raw`(?:PropertyNameV0\s*::\s*(?:from_authored|custom|standard|canonical_custom_key|canonical_standard_key)|property_names_same)\s*\([^;{}]*?${expressionPattern}[^;{}]*?\)`,
    "gu",
  );
  const identityContext = context.replace(authorityCall, "sealed_property_result");
  const identityLine = sourceLine.replace(authorityCall, "sealed_property_result");
  const directMethod = new RegExp(
    String.raw`${expressionPattern}(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\([^;{}]*?\))*\s*\.\s*(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)\s*\(`,
    "u",
  );
  const directBinary = new RegExp(
    String.raw`(?:${expressionPattern}(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\([^;{}]*?\))*\s*(?:==|!=)|(?:==|!=)\s*&?\s*${expressionPattern})`,
    "u",
  );
  const collectionArgument = new RegExp(
    String.raw`\.\s*(?:insert|get|entry|contains_key|contains|remove|binary_search(?:_by|_by_key)?)\s*\([^;{}]*${expressionPattern}`,
    "u",
  );
  const directMatch = new RegExp(
    String.raw`(?:\bmatch\s+${expressionPattern}\b|\bmatches\s*!\s*\(\s*${expressionPattern}\b|\b(?:str|PartialEq)\s*::\s*eq\s*\([^;{}]*${expressionPattern})`,
    "u",
  );
  const identityShaped =
    directMethod.test(identityLine) ||
    directBinary.test(identityLine) ||
    collectionArgument.test(identityLine) ||
    directMatch.test(identityContext);
  const classification: ResidualPropertyConsumerClassification = identityShaped
    ? "identity-shaped"
    : /(?:serde|serialize|json\s*!|format\s*!|write\s*!|writeln\s*!|\.to_string\s*\(|\breturn\b)/u.test(
          context,
        )
      ? "egress"
      : "presentation";
  return {
    ...siteAt(
      relativePath,
      source,
      scannable,
      offset,
      `residual-property-consumer:${classification}`,
    ),
    classification,
  };
}

function rustTypeAliasesByPath(
  sources: readonly MutableRustSource[],
): ReadonlyMap<string, ReadonlyMap<string, string>> {
  const byPath = new Map<string, ReadonlyMap<string, string>>();
  for (const { relativePath, source } of sources) {
    const aliases = new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const alias of scannable.matchAll(
      /\buse\s+(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      aliases.set(alias[2], alias[1]);
    }
    byPath.set(relativePath, aliases);
  }
  return byPath;
}

function rustStructFieldTypes(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, ReadonlyMap<string, string>> {
  const structures = new Map<string, Map<string, string>>();
  for (const { relativePath, source } of sources) {
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const structure of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = structure.index + structure[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const fields = structures.get(structure[1]) ?? new Map<string, string>();
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu,
      )) {
        fields.set(field[1], normalizeRustType(field[2], aliases));
      }
      structures.set(structure[1], fields);
    }
    for (const enumeration of scannable.matchAll(/\benum\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = enumeration.index + enumeration[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const fields = structures.get(enumeration[1]) ?? new Map<string, string>();
      const body = scannable.slice(openBrace + 1, closeBrace);
      let syntheticIndex = fields.size;
      for (const payload of body.matchAll(/\b[A-Za-z_][A-Za-z0-9_]*\s*\(([^)]*)\)/gu)) {
        for (const rustType of payload[1]
          .split(",")
          .map((part) => part.trim())
          .filter(Boolean)) {
          fields.set(`variant_${syntheticIndex}`, normalizeRustType(rustType, aliases));
          syntheticIndex += 1;
        }
      }
      for (const payload of body.matchAll(/\b[A-Za-z_][A-Za-z0-9_]*\s*\{([^}]*)\}/gu)) {
        for (const field of payload[1].matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu)) {
          fields.set(`variant_${syntheticIndex}_${field[1]}`, normalizeRustType(field[2], aliases));
          syntheticIndex += 1;
        }
      }
      structures.set(enumeration[1], fields);
    }
  }
  return structures;
}

function resolvedRustBindingsForFunction(
  functionSlice: RustFunctionSlice,
  source: string,
  aliases: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, string> {
  const bindings = new Map<string, string>();
  const text = `${functionSlice.signature}{${functionSlice.scannable}}`;
  for (const binding of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,)=;{]+(?:<[^;{=]+>)?)/gu,
  )) {
    bindings.set(binding[1], normalizeRustType(binding[2], aliases));
  }
  const selfType = enclosingImplType(source, functionSlice.bodyStart, aliases);
  if (selfType) bindings.set("self", selfType);

  for (let pass = 0; pass < 4; pass += 1) {
    let changed = false;
    for (const closure of functionSlice.scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.iter\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
    )) {
      const receiverType = resolveRustAccessType(closure[1], bindings, structFields);
      const elementType = rustCollectionElementType(receiverType, aliases) ?? receiverType;
      if (elementType && bindings.get(closure[2]) !== elementType) {
        bindings.set(closure[2], elementType);
        changed = true;
      }
    }
    if (!changed) break;
  }
  return bindings;
}

function normalizeRustType(typeSource: string, aliases: ReadonlyMap<string, string>): string {
  const cleaned = typeSource
    .replace(/&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?/gu, "")
    .replace(/\bmut\s+/gu, "")
    .trim();
  const identifiers = cleaned.match(/[A-Za-z_][A-Za-z0-9_]*/gu) ?? [];
  const named = identifiers.findLast((identifier) => /^[A-Z]/u.test(identifier));
  if (!named) return cleaned;
  return aliases.get(named) ?? named;
}

function rustCollectionElementType(
  typeSource: string | undefined,
  aliases: ReadonlyMap<string, string>,
): string | undefined {
  if (!typeSource) return undefined;
  const generic = typeSource.match(
    /(?:Vec|Box|Arc|Rc|Option|BTreeSet|HashSet)\s*<\s*([^,>]+)/u,
  )?.[1];
  const slice = typeSource.match(/\[\s*([^;\]]+)/u)?.[1];
  const candidate = generic ?? slice;
  return candidate ? normalizeRustType(candidate, aliases) : undefined;
}

function resolveRustAccessType(
  access: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): string | undefined {
  const [root, ...segments] = access.split(".");
  let current = bindings.get(root);
  for (const segment of segments) {
    if (!current) return undefined;
    current = structFields.get(current)?.get(segment);
  }
  return current;
}

function enclosingImplType(
  source: string,
  offset: number,
  aliases: ReadonlyMap<string, string>,
): string | undefined {
  let candidate: string | undefined;
  for (const implementation of source.matchAll(
    /\bimpl(?:\s*<[^>{}]*>)?\s+(?:[^{}]*?\s+for\s+)?([A-Za-z_][A-Za-z0-9_:]*)[^{}]*\{/gu,
  )) {
    if (implementation.index > offset) break;
    const openBrace = implementation.index + implementation[0].lastIndexOf("{");
    const closeBrace = matchingBrace(source, openBrace);
    if (closeBrace !== undefined && offset < closeBrace) {
      candidate = normalizeRustType(implementation[1], aliases);
    }
  }
  return candidate;
}

interface RustFunctionSlice {
  readonly name: string;
  readonly signature: string;
  readonly scannable: string;
  readonly bodyStart: number;
  readonly bodyEnd: number;
  readonly propertyContext: boolean;
}

interface RawStringCollectionDeclaration {
  readonly binding: string;
  readonly offset: number;
}

interface MutableRustSource {
  readonly relativePath: string;
  source: string;
}

function discoverTypedRawPropertyIdentitySites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  sources.push(
    ...(generatedFixtureManifest?.sources.map((source) => ({
      relativePath: source.relativePath,
      source: source.source,
    })) ?? []),
  );
  applyResidualCarrierProbeMutations(sources);
  applyPropertyAuthorityDecreaseMutation(sources);
  if (injectPropertyStructuralEquality) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_identity.rs",
      source:
        "fn injected(left_property: String, right_property: String) -> bool { left_property == right_property }\n",
    });
  }
  if (injectPropertyRoundtripEquality) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_roundtrip.rs",
      source:
        "struct Input { property: String }\nfn injected(actual: &Input, expected: &str) -> bool { actual.property != expected.as_ref() }\n",
    });
  }
  if (injectPropertyRawMap) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_map.rs",
      source:
        "use std::collections::HashMap;\nfn injected() {\n  let custom_property_index: HashMap<&'static str, u8> = HashMap::new();\n  drop(custom_property_index);\n}\n",
    });
  }
  if (injectPropertyRawCanonicalization) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property(left_property: &str) -> String { left_property.to_ascii_lowercase() }",
    );
  }
  if (injectPropertyFqnRawMap) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_property_fqn_map() {\n  let custom_property_index: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new();\n  drop(custom_property_index);\n}",
    );
  }
  if (injectPropertyValuesRawMap) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_property_values_map() {\n  let custom_property_values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new();\n  drop(custom_property_values);\n}",
    );
  }
  if (injectPropertySameLineRawOperation) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_same_line_property_operation(left_property: String, right_property: String) -> bool { let _ = omena_syntax::ident::PropertyNameV0::from_authored(&left_property); left_property == right_property }",
    );
  }
  if (injectPropertyNewFileRawComparison) {
    appendTrackedSourceMutation(
      sources,
      "struct PropertyOperands { first: String, second: String }\nfn compare(values: &PropertyOperands) -> bool { values.first == values.second }",
    );
  }
  if (injectPropertyNewFileRawCanonicalization) {
    appendTrackedSourceMutation(
      sources,
      "struct PropertySpelling { text: String }\nfn canonicalize(value: &PropertySpelling) -> String { value.text.to_lowercase() }",
    );
  }
  if (injectPropertyTrimChain) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property_name(property_name: &str) -> String { property_name.trim().to_ascii_lowercase() }",
    );
  }
  if (injectPropertyContextRawOperations) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property_axis_order(declared: &str) -> String { declared.trim().to_ascii_lowercase() }\nfn compare_property_axis_order(left_name: &str, right_name: &str) -> bool { left_name == right_name }",
    );
  }
  if (injectAuthoredUppercaseTransform) {
    appendTrackedSourceMutation(
      sources,
      'fn authored_property_uppercase_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().to_uppercase() == "--PROBE" }',
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }
  if (injectAuthoredTrimMatchesTransform) {
    appendTrackedSourceMutation(
      sources,
      "fn authored_property_trim_matches_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().trim_matches('-') == \"probe\" }",
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }
  if (injectAuthoredStripPrefixTransform) {
    appendTrackedSourceMutation(
      sources,
      'fn authored_property_strip_prefix_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().strip_prefix("--") == Some("probe") }',
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }

  const carrierFields = discoverRawPropertyCarrierFields(sources);
  const functionResultKinds = rustFunctionResultKinds(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const sites: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const stringAwareScannable = maskCommentsStringsAndTestItems(source, true);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const classifier = propertyOperandClassifier(
        functionSlice,
        carrierFields,
        true,
        aliasesByPath.get(relativePath),
      );
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text !== "==" && token.text !== "!=") continue;
        const left = maskNonRawResultCalls(
          maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "left")),
          functionResultKinds,
        );
        const right = maskNonRawResultCalls(
          maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "right")),
          functionResultKinds,
        );
        if (
          !classifier.containsRawPropertyAccess(left) &&
          !classifier.containsRawPropertyAccess(right)
        )
          continue;
        const absoluteOffset = functionSlice.bodyStart + token.start;
        const sourceLine = source.split(/\r?\n/u)[lineNumberAt(source, absoluteOffset) - 1] ?? "";
        if (
          !classifier.containsRawPropertyAccess(
            maskPropertyAuthorityCalls(rustSemanticTokens(sourceLine)),
          )
        )
          continue;
        sites.push(
          siteAt(relativePath, source, scannable, absoluteOffset, "raw-property-comparison"),
        );
      }

      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          !/^(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)$/u.test(token.text) ||
          tokens[tokenIndex - 1]?.text !== "." ||
          tokens[tokenIndex + 1]?.text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 1, "(", ")");
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        const argumentTokens =
          closeParen === undefined ? [] : tokens.slice(tokenIndex + 2, closeParen);
        if (
          !classifier.containsRawPropertyOrigin(receiver) &&
          !classifier.containsRawPropertyOrigin(argumentTokens)
        ) {
          continue;
        }
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "raw-property-identity-method",
          ),
        );
      }

      for (let tokenIndex = 0; tokenIndex + 3 < tokens.length; tokenIndex += 1) {
        const ufcsEquality =
          (tokens[tokenIndex].text === "str" || tokens[tokenIndex].text === "PartialEq") &&
          tokens[tokenIndex + 1]?.text === "::" &&
          tokens[tokenIndex + 2]?.text === "eq" &&
          tokens[tokenIndex + 3]?.text === "(";
        if (!ufcsEquality) continue;
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 3, "(", ")");
        if (closeParen === undefined) continue;
        if (!classifier.containsRawPropertyOrigin(tokens.slice(tokenIndex + 4, closeParen)))
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + tokens[tokenIndex + 2].start,
            "raw-property-ufcs-identity",
          ),
        );
      }

      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          token.text !== "to_ascii_lowercase" &&
          token.text !== "make_ascii_lowercase" &&
          token.text !== "to_lowercase" &&
          token.text !== "to_ascii_uppercase" &&
          token.text !== "make_ascii_uppercase" &&
          token.text !== "to_uppercase"
        )
          continue;
        if (tokens[tokenIndex - 1]?.text !== "." || tokens[tokenIndex + 1]?.text !== "(") continue;
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        if (!classifier.containsRawPropertyOrigin(receiver)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "raw-property-canonicalization",
          ),
        );
      }

      for (const collection of rawStringCollectionDeclarations(functionSlice)) {
        const propertyKeyed =
          isPropertyMapBinding(collection.binding) ||
          classifier.collectionReceivesPropertyKey(collection.binding);
        if (!propertyKeyed) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + collection.offset,
            "raw-custom-property-map",
          ),
        );
      }

      const operand = String.raw`&?\s*[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*(?:\.(?:as_ref|as_str)\s*\(\s*\))*`;
      const rawPrefix = new RegExp(
        String.raw`(?<operand>${operand})\.starts_with\s*\(\s*"--"\s*\)`,
        "gu",
      );
      const stringAwareBody = stringAwareScannable.slice(
        functionSlice.bodyStart,
        functionSlice.bodyEnd,
      );
      for (const match of stringAwareBody.matchAll(rawPrefix)) {
        if (!classifier.isRawPropertyExpression(match.groups?.operand ?? "")) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            stringAwareScannable,
            functionSlice.bodyStart + match.index,
            "raw-custom-property-prefix",
          ),
        );
      }
    }
  }
  return uniqueSites(sites);
}

function appendTrackedSourceMutation(
  sources: MutableRustSource[],
  mutation: string,
  relativePath = "rust/crates/omena-cascade/src/axis_order.rs",
): void {
  const target = sources.find((source) => source.relativePath === relativePath);
  assert.ok(target, "tracked property mutation target must be in census scope");
  target.source = `${target.source.trimEnd()}\n\n${mutation}\n`;
}

function applyResidualCarrierProbeMutations(sources: MutableRustSource[]): void {
  const targetPath = "rust/crates/omena-cascade/src/axis_order.rs";
  if (injectResidualKeyedCarrierWithoutJoin) {
    appendTrackedSourceMutation(
      sources,
      "struct InjectedKeyedPropertyCarrierWithoutJoin { property_key: omena_syntax::ident::CanonicalCustomPropertyNameV0, property_name: String }",
      targetPath,
    );
  }
  if (injectResidualKeyedCarrierWithJoin) {
    appendTrackedSourceMutation(
      sources,
      "struct InjectedKeyedPropertyCarrierWithJoin { property_key: omena_syntax::ident::CanonicalCustomPropertyNameV0, property_name: String }\nfn injected_keyed_property_carrier_raw_join(left: &InjectedKeyedPropertyCarrierWithJoin, right: &InjectedKeyedPropertyCarrierWithJoin) -> bool { left.property_name == right.property_name }",
      targetPath,
    );
  }
  if (injectCanonicalVectorCarrier) {
    appendTrackedSourceMutation(
      sources,
      "struct InjectedCanonicalPropertyVectorCarrier { property_names: Vec<String> }\nfn injected_canonical_property_vector(property: &omena_syntax::ident::PropertyNameV0) -> InjectedCanonicalPropertyVectorCarrier { InjectedCanonicalPropertyVectorCarrier { property_names: vec![property.canonical_name().to_string()] } }",
      targetPath,
    );
  }
  if (injectResidualEntryParameterInventory) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_residual_entry_parameter(property_name: &str) -> omena_syntax::ident::CanonicalPropertyKeyV0 { omena_syntax::ident::PropertyNameV0::from_authored(property_name).canonical_key() }",
      targetPath,
    );
  }
  if (injectResidualStaticLiteralInventory) {
    appendTrackedSourceMutation(
      sources,
      'struct InjectedStaticPropertyLiteral { property_name: &\'static str }\nconst INJECTED_STATIC_PROPERTY_LITERAL: InjectedStaticPropertyLiteral = InjectedStaticPropertyLiteral { property_name: "color" };\nfn injected_static_property_key() -> omena_syntax::ident::CanonicalStandardPropertyNameV0 { omena_syntax::ident::PropertyNameV0::canonical_standard_key(INJECTED_STATIC_PROPERTY_LITERAL.property_name) }',
      targetPath,
    );
  }
  if (injectResidualNonPropertyInventory) {
    appendTrackedSourceMutation(
      sources,
      "struct InjectedNonPropertySelectorCarrier { property_selector: String }\nfn injected_non_property_selector(selector_name: String) -> InjectedNonPropertySelectorCarrier { InjectedNonPropertySelectorCarrier { property_selector: selector_name } }",
      targetPath,
    );
  }
  if (injectResidualIdentityConsumer) {
    appendTrackedSourceMutation(
      sources,
      'struct InjectedResidualIdentityConsumer { property_name: &\'static str }\nconst INJECTED_RESIDUAL_IDENTITY_CONSUMER: InjectedResidualIdentityConsumer = InjectedResidualIdentityConsumer { property_name: "color" };\nfn injected_residual_identity_consumer(entry: &InjectedResidualIdentityConsumer, expected: &str) -> bool { entry.property_name == expected }',
      targetPath,
    );
  }
}

function assertResidualCarrierProbeOutcomes(sites: readonly DiscoveredSite[]): void {
  const operationFor = (typeName: string) =>
    sites.find((site) =>
      site.operation.startsWith(`residual-property-field:${typeName}.property_name:`),
    );
  if (injectResidualKeyedCarrierWithoutJoin) {
    const site = operationFor("InjectedKeyedPropertyCarrierWithoutJoin");
    assert.ok(site, "keyed raw authored carrier without a join must remain inventoried");
    process.stderr.write("residualKeyedCarrierWithoutJoin=inventoried\n");
  }
  if (injectResidualKeyedCarrierWithJoin) {
    const site = operationFor("InjectedKeyedPropertyCarrierWithJoin");
    assert.ok(site, "keyed raw authored carrier with a join must remain inventoried");
    process.stderr.write("residualKeyedCarrierWithJoin=inventoried\n");
  }
  if (injectCanonicalVectorCarrier) {
    const site = sites.find((candidate) =>
      candidate.operation.startsWith(
        "residual-property-field:InjectedCanonicalPropertyVectorCarrier.property_names:",
      ),
    );
    assert.equal(site?.provenance, "p-canonical", "canonical Vec<String> provenance");
    assert.equal(
      site?.residualClass,
      "canonical-text-carrier",
      "canonical Vec<String> must route to R4",
    );
    process.stderr.write("canonicalVectorCarrier=p-canonical:R4\n");
  }
  if (injectContainerPredicateRevert) {
    assert.ok(
      !sites.some(
        (site) =>
          site.path === "rust/crates/engine-style-parser/src/lib.rs" &&
          site.operation.startsWith(
            "residual-property-field:IndexSummaryAcc.custom_property_decl_names:Vec<String>",
          ),
      ),
      "container-predicate mutation must remove the known Vec<String> authored carrier",
    );
    process.stderr.write("knownAuthoredVectorCarrier=missing-after-container-predicate-revert\n");
  }
}

function applyPropertyAuthorityDecreaseMutation(sources: MutableRustSource[]): void {
  if (!injectPropertyAuthorityDecreaseLaundering) return;
  const targetPath = "rust/crates/omena-lsp-server/src/lib.rs";
  const target = sources.find((source) => source.relativePath === targetPath);
  assert.ok(target, "authority decrease mutation target must be in census scope");
  const authorityJoin = "target.property_key == candidate.property_key";
  const launderingJoin =
    "target.name.to_string().eq_ignore_ascii_case(&candidate.name.to_string())";
  assert.equal(
    target.source.split(authorityJoin).length - 1,
    1,
    "authority decrease mutation must replace exactly one sealed join",
  );
  target.source = target.source.replace(authorityJoin, launderingJoin);
}

function discoverRawPropertyCarrierFields(
  sources: readonly { relativePath: string; source: string }[],
): ReadonlyMap<string, ReadonlySet<string>> {
  const fields = new Map<string, Set<string>>();
  const classifications = deriveRawPropertyFieldClassifications(sources);
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of rustStructFields(body)) {
        if (
          rawPropertyCarrierType(field.rustType) &&
          classifications.has(rawPropertyFieldKey(relativePath, typeName, field.name))
        ) {
          const typeFields = fields.get(typeName) ?? new Set<string>();
          typeFields.add(field.name);
          fields.set(typeName, typeFields);
        }
      }
    }
  }
  return fields;
}

function rustFunctionSlices(scannable: string): RustFunctionSlice[] {
  const functions: RustFunctionSlice[] = [];
  for (const match of scannable.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
    const openParen = scannable.indexOf("(", match.index + match[0].length);
    if (openParen < 0) continue;
    const closeParen = matchingDelimiter(scannable, openParen, "(", ")");
    if (closeParen === undefined) continue;
    const openBrace = scannable.indexOf("{", closeParen + 1);
    const semicolon = scannable.indexOf(";", closeParen + 1);
    if (openBrace < 0 || (semicolon >= 0 && semicolon < openBrace)) continue;
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    const signature = scannable.slice(match.index, openBrace);
    const propertyContext = isPropertyIdentityFunctionIdentifier(match[1]);
    functions.push({
      name: match[1],
      signature,
      scannable: scannable.slice(openBrace + 1, closeBrace),
      bodyStart: openBrace + 1,
      bodyEnd: closeBrace,
      propertyContext,
    });
  }
  return functions;
}

function propertyOperandClassifier(
  functionSlice: RustFunctionSlice,
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
  includeRawParameters = true,
  aliases: ReadonlyMap<string, string> = new Map(),
  authoredReturningFunctions: ReadonlySet<string> = new Set(),
): {
  readonly isRawPropertyExpression: (expression: string) => boolean;
  readonly containsRawPropertyAccess: (tokens: readonly RustSemanticToken[]) => boolean;
  readonly containsRawPropertyOrigin: (tokens: readonly RustSemanticToken[]) => boolean;
  readonly collectionReceivesPropertyKey: (binding: string) => boolean;
  readonly collectionUsesIdentityOperation: (binding: string) => boolean;
} {
  const text = `${functionSlice.signature}{${functionSlice.scannable}}`;
  const rawBindings = new Set<string>();
  const propertyBindings = new Set<string>();
  const bindingTypes = new Map<string, string>();
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str|String)\b/gu,
  )) {
    if (!includeRawParameters) continue;
    rawBindings.add(match[1]);
    if (
      isPropertySemanticIdentifier(match[1]) ||
      (functionSlice.propertyContext &&
        /^(?:name|left|right|actual|expected|target|candidate|declared)$/u.test(match[1]))
    ) {
      propertyBindings.add(match[1]);
    }
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?([A-Za-z_][A-Za-z0-9_:]*)\b/gu,
  )) {
    const typeName = normalizeRustType(match[2], aliases);
    if (typeName !== "AuthoredPropertyTextV0") continue;
    rawBindings.add(match[1]);
    propertyBindings.add(match[1]);
    bindingTypes.set(match[1], typeName);
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?)?(?:mut\s+)?([A-Za-z_][A-Za-z0-9_:]*)/gu,
  )) {
    bindingTypes.set(match[1], normalizeRustType(match[2], aliases));
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\]/gu,
  )) {
    bindingTypes.set(match[1], normalizeRustType(match[2], aliases));
  }
  for (const closure of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\.iter\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
  )) {
    const elementType = bindingTypes.get(closure[1]);
    if (elementType) bindingTypes.set(closure[2], elementType);
  }

  const accessIsRawProperty = (access: RustAccessPath): boolean => {
    return rawPropertyExpressionKind(
      access,
      rawBindings,
      propertyBindings,
      bindingTypes,
      carrierFields,
    );
  };

  const containsRawPropertyAccess = (tokens: readonly RustSemanticToken[]): boolean => {
    return rustAccessPaths(tokens).some(accessIsRawProperty);
  };
  const containsRawPropertyOrigin = (tokens: readonly RustSemanticToken[]): boolean => {
    return (
      rustAccessPaths(tokens).some((access) =>
        rawPropertyExpressionKind(
          access,
          rawBindings,
          propertyBindings,
          bindingTypes,
          carrierFields,
          true,
        ),
      ) ||
      tokens.some(
        (token, index) =>
          authoredReturningFunctions.has(token.text) && tokens[index + 1]?.text === "(",
      )
    );
  };

  for (let pass = 0; pass < 4; pass += 1) {
    let changed = false;
    for (const assignment of functionSlice.scannable.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^=;]*=\s*([^;]+);/gu,
    )) {
      if (
        !propertyBindings.has(assignment[1]) &&
        !/\b(?:PropertyNameV0|CanonicalPropertyKeyV0|CanonicalCustomPropertyNameV0|CanonicalStandardPropertyNameV0)\b|\.(?:canonical_key|to_property_name|to_custom_key|to_standard_key)\s*\(/u.test(
          assignment[2],
        ) &&
        authoredStringExpression(
          assignment[2],
          containsRawPropertyOrigin,
          authoredReturningFunctions,
        )
      ) {
        propertyBindings.add(assignment[1]);
        rawBindings.add(assignment[1]);
        changed = true;
      }
    }
    if (!changed) break;
  }

  return {
    isRawPropertyExpression(expression: string): boolean {
      return containsRawPropertyAccess(rustSemanticTokens(expression));
    },
    containsRawPropertyAccess,
    containsRawPropertyOrigin,
    collectionReceivesPropertyKey(binding: string): boolean {
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (let index = 0; index + 3 < tokens.length; index += 1) {
        if (
          tokens[index].text !== binding ||
          tokens[index + 1].text !== "." ||
          !["entry", "get", "contains_key", "remove", "insert", "push"].includes(
            tokens[index + 2].text,
          ) ||
          tokens[index + 3].text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, index + 3, "(", ")");
        if (closeParen === undefined) continue;
        const argument = firstArgumentTokens(tokens.slice(index + 4, closeParen));
        if (containsRawPropertyAccess(argument)) return true;
      }
      return false;
    },
    collectionUsesIdentityOperation(binding: string): boolean {
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (let index = 0; index + 3 < tokens.length; index += 1) {
        if (
          tokens[index].text === binding &&
          tokens[index + 1].text === "." &&
          /^(?:get|entry|contains_key|contains|remove|sort(?:_by|_by_key|_by_cached_key|_unstable(?:_by|_by_key)?)?|dedup(?:_by|_by_key)?|binary_search(?:_by|_by_key)?)$/u.test(
            tokens[index + 2].text,
          ) &&
          tokens[index + 3].text === "("
        ) {
          return true;
        }
      }
      return false;
    },
  };
}

function rawStringValueExpression(expression: string): boolean {
  return /^\s*&?\s*[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*(?:\(\s*\))?)*\s*$/u.test(
    expression,
  );
}

function authoredStringExpression(
  expression: string,
  containsOrigin: (tokens: readonly RustSemanticToken[]) => boolean,
  authoredReturningFunctions: ReadonlySet<string>,
): boolean {
  const tokens = rustSemanticTokens(expression);
  if (!containsOrigin(tokens)) return false;
  return (
    !/(?:==|!=|\b(?:matches|match)\b|\.\s*(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)\s*\()/u.test(
      expression,
    ) &&
    (/^\s*format\s*!\s*\(/u.test(expression) ||
      rawStringValueExpression(expression) ||
      (() => {
        const called = expression.match(
          /^\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*\(/u,
        )?.[1];
        return called !== undefined && authoredReturningFunctions.has(called);
      })())
  );
}

function authoredReturningFunctionNames(
  sources: readonly MutableRustSource[],
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
): ReadonlySet<string> {
  const returning = new Set<string>();
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  for (let pass = 0; pass < 3; pass += 1) {
    let changed = false;
    for (const { relativePath, source } of sources) {
      const scannable = maskCommentsStringsAndTestItems(source, false);
      const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
      for (const functionSlice of rustFunctionSlices(scannable)) {
        if (returning.has(functionSlice.name)) continue;
        const returnType = functionSlice.signature.match(
          /->\s*([^\n{]+?)(?:\s+where\b|\s*$)/u,
        )?.[1];
        if (!returnType || !rawStringType(returnType.trim())) continue;
        const bindings = resolvedRustBindingsForFunction(
          functionSlice,
          source,
          aliases,
          structFields,
        );
        const directAuthoredParameter = [...bindings].some(
          ([binding, typeName]) =>
            typeName === "AuthoredPropertyTextV0" &&
            new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(functionSlice.scannable),
        );
        const authoredFieldAccess = rustAccessPaths(
          rustSemanticTokens(functionSlice.scannable),
        ).some((access) => {
          let currentType = bindings.get(access.root);
          for (const segment of access.segments) {
            if (segment.method) break;
            if (currentType && carrierFields.get(currentType)?.has(segment.name)) return true;
            currentType = currentType
              ? structFields.get(currentType)?.get(segment.name)
              : undefined;
          }
          return false;
        });
        const callsAuthoredReturning = [...returning].some((name) =>
          new RegExp(`\\b${escapeRegExp(name)}\\s*\\(`, "u").test(functionSlice.scannable),
        );
        if (!directAuthoredParameter && !authoredFieldAccess && !callsAuthoredReturning) continue;
        returning.add(functionSlice.name);
        changed = true;
      }
    }
    if (!changed) break;
  }
  return returning;
}

function validateGeneratedFixtureManifest(manifest: GeneratedFixtureManifest): void {
  const pinnedOriginBaseline = [
    "field-access",
    "bare-parameter",
    "fqn-parameter",
    "alias-parameter",
    "closure-inferred",
    "two-statement-local",
    "wrapper-function",
    "named-escape",
  ] as const;
  const pinnedComparisonBaseline = authoredComparisonAxis.slice(0, 33);
  const axisBaselines = {
    origins: "7976976895e7c7c6e6041b58cc4738c1a85b321a9fdd5828489591dc89ada27d",
    comparisons: "4f0b68e1929f55090fddf2560e81fd292006d390ac1a5ec94c5ad32b3b26c19e",
    positions: "ffd26a47bdfef655e9e2cafa05f544deb48354f1153ab53ced4f88285008744e",
  } as const;
  assert.deepEqual(manifest.axes.origins, authoredOriginAxis, "origin axis literal drift");
  assert.deepEqual(
    manifest.axes.comparisons,
    authoredComparisonAxis,
    "comparison axis literal drift",
  );
  assert.deepEqual(manifest.axes.positions, authoredPositionAxis, "position axis literal drift");
  assert.deepEqual(manifest.axes.escapes, authoredEscapeAxis, "escape axis literal drift");
  assert.deepEqual(
    manifest.axes.pinnedOriginBaseline,
    pinnedOriginBaseline,
    "pinned origin baseline drift",
  );
  assert.deepEqual(
    manifest.axes.pinnedComparisonBaseline,
    pinnedComparisonBaseline,
    "pinned comparison baseline drift",
  );
  assert.deepEqual(manifest.axes.axisBaselines, axisBaselines, "axis baseline receipt drift");
  assert.equal(
    createHash("sha256").update(JSON.stringify(pinnedOriginBaseline)).digest("hex"),
    axisBaselines.origins,
    "pinned origin digest",
  );
  assert.equal(
    createHash("sha256").update(JSON.stringify(pinnedComparisonBaseline)).digest("hex"),
    axisBaselines.comparisons,
    "pinned comparison digest",
  );
  assert.equal(
    createHash("sha256").update(JSON.stringify(authoredPositionAxis)).digest("hex"),
    axisBaselines.positions,
    "pinned position digest",
  );

  const derivedEscapes = derivedEscapeAxisFromWorkspaceManifests();
  assert.deepEqual(
    derivedEscapes,
    authoredEscapeAxis,
    "serialization entry point has no escape id or the escape grammar shrank",
  );
  assert.equal(
    manifest.fullProductCells.length,
    authoredOriginAxis.length * authoredComparisonAxis.length * authoredPositionAxis.length,
    "full origin x comparison x position product count",
  );
  assert.equal(
    manifest.escapeCoveringCells.length,
    authoredEscapeAxis.length * authoredComparisonAxis.length,
    "escape covering-array count",
  );

  const sourcesByFunction = generatedFixtureFunctions(manifest.sources);
  const detectedFullCells: GeneratedFixtureCell[] = [];
  const detectedEscapeCells: (GeneratedFixtureCell & { readonly escape: AuthoredEscapeId })[] = [];
  const seenFunctions = new Set<string>();
  const cellArmReceipts: string[] = [];
  for (const expected of manifest.fullProductCells) {
    assert.ok(!seenFunctions.has(expected.functionName), `duplicate cell ${expected.functionName}`);
    seenFunctions.add(expected.functionName);
    const functionBody = sourcesByFunction.get(expected.functionName);
    assert.ok(functionBody, `generated cell function missing: ${expected.functionName}`);
    const detected = detectGeneratedFixtureCell(functionBody);
    assert.deepEqual(
      detected,
      {
        functionName: expected.functionName,
        origin: expected.origin,
        comparison: expected.comparison,
        position: expected.position,
      },
      `generated cell ${expected.functionName} did not fire its named arm`,
    );
    detectedFullCells.push(detected);
    cellArmReceipts.push(
      `${detected.functionName}:${detected.origin}/${detected.comparison}/${detected.position}`,
    );
  }
  for (const expected of manifest.escapeCoveringCells) {
    assert.ok(!seenFunctions.has(expected.functionName), `duplicate cell ${expected.functionName}`);
    seenFunctions.add(expected.functionName);
    const functionBody = sourcesByFunction.get(expected.functionName);
    assert.ok(functionBody, `generated escape cell function missing: ${expected.functionName}`);
    const detected = detectGeneratedFixtureCell(functionBody, expected.escape);
    assert.deepEqual(
      detected,
      {
        functionName: expected.functionName,
        origin: expected.origin,
        comparison: expected.comparison,
        position: expected.position,
        escape: expected.escape,
      },
      `generated escape cell ${expected.functionName} did not fire its named arms`,
    );
    detectedEscapeCells.push(detected);
    cellArmReceipts.push(
      `${detected.functionName}:${detected.escape}/${detected.origin}/${detected.comparison}/${detected.position}`,
    );
  }

  assertGeneratedPairCoverage(detectedFullCells, detectedEscapeCells);
  if (process.env.OMENA_IDENTIFIER_AUTHORITY_EMIT_CELL_ARMS === "1") {
    process.stdout.write(
      cellArmReceipts.map((receipt) => `generatedCellArm=${receipt}\n`).join(""),
    );
  }
  process.stderr.write(`generatedFullProductCellCount=${detectedFullCells.length}\n`);
  process.stderr.write(`generatedEscapeCoveringCellCount=${detectedEscapeCells.length}\n`);
  process.stderr.write("generatedPairFamilies=6/6\n");
  process.stderr.write(`generatedEscapeEntryPointCount=${derivedEscapes.length}\n`);
  process.stderr.write(
    `generatedCellArmDigest=${digest([...detectedFullCells, ...detectedEscapeCells])}\n`,
  );
}

function derivedEscapeAxisFromWorkspaceManifests(): readonly AuthoredEscapeId[] {
  const manifestResult = spawnSync(
    "git",
    ["ls-files", "rust/Cargo.toml", "rust/crates/*/Cargo.toml"],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );
  assert.equal(
    manifestResult.status,
    0,
    `failed to enumerate Rust manifests: ${(manifestResult.stderr ?? "").trim()}`,
  );
  const manifestText = manifestResult.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((relativePath) => readFileSync(path.join(repoRoot, relativePath), "utf8"))
    .join("\n");
  const available = new Set<AuthoredEscapeId>([
    "write-into-call",
    "write-into-ufcs",
    "write-into-fn-pointer",
    "render-authored-helper",
    "debug-format-spec",
    "debug-fmt-ufcs",
    "dbg-macro",
    "format-args-debug",
    "tracing-debug-sigil",
    "aliased-escape-path",
  ]);
  if (/\bserde\s*(?:=|\.)/u.test(manifestText)) {
    available.add("serialize-method-call");
    available.add("serializer-impl");
  }
  if (/\bserde_json\s*(?:=|\.)/u.test(manifestText)) {
    for (const id of [
      "serde-json-to-string",
      "serde-json-to-string-pretty",
      "serde-json-to-vec",
      "serde-json-to-vec-pretty",
      "serde-json-to-writer",
      "serde-json-to-writer-pretty",
      "serde-json-to-value",
      "serde-json-value-to-string",
      "json-macro",
      "serde-json-serializer",
      "serde-json-value-serializer",
    ] as const) {
      available.add(id);
    }
  }
  if (/\bserde_yaml_ng\s*(?:=|\.)/u.test(manifestText)) {
    for (const id of [
      "serde-yaml-to-string",
      "serde-yaml-to-writer",
      "serde-yaml-to-value",
      "serde-yaml-serializer",
      "serde-yaml-value-serializer",
    ] as const) {
      available.add(id);
    }
  }
  if (/\btoml\s*(?:=|\.)/u.test(manifestText)) {
    for (const id of [
      "toml-to-string",
      "toml-to-string-pretty",
      "toml-value-to-string",
      "toml-serializer",
      "toml-value-serializer",
    ] as const) {
      available.add(id);
    }
  }
  if (/\bserde-wasm-bindgen\s*(?:=|\.)/u.test(manifestText)) {
    available.add("serde-wasm-bindgen-to-value");
    available.add("serde-wasm-bindgen-serializer");
  }
  if (/\bnapi\s*(?:=|\.)/u.test(manifestText)) available.add("napi-serde-egress");
  if (injectGeneratedEntryPointIdDeletion) available.delete("serde-json-to-string");
  return authoredEscapeAxis.filter((id) => available.has(id));
}

interface GeneratedFunctionBody {
  readonly functionName: string;
  readonly path: string;
  readonly source: string;
  readonly scannable: string;
}

function generatedFixtureFunctions(
  sources: readonly MutableRustSource[],
): ReadonlyMap<string, GeneratedFunctionBody> {
  const functions = new Map<string, GeneratedFunctionBody>();
  for (const { relativePath, source } of sources) {
    const scannableSource = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannableSource)) {
      if (!functionSlice.name.startsWith("generated_property_")) continue;
      assert.ok(!functions.has(functionSlice.name), `duplicate function ${functionSlice.name}`);
      functions.set(functionSlice.name, {
        functionName: functionSlice.name,
        path: relativePath,
        source: source.slice(functionSlice.bodyStart, functionSlice.bodyEnd),
        scannable: functionSlice.scannable,
      });
    }
  }
  return functions;
}

function detectGeneratedFixtureCell(
  body: GeneratedFunctionBody,
  expectedEscape?: AuthoredEscapeId,
): GeneratedFixtureCell & { readonly escape?: AuthoredEscapeId } {
  const origin = detectGeneratedOrigin(body.scannable);
  const comparison = detectGeneratedComparison(body.scannable);
  const position = detectGeneratedPosition(body);
  assert.ok(origin, `${body.functionName}: no authored origin arm fired`);
  assert.ok(comparison, `${body.functionName}: no comparison arm fired`);
  const detected: GeneratedFixtureCell & { escape?: AuthoredEscapeId } = {
    functionName: body.functionName,
    origin,
    comparison,
    position,
  };
  if (expectedEscape) {
    const detectedEscapes = new Set(
      escapeCandidatesInBody(body.source, rustEscapeAliases(body.source)).map(
        (candidate) => candidate.escapeId,
      ),
    );
    assert.ok(
      detectedEscapes.has(expectedEscape),
      `${body.functionName}: no ${expectedEscape} escape arm fired`,
    );
    detected.escape = expectedEscape;
  }
  return detected;
}

function detectGeneratedOrigin(scannable: string): AuthoredOriginId | undefined {
  if (/\blet\s*\(\s*authored\s*,\s*_\s*\)/u.test(scannable)) return "tuple-destructuring";
  if (/\btuple_carrier\s*\.\s*0\b/u.test(scannable)) return "tuple-field-access";
  if (/\bfor\s+carrier\s+in\s+carriers\b/u.test(scannable))
    return injectGeneratedForLoopResolverDeletion ? undefined : "for-loop-binding";
  if (/\bwhile\s+let\s+Some\s*\(\s*carrier\s*\)/u.test(scannable)) return "while-let-binding";
  if (/(?:^|[;{}])\s*let\s+Some\s*\(\s*carrier\s*\)\s*=.*\belse\b/su.test(scannable))
    return "let-else-binding";
  if (/\bif\s+let\s+Some\s*\(\s*carrier\s*\).*&&\s*let\b/su.test(scannable))
    return "let-chain-binding";
  if (/\bif\s+let\s+Some\s*\(\s*carrier\s*\)/u.test(scannable)) return "if-let-binding";
  if (/\bmatch\s+carriers\s*\.\s*first\s*\(\s*\)/u.test(scannable)) return "match-arm-binding";
  if (/\bauthored_carrier\s*@\s*_/u.test(scannable)) return "at-binding";
  if (/Some\s*\(\s*bound\s*\)\s*\|\s*None/u.test(scannable)) return "or-pattern-binding";
  if (/\{\s*property\s*:\s*authored\s*\}\s*=\s*carrier/u.test(scannable))
    return "struct-destructuring";
  if (/\[\s*first\s*,\s*\.\.\s*\]\s*=>/u.test(scannable)) return "slice-pattern";
  if (/\bcarriers\s*\.\s*iter\s*\(\s*\)\s*\.\s*map\s*\(\s*\|\s*carrier\s*\|/u.test(scannable))
    return "closure-inferred";
  if (
    /\blet\s+first\s*=\s*&\s*carrier\s*\.\s*property\s*;\s*let\s+authored\s*=\s*first/u.test(
      scannable,
    )
  )
    return "two-statement-local";
  if (/\bauthored_wrapper_two\s*\(\s*authored_wrapper_one\s*\(/u.test(scannable))
    return "two-step-wrapper";
  if (/\bAuthoredAccess\s*::\s*authored\s*\(/u.test(scannable)) return "trait-method-return";
  if (/\bcarrier\s*\.\s*authored\s*\(\s*\)/u.test(scannable)) return "accessor-return-inline";
  if (/\bauthored_from_macro\s*!\s*\(/u.test(scannable)) return "macro-rules-body";
  if (/\bnamed_authored_escape\s*\(/u.test(scannable)) return "named-escape";
  if (/\bauthored_wrapper\s*\(/u.test(scannable)) return "wrapper-function";
  if (/\bgeneric_property\s*\.\s*as_ref\s*\(\s*\)/u.test(scannable))
    return "generic-bound-parameter";
  if (/\bself_value\s*\.\s*property\b/u.test(scannable)) return "self-receiver";
  if (/\blet\s+authored\s*=\s*fqn_property\s*;/u.test(scannable)) return "fqn-parameter";
  if (/\blet\s+authored\s*=\s*alias_property\s*;/u.test(scannable)) return "alias-parameter";
  if (/\blet\s+authored\s*=\s*property\s*;/u.test(scannable)) return "bare-parameter";
  if (/\bcarrier\s*\.\s*property\b/u.test(scannable)) return "field-access";
  return undefined;
}

function detectGeneratedComparison(scannable: string): AuthoredComparisonId | undefined {
  const orderedPatterns: readonly (readonly [AuthoredComparisonId, RegExp])[] = [
    ["argument-position-compare", /\bcompare_argument_position\s*\(/u],
    ["depth-two-return-compare", /\bcompare_depth_two\s*\(\s*pass_depth_one\s*\(/u],
    [
      "map-to-string-collect-sort",
      /\.\s*map\s*\(\s*ToString\s*::\s*to_string\s*\).*\.\s*sort\s*\(/su,
    ],
    ["write-into-buffer-compare", /\bWrite\s*::\s*write_str\s*\(/u],
    ["derived-ord-sort", /DerivedPropertyText\s*\([^)]*\).*\.\s*sort\s*\(/su],
    ["macro-rules-compare", /\braw_property_compare\s*!\s*\(/u],
    ["manual-hash-newtype", /ManualPropertyText\s*\([^)]*\)\s*\.\s*hash\s*\(/u],
    ["manual-partialeq-newtype", /ManualPropertyText\s*\([^)]*\)\s*\.\s*eq\s*\(/u],
    ["entry-format-key", /\.\s*entry\s*\(\s*format\s*!\s*\(/u],
    ["trim-matches-normalize", /\.\s*trim_matches\s*\(/u],
    ["strip-prefix-normalize", /\.\s*strip_prefix\s*\(/u],
    ["to-ascii-lowercase-fold", /\.\s*to_ascii_lowercase\s*\(/u],
    ["to-uppercase-fold", /\.\s*to_uppercase\s*\(/u],
    ["to-lowercase-fold", /\.\s*to_lowercase\s*\(/u],
    ["chars-eq", /\.\s*chars\s*\(\s*\)\s*\.\s*eq\s*\(/u],
    ["bytes-eq", /\.\s*bytes\s*\(\s*\)\s*\.\s*eq\s*\(/u],
    ["len-and-starts-with", /\.\s*len\s*\(\s*\)\s*==.*\.\s*starts_with\s*\(/su],
    ["sort-by-cached-key", /\.\s*sort_by_cached_key\s*\(/u],
    ["sort-unstable-by-key", /\.\s*sort_unstable_by_key\s*\(/u],
    ["sort-unstable-by", /\.\s*sort_unstable_by\s*\(/u],
    ["sort-unstable", /\.\s*sort_unstable\s*\(/u],
    ["sort-by-key", /\.\s*sort_by_key\s*\(/u],
    ["sort-by", /\.\s*sort_by\s*\(/u],
    ["dedup-by-key", /\.\s*dedup_by_key\s*\(/u],
    ["dedup-by", /\.\s*dedup_by\s*\(/u],
    ["dedup", /\.\s*dedup\s*\(/u],
    ["binary-search-by-key", /\.\s*binary_search_by_key\s*\(/u],
    ["binary-search-by", /\.\s*binary_search_by\s*\(/u],
    ["binary-search", /\.\s*binary_search\s*\(/u],
    ["map-contains-key", /\.\s*contains_key\s*\(/u],
    ["map-entry", /\.\s*entry\s*\(/u],
    ["map-get", /BTreeMap[^;]*;.*\.\s*get\s*\(/su],
    ["map-remove", /BTreeMap[^;]*;.*\.\s*remove\s*\(/su],
    ["map-insert", /BTreeMap[^;]*;.*\.\s*insert\s*\([^;]*;\s*true/su],
    ["set-get", /BTreeSet[^;]*;.*\.\s*get\s*\(/su],
    ["set-contains", /BTreeSet[^;]*;.*\.\s*contains\s*\(/su],
    ["set-remove", /BTreeSet[^;]*;.*\.\s*remove\s*\(/su],
    ["set-insert", /BTreeSet[^;]*;.*\.\s*insert\s*\([^;]*;\s*true/su],
    ["partial-cmp-is-eq", /\.\s*partial_cmp\s*\(.*?\)\s*\.\s*is_some_and/su],
    ["cmp-is-eq", /\.\s*cmp\s*\(.*?\)\s*\.\s*is_eq/su],
    ["eq-ignore-ascii-case", /\.\s*eq_ignore_ascii_case\s*\(/u],
    ["ufcs-str-eq", /\bstr\s*::\s*eq\s*\(/u],
    ["ufcs-partial-eq", /\bPartialEq\s*::\s*eq\s*\(/u],
    ["method-ne", /\bvalue\s*\.\s*ne\s*\(/u],
    ["method-eq", /\bvalue\s*\.\s*eq\s*\(/u],
    ["matches-literal", /\bmatches\s*!\s*\(\s*value/u],
    ["match-literal", /\bmatch\s+value\s*\.\s*as_str/u],
    ["binary-ne", /\bvalue\s*!=/u],
    ["binary-eq", /\bvalue\s*==/u],
    ["sort", /\.\s*sort\s*\(/u],
  ];
  for (const [id, pattern] of orderedPatterns) {
    if (id === "argument-position-compare" && injectGeneratedArgumentReturnEdgeDeletion) continue;
    if (pattern.test(scannable)) return id;
  }
  return undefined;
}

function detectGeneratedPosition(body: GeneratedFunctionBody): AuthoredPositionId {
  if (body.path.endsWith("generated_property_matrix_ranking.rs")) return "cross-file";
  if (body.path.endsWith("generated_property_matrix_2.rs")) return "authority-zero-file";
  return "same-file";
}

function assertGeneratedPairCoverage(
  fullCells: readonly GeneratedFixtureCell[],
  escapeCells: readonly (GeneratedFixtureCell & { readonly escape: AuthoredEscapeId })[],
): void {
  const pairSet = (rows: readonly unknown[][]): ReadonlySet<string> =>
    new Set(rows.map((row) => JSON.stringify(row)));
  assert.equal(
    pairSet(fullCells.map((cell) => [cell.origin, cell.comparison])).size,
    authoredOriginAxis.length * authoredComparisonAxis.length,
    "origin x comparison coverage",
  );
  assert.equal(
    pairSet(fullCells.map((cell) => [cell.origin, cell.position])).size,
    authoredOriginAxis.length * authoredPositionAxis.length,
    "origin x position coverage",
  );
  assert.equal(
    pairSet(fullCells.map((cell) => [cell.comparison, cell.position])).size,
    authoredComparisonAxis.length * authoredPositionAxis.length,
    "comparison x position coverage",
  );
  assert.equal(
    pairSet(escapeCells.map((cell) => [cell.escape, cell.origin])).size,
    authoredEscapeAxis.length * authoredOriginAxis.length,
    "escape x origin coverage",
  );
  assert.equal(
    pairSet(escapeCells.map((cell) => [cell.escape, cell.comparison])).size,
    authoredEscapeAxis.length * authoredComparisonAxis.length,
    "escape x comparison coverage",
  );
  assert.equal(
    pairSet(escapeCells.map((cell) => [cell.escape, cell.position])).size,
    authoredEscapeAxis.length * authoredPositionAxis.length,
    "escape x position coverage",
  );
}

interface AuthoredEscapeClosureAuditResult {
  readonly escapeSites: readonly AuthoredEscapeSite[];
  readonly identityFlows: readonly AuthoredEscapeIdentityFlow[];
  readonly writeIntoSites: readonly WriteIntoSinkSite[];
  readonly unresolvedCallEdges: readonly CensusSite[];
  readonly carrierAudit: readonly AuthoredIdentityCarrierAudit[];
  readonly identityDeriveViolationCount: number;
  readonly identityDeriveViolations: readonly string[];
}

interface EscapeOccurrenceDraft {
  readonly siteIndex: number;
  readonly path: string;
  readonly line: number;
  readonly functionName: string;
  readonly escapeId: AuthoredEscapeId;
  readonly evidence: string;
  readonly matchedText: string;
  readonly operandClasses: readonly AuthoredEscapeOperandClass[];
  readonly operandDerivation: string;
  readonly resultBindings: readonly string[];
  readonly sinkBinding?: string;
}

interface EscapeFunctionRecord {
  readonly crateName: string;
  readonly path: string;
  readonly source: string;
  readonly functionName: string;
  readonly signature: string;
  readonly body: string;
  readonly scannable: string;
  readonly bodyStart: number;
  readonly bindings: ReadonlyMap<string, string>;
  readonly occurrences: readonly EscapeOccurrenceDraft[];
}

interface FunctionTaintResult {
  readonly bindingSources: ReadonlyMap<string, ReadonlySet<number>>;
  readonly returnSources: ReadonlySet<number>;
  readonly unresolvedEdges: readonly CensusSite[];
}

function discoverAuthoredEscapeClosureAudit(): AuthoredEscapeClosureAuditResult {
  const fixtureCrates = new Set(["omena-diff-test", "engine-style-parser", "omena-benchmarks"]);
  const sources: MutableRustSource[] = productionSources
    .filter((relativePath) => !fixtureCrates.has(relativePath.split("/")[2] ?? ""))
    .map((relativePath) => ({
      relativePath,
      source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
    }));
  applyAuthoredEscapeProbeMutations(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  const functionReturnTypes = uniqueRustFunctionReturnTypes(sources, aliasesByPath);
  const records: EscapeFunctionRecord[] = [];
  const occurrenceDrafts: EscapeOccurrenceDraft[] = [];
  for (const { relativePath, source } of sources) {
    const scannableSource = maskCommentsStringsAndTestItems(source, false);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    const useAliases = rustEscapeAliases(source);
    for (const functionSlice of rustFunctionSlices(scannableSource)) {
      const body = source.slice(functionSlice.bodyStart, functionSlice.bodyEnd);
      const bindings = inferredEscapeBindings(
        functionSlice,
        source,
        aliases,
        structFields,
        relativePath.split("/")[2] ?? "<unknown-crate>",
        functionReturnTypes,
      );
      const occurrences = escapeOccurrencesInFunction(
        relativePath,
        source,
        functionSlice,
        body,
        bindings,
        structFields,
        useAliases,
        occurrenceDrafts.length,
      );
      occurrenceDrafts.push(...occurrences);
      records.push({
        crateName: relativePath.split("/")[2] ?? "<unknown-crate>",
        path: relativePath,
        source,
        functionName: functionSlice.name,
        signature: functionSlice.signature,
        body,
        scannable: functionSlice.scannable,
        bodyStart: functionSlice.bodyStart,
        bindings,
        occurrences,
      });
    }
  }

  const definitionCounts = new Map<string, number>();
  for (const record of records) {
    const key = `${record.crateName}\0${record.functionName}`;
    definitionCounts.set(key, (definitionCounts.get(key) ?? 0) + 1);
  }
  const returnSummaries = new Map<string, ReadonlySet<number>>();
  let taintByFunction = new Map<EscapeFunctionRecord, FunctionTaintResult>();
  for (let pass = 0; pass <= records.length; pass += 1) {
    let changed = false;
    const nextTaint = new Map<EscapeFunctionRecord, FunctionTaintResult>();
    for (const record of records) {
      const taint = computeFunctionTaint(record, returnSummaries, definitionCounts);
      nextTaint.set(record, taint);
      const key = `${record.crateName}\0${record.functionName}`;
      const prior = returnSummaries.get(key) ?? new Set<number>();
      const merged = new Set([...prior, ...taint.returnSources]);
      if (merged.size !== prior.size) {
        returnSummaries.set(key, merged);
        changed = true;
      }
    }
    taintByFunction = nextTaint;
    if (!changed) break;
    assert.ok(pass < records.length, "escape taint fixpoint exceeded the finite function set");
  }

  const flows: AuthoredEscapeIdentityFlow[] = [];
  const flowSourceIndexes = new Set<number>();
  for (const record of records) {
    const taint = taintByFunction.get(record);
    if (!taint) continue;
    for (const statement of rustStatementSlices(record.body)) {
      const absoluteOffset = record.bodyStart + statement.start;
      const flowLine = lineNumberAt(record.source, absoluteOffset);
      const sourceIndexes = new Set(
        [
          ...taintedSourcesForText(
            statement.text,
            taint.bindingSources,
            record.crateName,
            returnSummaries,
            definitionCounts,
            record.occurrences,
          ),
        ].filter((index) => {
          const sourceSite = occurrenceDrafts[index];
          return (
            sourceSite === undefined ||
            sourceSite.path !== record.path ||
            sourceSite.functionName !== record.functionName ||
            sourceSite.line <= flowLine
          );
        }),
      );
      if (sourceIndexes.size === 0) continue;
      const comparisonId = productionComparisonId(statement.text, taint.bindingSources);
      if (!comparisonId) continue;
      const sourceSites = [...sourceIndexes]
        .map((index) => occurrenceDrafts[index])
        .filter((site): site is EscapeOccurrenceDraft => site !== undefined);
      const sanctioned = sourceSites.every(
        (site) => !site.operandClasses.includes("authored-bearing"),
      );
      for (const index of sourceIndexes) flowSourceIndexes.add(index);
      flows.push({
        path: record.path,
        line: flowLine,
        function: record.functionName,
        comparisonId,
        escapeIds: [...new Set(sourceSites.map((site) => site.escapeId))].toSorted(),
        evidence: evidenceLine(record.source, absoluteOffset),
        operandDerivation: sourceSites.map((site) => site.operandDerivation).join("; "),
        sanctioned,
      });
    }
  }

  const escapeSites: AuthoredEscapeSite[] = occurrenceDrafts.map((site) => {
    const flowsForSite = flows.filter(
      (flow) =>
        flow.escapeIds.includes(site.escapeId) &&
        flow.operandDerivation.includes(site.operandDerivation),
    );
    const reachesIdentity = flowSourceIndexes.has(site.siteIndex);
    const sanctioned =
      reachesIdentity &&
      !site.operandClasses.includes("authored-bearing") &&
      flowsForSite.length > 0;
    const disposition: AuthoredEscapeDisposition = reachesIdentity
      ? sanctioned
        ? "sanctioned-identity"
        : "identity-violation"
      : /(?:serializer|serialize|serde|toml|napi|wasm)/u.test(site.escapeId)
        ? "egress"
        : "presentation";
    return {
      path: site.path,
      line: site.line,
      function: site.functionName,
      escapeId: site.escapeId,
      evidence: site.evidence,
      operandClasses: site.operandClasses,
      operandDerivation: site.operandDerivation,
      disposition,
      resultBindings: site.resultBindings,
    };
  });
  const writeIntoSites = occurrenceDrafts
    .filter(
      (site): site is EscapeOccurrenceDraft & { sinkBinding: string } =>
        site.sinkBinding !== undefined &&
        (site.escapeId === "write-into-call" ||
          site.escapeId === "write-into-ufcs" ||
          site.escapeId === "write-into-fn-pointer" ||
          site.escapeId === "render-authored-helper"),
    )
    .map((site) => {
      const record = records.find(
        (candidate) => candidate.path === site.path && candidate.functionName === site.functionName,
      );
      assert.ok(record, `write_into record missing for ${site.path}:${site.functionName}`);
      const taint = taintByFunction.get(record);
      assert.ok(taint, `write_into taint summary missing for ${site.path}:${site.functionName}`);
      const sinkClass = classifyWriteIntoSink(site, taint, flowSourceIndexes);
      return {
        path: site.path,
        line: site.line,
        function: site.functionName,
        escapeId: site.escapeId,
        sinkClass,
        sinkBinding: site.sinkBinding,
        evidence: site.evidence,
        derivation: `sink=${site.sinkBinding}; class=${sinkClass}; function-return=${escapeFunctionReturnType(record.signature) ?? "unit"}`,
      } satisfies WriteIntoSinkSite;
    });
  const unresolvedCallEdges = uniqueSites(
    [...taintByFunction.values()].flatMap((taint) => taint.unresolvedEdges),
  ).map((site) => ({
    ...site,
    disposition: "named-exempt" as const,
    reason: "Text-only resolver cannot prove this tainted argument-to-return edge intra-crate.",
  }));
  const carrierResult = discoverAuthoredIdentityCarrierAudit(sources, structFields);
  return {
    escapeSites: escapeSites.toSorted(authoredEscapeSiteOrder),
    identityFlows: flows.toSorted(authoredEscapeFlowOrder),
    writeIntoSites: writeIntoSites.toSorted(writeIntoSiteOrder),
    unresolvedCallEdges,
    carrierAudit: carrierResult.audit,
    identityDeriveViolationCount: carrierResult.deriveViolationCount,
    identityDeriveViolations: carrierResult.deriveViolations,
  };
}

function assertReferenceSanctionedEscapeSites(sites: readonly AuthoredEscapeSite[]): void {
  const pairIdentity = sites.find(
    (site) =>
      site.path === "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs" &&
      site.function === "pair_identity" &&
      site.escapeId === "debug-format-spec",
  );
  assert.ok(pairIdentity, "pair_identity sanctioned escape reference site must remain visible");
  assert.deepEqual(
    pairIdentity.operandClasses,
    ["key-bearing", "non-property"],
    "pair_identity must derive only key-bearing and non-property operands",
  );
  assert.equal(
    pairIdentity.disposition,
    "sanctioned-identity",
    "pair_identity must remain a sanctioned identity escape",
  );

  const elementSignature = sites.find(
    (site) =>
      site.path === "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs" &&
      site.function === "guarded_winner_function_equality_for_pair" &&
      site.escapeId === "debug-format-spec" &&
      site.evidence.includes("let element_signature = format!"),
  );
  assert.ok(
    elementSignature,
    "element_signature sanctioned escape reference site must remain visible",
  );
  assert.deepEqual(
    elementSignature.operandClasses,
    ["non-property"],
    "element_signature must derive only a non-property operand",
  );
  assert.equal(
    elementSignature.disposition,
    "sanctioned-identity",
    "element_signature must remain a sanctioned identity escape",
  );
}

function applyAuthoredEscapeProbeMutations(sources: MutableRustSource[]): void {
  const targetPath = "rust/crates/omena-cascade/src/axis_order.rs";
  if (injectAuthoredWrapperEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      'fn injected_authored_wrapper_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); escaped == "--probe" }',
      targetPath,
    );
  }
  if (injectAuthoredContainerEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      'fn injected_authored_container_escape_identity(properties: &Vec<omena_syntax::ident::AuthoredPropertyTextV0>) -> bool { let escaped = serde_json::to_string(properties).unwrap(); escaped == "--probe" }',
      targetPath,
    );
  }
  if (injectPropertyNameEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      'fn injected_property_name_escape_identity(property: &omena_syntax::ident::PropertyNameV0) -> bool { let escaped = format!("{property:?}"); escaped == "--probe" }',
      targetPath,
    );
  }
  if (injectSanctionedEscapeInventory) {
    appendTrackedSourceMutation(
      sources,
      'fn injected_sanctioned_escape_inventory(value: &String) -> bool { let escaped = serde_json::to_string(value).unwrap(); escaped == "probe" }',
      targetPath,
    );
  }
  if (injectWriteIntoInventory) {
    appendTrackedSourceMutation(
      sources,
      'fn injected_write_into_inventory(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> String { let mut output = String::new(); property.write_into(&mut output).expect("fmt to String cannot fail"); output }',
      targetPath,
    );
  }
  if (injectDerivedCarrierIdentity) {
    appendTrackedSourceMutation(
      sources,
      "#[derive(PartialEq, Eq)]\nstruct InjectedDerivedAuthoredCarrier { property: omena_syntax::ident::AuthoredPropertyTextV0 }",
      targetPath,
    );
  }
  if (injectRawSelectorDefinitionSort) {
    replaceTrackedSourceMutation(
      sources,
      "rust/crates/omena-query/src/style/source_refs.rs",
      "canonical_class_key(definition.name.as_str()),",
      "definition.name.clone(),",
      "selector definition sort mutation target",
    );
  }
  if (injectRawTransformNodeSort) {
    replaceTrackedSourceMutation(
      sources,
      "rust/crates/omena-transform-cst/src/lib.rs",
      "left.semantic_key.cmp(&right.semantic_key)",
      "left.label.cmp(&right.label)",
      "transform node sort mutation target",
    );
  }
}

function replaceTrackedSourceMutation(
  sources: MutableRustSource[],
  relativePath: string,
  before: string,
  after: string,
  label: string,
): void {
  const target = sources.find((source) => source.relativePath === relativePath);
  assert.ok(target, `${label} must be in census scope`);
  assert.equal(target.source.split(before).length - 1, 1, `${label} must be unique`);
  target.source = target.source.replace(before, after);
}

function rustEscapeAliases(source: string): ReadonlyMap<string, AuthoredEscapeId> {
  const aliases = new Map<string, AuthoredEscapeId>();
  for (const match of source.matchAll(
    /\buse\s+((?:serde_json|serde_yaml_ng|toml|serde_wasm_bindgen)(?:::[A-Za-z_][A-Za-z0-9_]*)+)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
  )) {
    const id = escapeIdForQualifiedPath(match[1]);
    if (id) aliases.set(match[2], "aliased-escape-path");
  }
  return aliases;
}

function escapeIdForQualifiedPath(qualifiedPath: string): AuthoredEscapeId | undefined {
  const pathToId = new Map<string, AuthoredEscapeId>([
    ["serde_json::to_string", "serde-json-to-string"],
    ["serde_json::to_string_pretty", "serde-json-to-string-pretty"],
    ["serde_json::to_vec", "serde-json-to-vec"],
    ["serde_json::to_vec_pretty", "serde-json-to-vec-pretty"],
    ["serde_json::to_writer", "serde-json-to-writer"],
    ["serde_json::to_writer_pretty", "serde-json-to-writer-pretty"],
    ["serde_json::to_value", "serde-json-to-value"],
    ["serde_yaml_ng::to_string", "serde-yaml-to-string"],
    ["serde_yaml_ng::to_writer", "serde-yaml-to-writer"],
    ["serde_yaml_ng::to_value", "serde-yaml-to-value"],
    ["toml::to_string", "toml-to-string"],
    ["toml::to_string_pretty", "toml-to-string-pretty"],
    ["serde_wasm_bindgen::to_value", "serde-wasm-bindgen-to-value"],
  ]);
  return pathToId.get(qualifiedPath);
}

function inferredEscapeBindings(
  functionSlice: RustFunctionSlice,
  source: string,
  aliases: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  crateName: string,
  functionReturnTypes: ReadonlyMap<string, string>,
): ReadonlyMap<string, string> {
  const bindings = new Map(
    resolvedRustBindingsForFunction(functionSlice, source, aliases, structFields),
  );
  const callableReturns = new Map<string, string>();
  for (const parameter of functionSlice.signature.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*impl\s+Fn(?:Once|Mut)?\s*\([^)]*\)\s*->\s*([^,\n{]+)/gu,
  )) {
    callableReturns.set(parameter[1], normalizeRustType(parameter[2], aliases));
  }
  for (const binding of functionSlice.scannable.matchAll(
    /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);/gu,
  )) {
    const expression = binding[2];
    const calledBinding = expression.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/u)?.[1];
    const inferred = /\.as_str\s*\(\s*\)\s*$/u.test(expression)
      ? "str"
      : calledBinding && callableReturns.has(calledBinding)
        ? callableReturns.get(calledBinding)
        : calledBinding && functionReturnTypes.has(`${crateName}\0${calledBinding}`)
          ? functionReturnTypes.get(`${crateName}\0${calledBinding}`)
          : /\.(?:to_custom_key|canonical_custom_key)\s*\(/u.test(expression)
            ? "CanonicalCustomPropertyNameV0"
            : /\.(?:to_standard_key|canonical_standard_key)\s*\(/u.test(expression)
              ? "CanonicalStandardPropertyNameV0"
              : /\.canonical_key\s*\(/u.test(expression)
                ? "CanonicalPropertyKeyV0"
                : /(?:PropertyNameV0\s*::\s*(?:new|from_authored|custom|standard)|\.to_property_name)\s*\(/u.test(
                      expression,
                    )
                  ? "PropertyNameV0"
                  : /(?:String\s*::\s*new|format\s*!|\.to_string\s*\()/u.test(expression)
                    ? "String"
                    : undefined;
    if (inferred) bindings.set(binding[1], inferred);
  }
  for (const binding of functionSlice.scannable.matchAll(
    /\b(?:if|while)\s+let\s+Some\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    const returnType = functionReturnTypes.get(`${crateName}\0${binding[2]}`);
    if (returnType) bindings.set(binding[1], returnType);
  }
  for (const binding of functionSlice.scannable.matchAll(
    /\blet\s+Some\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    const returnType = functionReturnTypes.get(`${crateName}\0${binding[2]}`);
    if (returnType) bindings.set(binding[1], returnType);
  }
  for (const closure of functionSlice.scannable.matchAll(
    /\btoml\s*::\s*from_str\s*::\s*<\s*([^>]+)\s*>[\s\S]*?\.and_then\s*\(\s*\|\s*([A-Za-z_][A-Za-z0-9_]*)\s*\|/gu,
  )) {
    bindings.set(closure[2], normalizeRustType(closure[1], aliases));
  }
  return bindings;
}

function uniqueRustFunctionReturnTypes(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, string> {
  const candidates = new Map<string, Set<string>>();
  for (const { relativePath, source } of sources) {
    const crateName = relativePath.split("/")[2] ?? "<unknown-crate>";
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const returnType = escapeFunctionReturnType(functionSlice.signature);
      if (!returnType) continue;
      const key = `${crateName}\0${functionSlice.name}`;
      const types = candidates.get(key) ?? new Set<string>();
      types.add(normalizeRustType(returnType, aliases));
      candidates.set(key, types);
    }
  }
  return new Map(
    [...candidates]
      .filter(([, types]) => types.size === 1)
      .map(([key, types]) => [key, [...types][0]] as const),
  );
}

interface EscapeCandidateDraft {
  readonly offset: number;
  readonly escapeId: AuthoredEscapeId;
  readonly matchedText: string;
  readonly operands: readonly string[];
  readonly sinkBinding?: string;
}

function escapeCandidatesInBody(
  body: string,
  useAliases: ReadonlyMap<string, AuthoredEscapeId>,
  functionSlice?: RustFunctionSlice,
): readonly EscapeCandidateDraft[] {
  const candidates: EscapeCandidateDraft[] = [];
  const addMatches = (
    escapeId: AuthoredEscapeId,
    pattern: RegExp,
    operands: (match: RegExpMatchArray) => readonly string[],
    sink?: (match: RegExpMatchArray) => string | undefined,
  ): void => {
    for (const match of body.matchAll(pattern)) {
      candidates.push({
        offset: match.index,
        escapeId,
        matchedText: match[0],
        operands: operands(match),
        sinkBinding: sink?.(match),
      });
    }
  };
  addMatches(
    "render-authored-helper",
    /\b(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*render_authored\s*\(\s*&?\s*([^,]+?)\s*,\s*&mut\s+([A-Za-z_][A-Za-z0-9_]*)/gu,
    (match) => [match[1]],
    (match) => match[2],
  );
  for (const pointer of body.matchAll(
    /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*AuthoredPropertyTextV0\s*::\s*write_into\s*;/gu,
  )) {
    const pattern = new RegExp(
      `\\b${escapeRegExp(pointer[1])}\\s*\\(\\s*&?\\s*([^,]+?)\\s*,\\s*&mut\\s+([A-Za-z_][A-Za-z0-9_]*)`,
      "gu",
    );
    addMatches(
      "write-into-fn-pointer",
      pattern,
      (match) => [match[1]],
      (match) => match[2],
    );
  }
  addMatches(
    "write-into-ufcs",
    /\bAuthoredPropertyTextV0\s*::\s*write_into\s*\(\s*&?\s*([^,]+?)\s*,\s*&mut\s+([A-Za-z_][A-Za-z0-9_]*)/gu,
    (match) => [match[1]],
    (match) => match[2],
  );
  addMatches(
    "write-into-call",
    /\b([A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*(?:\s*\(\s*\))?)*)\s*\.\s*write_into\s*\(\s*&mut\s+([A-Za-z_][A-Za-z0-9_]*)/gu,
    (match) => [match[1]],
    (match) => match[2],
  );
  const simpleCallPatterns: readonly (readonly [AuthoredEscapeId, RegExp, number])[] = [
    [
      "serde-json-to-string-pretty",
      /\bserde_json\s*::\s*to_string_pretty\s*\(\s*&?\s*([^,)]+)/gu,
      1,
    ],
    ["serde-json-to-string", /\bserde_json\s*::\s*to_string\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["serde-json-to-vec-pretty", /\bserde_json\s*::\s*to_vec_pretty\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["serde-json-to-vec", /\bserde_json\s*::\s*to_vec\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["serde-json-to-value", /\bserde_json\s*::\s*to_value\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["serde-yaml-to-string", /\bserde_yaml_ng\s*::\s*to_string\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["serde-yaml-to-value", /\bserde_yaml_ng\s*::\s*to_value\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["toml-to-string-pretty", /\btoml\s*::\s*to_string_pretty\s*\(\s*&?\s*([^,)]+)/gu, 1],
    ["toml-to-string", /\btoml\s*::\s*to_string\s*\(\s*&?\s*([^,)]+)/gu, 1],
    [
      "serde-wasm-bindgen-to-value",
      /\bserde_wasm_bindgen\s*::\s*to_value\s*\(\s*&?\s*([^,)]+)/gu,
      1,
    ],
  ];
  for (const [id, pattern, operandIndex] of simpleCallPatterns) {
    addMatches(id, pattern, (match) => [match[operandIndex]]);
  }
  const writerPatterns: readonly (readonly [AuthoredEscapeId, RegExp])[] = [
    [
      "serde-json-to-writer-pretty",
      /\bserde_json\s*::\s*to_writer_pretty\s*\([^,]+,\s*&?\s*([^)]+)\)/gu,
    ],
    ["serde-json-to-writer", /\bserde_json\s*::\s*to_writer\s*\([^,]+,\s*&?\s*([^)]+)\)/gu],
    ["serde-yaml-to-writer", /\bserde_yaml_ng\s*::\s*to_writer\s*\([^,]+,\s*&?\s*([^)]+)\)/gu],
  ];
  for (const [id, pattern] of writerPatterns) addMatches(id, pattern, (match) => [match[1]]);
  addMatches(
    "serde-json-value-to-string",
    /\bserde_json\s*::\s*Value\s*::\s*to_string\s*\(\s*&?\s*([^)]+)/gu,
    (match) => [match[1]],
  );
  addMatches(
    "toml-value-to-string",
    /\btoml\s*::\s*Value\s*::\s*try_from\s*\(\s*&?\s*([^)]+)/gu,
    (match) => [match[1]],
  );
  addMatches("json-macro", /\bserde_json\s*::\s*json\s*!\s*\(\s*&?\s*([^)]+)\)/gu, (match) => [
    match[1],
  ]);
  addMatches(
    "serialize-method-call",
    /\b([A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)*)\s*\.\s*serialize\s*\(/gu,
    (match) => [match[1]],
  );
  const serializerPatterns: readonly (readonly [AuthoredEscapeId, RegExp])[] = [
    ["serde-json-serializer", /\bserde_json\s*::\s*Serializer\s*::\s*new\s*\(/gu],
    ["serde-json-value-serializer", /\bserde_json\s*::\s*value\s*::\s*Serializer\b/gu],
    ["serde-yaml-serializer", /\bserde_yaml_ng\s*::\s*Serializer\s*::\s*new\s*\(/gu],
    ["serde-yaml-value-serializer", /\bserde_yaml_ng\s*::\s*value\s*::\s*Serializer\b/gu],
    ["toml-value-serializer", /\btoml\s*::\s*ser\s*::\s*ValueSerializer\s*::\s*new\s*\(/gu],
    ["toml-serializer", /\btoml\s*::\s*Serializer\s*::\s*new\s*\(/gu],
    [
      "serde-wasm-bindgen-serializer",
      /\bserde_wasm_bindgen\s*::\s*Serializer\s*::\s*json_compatible\s*\(/gu,
    ],
  ];
  for (const [escapeId, constructorPattern] of serializerPatterns) {
    for (const constructor of body.matchAll(constructorPattern)) {
      const prefix = body.slice(0, constructor.index);
      const binding = prefix.match(
        /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*[^;]*$/u,
      )?.[1];
      if (binding) {
        const serializeCall = new RegExp(
          `\\b([A-Za-z_][A-Za-z0-9_]*(?:\\s*\\.\\s*[A-Za-z_][A-Za-z0-9_]*)*)\\s*\\.\\s*serialize\\s*\\(\\s*&?(?:mut\\s+)?${escapeRegExp(binding)}\\b`,
          "gu",
        );
        addMatches(escapeId, serializeCall, (match) => [match[1]]);
      }
      const directCall = new RegExp(
        `\\b([A-Za-z_][A-Za-z0-9_]*(?:\\s*\\.\\s*[A-Za-z_][A-Za-z0-9_]*)*)\\s*\\.\\s*serialize\\s*\\(\\s*&?${escapeRegExp(constructor[0]).replaceAll("\\(", "\\s*\\(")}`,
        "gu",
      );
      addMatches(escapeId, directCall, (match) => [match[1]]);
    }
  }
  addMatches("napi-serde-egress", /\bnapi_serde_egress\s*\(\s*&?\s*([^,)]+)/gu, (match) => [
    match[1],
  ]);
  addMatches("serializer-impl", /\bproduct_serializer_impl\s*\(\s*&?\s*([^,)]+)/gu, (match) => [
    match[1],
  ]);
  addMatches("dbg-macro", /\bdbg\s*!\s*\(\s*&?\s*([^)]+)\)/gu, (match) => [match[1]]);
  addMatches(
    "debug-fmt-ufcs",
    /\b(?:std\s*::\s*fmt\s*::\s*)?Debug\s*::\s*fmt\s*\(\s*&?\s*([^,]+)/gu,
    (match) => [match[1]],
  );
  addMatches(
    "tracing-debug-sigil",
    /\bdebug\s*=\s*\?\s*([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)/gu,
    (match) => [match[1]],
  );
  for (const match of body.matchAll(/\bformat(?:_args)?\s*!\s*\(([^;\n]*\?[^;\n]*)\)/gu)) {
    const evidence = match[1];
    const operands = debugFormatOperands(evidence);
    candidates.push({
      offset: match.index,
      escapeId: match[0].startsWith("format_args") ? "format-args-debug" : "debug-format-spec",
      matchedText: match[0],
      operands,
    });
  }
  for (const [alias] of useAliases) {
    const pattern = new RegExp(`\\b${escapeRegExp(alias)}\\s*\\(\\s*&?\\s*([^,)]+)`, "gu");
    addMatches("aliased-escape-path", pattern, (match) => [match[1]]);
  }
  if (
    functionSlice?.name === "serialize" &&
    /\b(?:serde\s*::\s*)?Serializer\b/u.test(functionSlice.signature)
  ) {
    const bodyAnchor = body.search(/\S/u);
    candidates.push({
      offset: Math.max(bodyAnchor, 0),
      escapeId: "serializer-impl",
      matchedText: body.slice(Math.max(bodyAnchor, 0), Math.max(bodyAnchor, 0) + 1),
      operands: ["self"],
    });
  }

  const deduped = new Map<string, EscapeCandidateDraft>();
  for (const candidate of candidates) {
    const key = `${candidate.offset}\0${candidate.escapeId}`;
    deduped.set(key, candidate);
  }
  return [...deduped.values()];
}

function escapeOccurrencesInFunction(
  relativePath: string,
  source: string,
  functionSlice: RustFunctionSlice,
  body: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  useAliases: ReadonlyMap<string, AuthoredEscapeId>,
  firstSiteIndex: number,
): EscapeOccurrenceDraft[] {
  const candidates = escapeCandidatesInBody(body, useAliases, functionSlice);
  return candidates.map((candidate, localIndex) => {
    const statement = rustStatementAt(body, candidate.offset);
    const resultBindings = resultBindingsForEscape(statement.text, candidate.sinkBinding);
    const operandClasses = candidate.operands.map((operand) =>
      classifyAuthoredEscapeOperand(operand, bindings, structFields),
    );
    const absoluteOffset = functionSlice.bodyStart + candidate.offset;
    return {
      siteIndex: firstSiteIndex + localIndex,
      path: relativePath,
      line: lineNumberAt(source, absoluteOffset),
      functionName: functionSlice.name,
      escapeId: candidate.escapeId,
      evidence: evidenceLine(source, absoluteOffset),
      matchedText: candidate.matchedText,
      operandClasses,
      operandDerivation: candidate.operands
        .map(
          (operand, index) => `${operand.trim()}=>${operandClasses[index] ?? "authored-bearing"}`,
        )
        .join(", "),
      resultBindings,
      sinkBinding: candidate.sinkBinding,
    };
  });
}

function debugFormatOperands(formatArguments: string): readonly string[] {
  const operands = new Set<string>();
  for (const interpolation of formatArguments.matchAll(
    /\{\s*([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)?\s*:[^}]*\?[^}]*\}/gu,
  )) {
    if (interpolation[1]) operands.add(interpolation[1]);
  }
  const afterLiteral = formatArguments.replace(/^\s*"(?:[^"\\]|\\.)*"\s*,?/u, "");
  for (const argument of afterLiteral.split(",")) {
    const candidate = argument
      .trim()
      .replace(/^&/u, "")
      .match(/^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*/u)?.[0];
    if (candidate) operands.add(candidate);
  }
  if (operands.size === 0) operands.add("<unresolved-debug-operand>");
  return [...operands];
}

function classifyAuthoredEscapeOperand(
  expression: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): AuthoredEscapeOperandClass {
  const normalized = expression.trim().replace(/^&\s*/u, "");
  if (/^(?:true|false|None|Some\s*\(|[0-9]+|"|b?")/u.test(normalized)) return "non-property";
  if (/\.(?:canonical_name|as_str)\s*\(/u.test(normalized)) return "non-property";
  if (/\.(?:to_custom_key|to_standard_key|canonical_key)\s*\(/u.test(normalized))
    return "key-bearing";
  if (/\.(?:authored_text|to_property_name)\s*\(/u.test(normalized)) return "authored-bearing";
  const structLiteralType = normalized.match(
    /^((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Z][A-Za-z0-9_]*)\s*\{/u,
  )?.[1];
  if (structLiteralType) {
    return classifyAuthoredEscapeType(
      structLiteralType.split(/\s*::\s*/u).at(-1),
      structFields,
      new Set(),
    );
  }
  const access = normalized.match(/^([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)/u)?.[1];
  const rustType = access ? resolveRustAccessType(access, bindings, structFields) : undefined;
  return classifyAuthoredEscapeType(rustType, structFields, new Set());
}

function classifyAuthoredEscapeType(
  rustType: string | undefined,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  seen: ReadonlySet<string>,
): AuthoredEscapeOperandClass {
  if (!rustType || /\b(?:dyn|impl)\b/u.test(rustType)) return "authored-bearing";
  if (/\b(?:AuthoredPropertyTextV0|PropertyNameV0)\b/u.test(rustType)) return "authored-bearing";
  if (
    /\b(?:CanonicalPropertyKeyV0|CanonicalCustomPropertyNameV0|CanonicalStandardPropertyNameV0)\b/u.test(
      rustType,
    )
  )
    return "key-bearing";
  if (/\bCanonical(?:Class|Id|TypeSelector)KeyV0\b/u.test(rustType)) return "non-property";
  if (
    /^(?:str|String|PathBuf|Path|serde_json::Value|Value|bool|char|u(?:8|16|32|64|128|size)|i(?:8|16|32|64|128|size)|f(?:32|64)|Ordering|Range|Duration)$/u.test(
      rustType,
    )
  )
    return "non-property";
  const typeName = rustType.match(/[A-Z][A-Za-z0-9_]*$/u)?.[0] ?? rustType;
  if (externalLeafTypes.some((entry) => entry.typeName === typeName)) return "non-property";
  if (seen.has(typeName)) return "non-property";
  const fields = structFields.get(typeName);
  if (!fields) return "authored-bearing";
  const nextSeen = new Set(seen).add(typeName);
  const classes = [...fields.values()].map((fieldType) =>
    classifyAuthoredEscapeType(fieldType, structFields, nextSeen),
  );
  if (classes.includes("authored-bearing")) return "authored-bearing";
  if (classes.includes("key-bearing")) return "key-bearing";
  return "non-property";
}

function resultBindingsForEscape(
  statement: string,
  sinkBinding: string | undefined,
): readonly string[] {
  const bindings = new Set<string>();
  const assigned = statement.match(/\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=/u)?.[1];
  if (assigned && assigned !== "_") bindings.add(assigned);
  if (sinkBinding) bindings.add(sinkBinding);
  return [...bindings];
}

function computeFunctionTaint(
  record: EscapeFunctionRecord,
  returnSummaries: ReadonlyMap<string, ReadonlySet<number>>,
  definitionCounts: ReadonlyMap<string, number>,
): FunctionTaintResult {
  const bindings = new Map<string, Set<number>>();
  const unresolvedEdges: CensusSite[] = [];
  for (const occurrence of record.occurrences) {
    for (const binding of occurrence.resultBindings) {
      const sources = bindings.get(binding) ?? new Set<number>();
      sources.add(occurrence.siteIndex);
      bindings.set(binding, sources);
    }
  }
  for (let pass = 0; pass <= bindings.size + 8; pass += 1) {
    let changed = false;
    for (const binding of record.body.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);/gu,
    )) {
      if (binding[1] === "_") continue;
      const sources = sealedAuthorityExpression(binding[2])
        ? new Set<number>()
        : taintedSourcesForText(
            binding[2],
            bindings,
            record.crateName,
            returnSummaries,
            definitionCounts,
            record.occurrences,
          );
      const prior = bindings.get(binding[1]) ?? new Set<number>();
      const merged = new Set([...prior, ...sources]);
      if (merged.size !== prior.size) {
        bindings.set(binding[1], merged);
        changed = true;
      }
      for (const call of unresolvedTaintedCalls(binding[2], bindings, record, definitionCounts)) {
        unresolvedEdges.push(call);
      }
    }
    if (!changed) break;
    assert.ok(pass < bindings.size + 8, `local taint did not converge in ${record.functionName}`);
  }
  const returnSources = new Set<number>();
  for (const returned of record.body.matchAll(/\breturn\s+([^;]+);/gu)) {
    for (const source of taintedSourcesForText(
      returned[1],
      bindings,
      record.crateName,
      returnSummaries,
      definitionCounts,
      record.occurrences,
    ))
      returnSources.add(source);
  }
  const tail = record.body.trim().replace(/;+$/u, "").split(/\n/u).at(-1) ?? "";
  for (const source of taintedSourcesForText(
    tail,
    bindings,
    record.crateName,
    returnSummaries,
    definitionCounts,
    record.occurrences,
  ))
    returnSources.add(source);
  return { bindingSources: bindings, returnSources, unresolvedEdges };
}

function sealedAuthorityExpression(expression: string): boolean {
  return (
    /\b(?:ClassNameV0|PropertyNameV0)\s*::\s*(?:new|from_authored|custom|standard|canonical_custom_key|canonical_standard_key)\s*\(/u.test(
      expression,
    ) ||
    /\bcanonical_class_key\s*\(/u.test(expression) ||
    /\.(?:to_custom_key|to_standard_key|to_property_name|canonical_key)\s*\(/u.test(expression)
  );
}

function taintedSourcesForText(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
  crateName: string,
  returnSummaries: ReadonlyMap<string, ReadonlySet<number>>,
  definitionCounts: ReadonlyMap<string, number>,
  localOccurrences: readonly EscapeOccurrenceDraft[],
): ReadonlySet<number> {
  const sources = new Set<number>();
  for (const occurrence of localOccurrences) {
    if (text.includes(occurrence.matchedText)) sources.add(occurrence.siteIndex);
  }
  for (const [binding, bindingTaint] of bindingSources) {
    if (new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text)) {
      for (const source of bindingTaint) sources.add(source);
    }
  }
  for (const call of text.matchAll(/(?<![.:])\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)) {
    const key = `${crateName}\0${call[1]}`;
    if (definitionCounts.get(key) !== 1) continue;
    for (const source of returnSummaries.get(key) ?? []) sources.add(source);
  }
  return sources;
}

function unresolvedTaintedCalls(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
  record: EscapeFunctionRecord,
  definitionCounts: ReadonlyMap<string, number>,
): readonly CensusSite[] {
  const sites: CensusSite[] = [];
  for (const call of text.matchAll(/(?<![.:])\b([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/gu)) {
    const key = `${record.crateName}\0${call[1]}`;
    if (definitionCounts.get(key) === 1) continue;
    const receivesTaint = [...bindingSources].some(
      ([binding, sources]) =>
        sources.size > 0 && new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(call[2]),
    );
    if (!receivesTaint) continue;
    const offset = record.body.indexOf(call[0]);
    sites.push({
      path: record.path,
      line: lineNumberAt(record.source, record.bodyStart + Math.max(offset, 0)),
      function: record.functionName,
      operation: "unresolved-tainted-call-edge",
      evidence: call[0].replace(/\s+/gu, " "),
      disposition: "named-exempt",
      reason: "unresolved callee default remains tainted",
    });
  }
  return sites;
}

interface RustStatementSlice {
  readonly text: string;
  readonly start: number;
}

function rustStatementSlices(body: string): readonly RustStatementSlice[] {
  const statements: RustStatementSlice[] = [];
  const scannable = maskCommentsStringsAndTestItems(body, false);
  let start = 0;
  let braceDepth = 0;
  let bracketDepth = 0;
  let parenthesisDepth = 0;
  for (let index = 0; index < body.length; index += 1) {
    switch (scannable[index]) {
      case "{":
        braceDepth += 1;
        break;
      case "}":
        braceDepth -= 1;
        break;
      case "[":
        bracketDepth += 1;
        break;
      case "]":
        bracketDepth -= 1;
        break;
      case "(":
        parenthesisDepth += 1;
        break;
      case ")":
        parenthesisDepth -= 1;
        break;
    }
    if (
      (scannable[index] !== ";" && scannable[index] !== "\n") ||
      braceDepth !== 0 ||
      bracketDepth !== 0 ||
      parenthesisDepth !== 0
    )
      continue;
    const text = body.slice(start, index + 1).trim();
    if (text) statements.push({ text, start });
    start = index + 1;
  }
  const tail = body.slice(start).trim();
  if (tail) statements.push({ text: tail, start });
  return statements;
}

function rustStatementAt(body: string, offset: number): RustStatementSlice {
  const beforeSemicolon = body.lastIndexOf(";", offset - 1);
  const beforeNewline = body.lastIndexOf("\n", offset - 1);
  const start = Math.max(beforeSemicolon, beforeNewline) + 1;
  const afterSemicolon = body.indexOf(";", offset);
  const end = afterSemicolon < 0 ? body.length : afterSemicolon + 1;
  return { text: body.slice(start, end), start };
}

function productionComparisonId(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
): AuthoredComparisonId | undefined {
  if (sealedAuthorityExpression(text)) return undefined;
  if (
    /\.sort_by_key\s*\(\s*\|\s*\(\s*[A-Za-z_][A-Za-z0-9_]*_key\s*,/u.test(text) &&
    !/\.label\b/u.test(text)
  )
    return undefined;
  if (
    /\.filter\s*\(\s*\|\s*\(\s*[A-Za-z_][A-Za-z0-9_]*_key\s*,[^|]*\|[^;]*\.insert\s*\([^)]*_key/u.test(
      text,
    )
  )
    return undefined;
  const taintedBindings = [...bindingSources]
    .filter(([, sources]) => sources.size > 0)
    .map(([binding]) => binding);
  if (
    !taintedBindings.some((binding) => new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text))
  )
    return undefined;
  let normalized = text;
  for (const binding of taintedBindings)
    normalized = normalized.replaceAll(new RegExp(`\\b${escapeRegExp(binding)}\\b`, "gu"), "value");
  const generated = detectGeneratedComparison(normalized);
  if (generated) return generated;
  if (/\.(?:insert|entry)\s*\([^)]*\bvalue\b/u.test(normalized)) return "map-insert";
  if (/\.(?:get|contains_key|contains|remove)\s*\([^)]*\bvalue\b/u.test(normalized))
    return "map-get";
  if (
    /\bmatch\s+value\s*\.\s*as_str\s*\(\s*\)\s*\{[\s\S]*?"(?:[^"\\]|\\.)*"\s*=>/u.test(normalized)
  )
    return "match-literal";
  return undefined;
}

function classifyWriteIntoSink(
  site: EscapeOccurrenceDraft & { readonly sinkBinding: string },
  taint: FunctionTaintResult,
  identitySources: ReadonlySet<number>,
): WriteIntoSinkClass {
  const binding = site.sinkBinding;
  if (/^(?:f|fmt|formatter)$/u.test(binding)) return "formatter";
  if (taint.returnSources.has(site.siteIndex)) return "returned-to-emitter";
  if (identitySources.has(site.siteIndex)) return "unresolved";
  return "emitter-output";
}

function escapeFunctionReturnType(signature: string): string | undefined {
  return signature.match(/->\s*([^\n{]+?)(?:\s+where\b|\s*$)/u)?.[1]?.trim();
}

function discoverAuthoredIdentityCarrierAudit(
  sources: readonly MutableRustSource[],
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): {
  readonly audit: readonly AuthoredIdentityCarrierAudit[];
  readonly deriveViolationCount: number;
  readonly deriveViolations: readonly string[];
} {
  const directAuthoredTypes = new Set(
    [...structFields]
      .filter(([, fields]) =>
        [...fields.values()].some(
          (fieldType) => fieldType === "AuthoredPropertyTextV0" || fieldType === "PropertyNameV0",
        ),
      )
      .map(([typeName]) => typeName),
  );
  const impls = new Map<string, Set<string>>();
  const paths = new Map<string, Set<string>>();
  let deriveViolationCount = 0;
  const deriveViolations: string[] = [];
  for (const { relativePath, source } of sources) {
    for (const typeName of directAuthoredTypes) {
      if (new RegExp(`\\b(?:struct|enum)\\s+${escapeRegExp(typeName)}\\b`, "u").test(source)) {
        const typePaths = paths.get(typeName) ?? new Set<string>();
        typePaths.add(relativePath);
        paths.set(typeName, typePaths);
      }
      const deriveBlock = source.match(
        new RegExp(
          `#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*(?:pub(?:\\([^)]*\\))?\\s+)?struct\\s+${escapeRegExp(typeName)}\\b`,
          "u",
        ),
      )?.[1];
      if (deriveBlock && /\b(?:PartialEq|Eq|PartialOrd|Ord|Hash)\b/u.test(deriveBlock)) {
        deriveViolationCount += 1;
        deriveViolations.push(
          `${relativePath}:${typeName}:${deriveBlock.replace(/\s+/gu, " ").trim()}`,
        );
      }
    }
    for (const implementation of source.matchAll(
      /\bimpl(?:\s*<[^>{}]*>)?\s+(PartialEq|Eq|PartialOrd|Ord|Hash)\s+for\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu,
    )) {
      if (!directAuthoredTypes.has(implementation[2])) continue;
      const typeImpls = impls.get(implementation[2]) ?? new Set<string>();
      typeImpls.add(implementation[1]);
      impls.set(implementation[2], typeImpls);
    }
  }
  const testResult = spawnSync("git", ["ls-files", "rust/crates/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(testResult.status, 0, "failed to enumerate Rust tests for carrier audit");
  const identityTests: {
    readonly crateName: string;
    readonly name: string;
    readonly body: string;
  }[] = [];
  for (const relativePath of testResult.stdout.split(/\r?\n/u).filter(Boolean)) {
    const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    const crateName = relativePath.split("/")[2] ?? "<unknown-crate>";
    for (const functionSlice of rustFunctionSlices(source)) {
      if (!/identity/u.test(functionSlice.name)) continue;
      if (
        !new RegExp(
          `#\\s*\\[\\s*test\\s*\\][\\s\\S]{0,160}\\bfn\\s+${escapeRegExp(functionSlice.name)}\\b`,
          "u",
        ).test(source)
      )
        continue;
      identityTests.push({
        crateName,
        name: functionSlice.name,
        body: functionSlice.scannable,
      });
    }
  }
  const audit = [...directAuthoredTypes]
    .filter((typeName) => (impls.get(typeName)?.size ?? 0) > 0)
    .map((typeName) => {
      const typePaths = [...(paths.get(typeName) ?? [])].toSorted();
      const crates = new Set(typePaths.map((relativePath) => relativePath.split("/")[2]));
      const discriminatingTests = identityTests
        .filter(
          (test) =>
            crates.has(test.crateName) &&
            (new RegExp(`\\b${escapeRegExp(typeName)}\\b`, "u").test(test.body) ||
              carrierTestNameStems(typeName).some((stem) => test.name.includes(stem))),
        )
        .map((test) => test.name)
        .toSorted();
      return {
        typeName,
        paths: typePaths,
        identityImpls: [...(impls.get(typeName) ?? [])].toSorted(),
        discriminatingTests,
      };
    })
    .toSorted((left, right) => left.typeName.localeCompare(right.typeName));
  const missingDiscriminatingTests = audit.filter(
    (row) => row.identityImpls.length > 0 && row.discriminatingTests.length === 0,
  );
  assert.deepEqual(
    missingDiscriminatingTests,
    [],
    "manual authored-carrier identity implementations require one named discriminating test per carrier",
  );
  return { audit, deriveViolationCount, deriveViolations: deriveViolations.toSorted() };
}

function carrierTestNameStems(typeName: string): readonly string[] {
  const base = typeName
    .replace(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .replace(/([A-Z])([A-Z][a-z])/gu, "$1_$2")
    .toLowerCase()
    .replace(/_v0$/u, "");
  const tokens = base.split("_");
  const stems = new Set<string>([base]);
  for (let start = 1; start < Math.min(tokens.length - 1, 4); start += 1) {
    stems.add(tokens.slice(start).join("_"));
  }
  for (const stem of Array.from(stems)) {
    stems.add(stem.replace(/_custom_property_/u, "_"));
    if (stem.endsWith("_annotation")) stems.add(`${stem}s`);
  }
  return [...stems].filter((stem) => stem.split("_").length >= 2);
}

function authoredEscapeSiteOrder(left: AuthoredEscapeSite, right: AuthoredEscapeSite): number {
  return (
    left.path.localeCompare(right.path) ||
    left.line - right.line ||
    left.escapeId.localeCompare(right.escapeId)
  );
}

function authoredEscapeFlowOrder(
  left: AuthoredEscapeIdentityFlow,
  right: AuthoredEscapeIdentityFlow,
): number {
  return (
    left.path.localeCompare(right.path) ||
    left.line - right.line ||
    left.comparisonId.localeCompare(right.comparisonId)
  );
}

function writeIntoSiteOrder(left: WriteIntoSinkSite, right: WriteIntoSinkSite): number {
  return (
    left.path.localeCompare(right.path) ||
    left.line - right.line ||
    left.escapeId.localeCompare(right.escapeId)
  );
}

function evidenceLine(source: string, offset: number): string {
  return (
    source.split(/\r?\n/u)[lineNumberAt(source, offset) - 1]?.trim().replace(/\s+/gu, " ") ?? ""
  );
}

interface RustSemanticToken {
  readonly text: string;
  readonly start: number;
  readonly end: number;
}

type RustFunctionResultKind = "raw-string" | "non-raw";

function rustFunctionResultKinds(
  sources: readonly { source: string }[],
): ReadonlyMap<string, RustFunctionResultKind> {
  const results = new Map<string, RustFunctionResultKind>();
  for (const { source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const returnType = functionSlice.signature.match(/->\s*([^\n{]+?)(?:\s+where\b|\s*$)/u)?.[1];
      if (!returnType) continue;
      const kind: RustFunctionResultKind = rawStringType(returnType.trim())
        ? "raw-string"
        : "non-raw";
      const prior = results.get(functionSlice.name);
      if (prior === "raw-string" || kind === "raw-string")
        results.set(functionSlice.name, "raw-string");
      else results.set(functionSlice.name, "non-raw");
    }
  }
  return results;
}

function maskNonRawResultCalls(
  tokens: readonly RustSemanticToken[],
  resultKinds: ReadonlyMap<string, RustFunctionResultKind>,
): readonly RustSemanticToken[] {
  const masked: RustSemanticToken[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (
      resultKinds.get(token.text) !== "non-raw" ||
      tokens[index + 1]?.text !== "(" ||
      tokens[index - 1]?.text === "." ||
      tokens[index - 1]?.text === "::"
    ) {
      masked.push(token);
      continue;
    }
    const closeIndex = matchingTokenDelimiter(tokens, index + 1, "(", ")");
    if (closeIndex === undefined) {
      masked.push(token);
      continue;
    }
    masked.push({ text: "non_raw_result", start: token.start, end: tokens[closeIndex].end });
    index = closeIndex;
  }
  return masked;
}

function maskPropertyAuthorityCalls(
  tokens: readonly RustSemanticToken[],
): readonly RustSemanticToken[] {
  if (
    tokens.some(
      (token, index) =>
        /^(?:to_property_name|to_custom_key|to_standard_key|canonical_key|as_custom_key|as_standard_key)$/u.test(
          token.text,
        ) &&
        tokens[index - 1]?.text === "." &&
        tokens[index + 1]?.text === "(",
    )
  ) {
    return tokens.length === 0
      ? []
      : [
          {
            text: "sealed_property_result",
            start: tokens[0].start,
            end: tokens[tokens.length - 1].end,
          },
        ];
  }
  const masked: RustSemanticToken[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const propertyNameCall =
      token.text === "PropertyNameV0" &&
      tokens[index + 1]?.text === "::" &&
      /^(?:from_authored|custom|standard|canonical_custom_key|canonical_standard_key)$/u.test(
        tokens[index + 2]?.text ?? "",
      ) &&
      tokens[index + 3]?.text === "(";
    const helperCall = token.text === "property_names_same" && tokens[index + 1]?.text === "(";
    const openIndex = propertyNameCall ? index + 3 : helperCall ? index + 1 : undefined;
    if (openIndex === undefined) {
      masked.push(token);
      continue;
    }
    const closeIndex = matchingTokenDelimiter(tokens, openIndex, "(", ")");
    if (closeIndex === undefined) {
      masked.push(token);
      continue;
    }
    masked.push({
      text: "sealed_property_result",
      start: token.start,
      end: tokens[closeIndex].end,
    });
    index = closeIndex;
  }
  return masked;
}

interface RustAccessPath {
  readonly root: string;
  readonly segments: readonly RustAccessSegment[];
}

interface RustAccessSegment {
  readonly name: string;
  readonly method: boolean;
}

function rustSemanticTokens(source: string): RustSemanticToken[] {
  const tokens: RustSemanticToken[] = [];
  const compoundSymbols = new Set(["==", "!=", "::", "&&", "||", "=>", "<=", ">="]);
  for (let index = 0; index < source.length;) {
    const character = source[index];
    if (/\s/u.test(character)) {
      index += 1;
      continue;
    }
    if (/[A-Za-z_]/u.test(character)) {
      let end = index + 1;
      while (end < source.length && /[A-Za-z0-9_]/u.test(source[end])) end += 1;
      tokens.push({ text: source.slice(index, end), start: index, end });
      index = end;
      continue;
    }
    const pair = source.slice(index, index + 2);
    if (compoundSymbols.has(pair)) {
      tokens.push({ text: pair, start: index, end: index + 2 });
      index += 2;
      continue;
    }
    tokens.push({ text: character, start: index, end: index + 1 });
    index += 1;
  }
  return tokens;
}

function rustAccessPaths(tokens: readonly RustSemanticToken[]): RustAccessPath[] {
  const paths: RustAccessPath[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const root = tokens[index];
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/u.test(root.text)) continue;
    if (tokens[index - 1]?.text === "." || tokens[index - 1]?.text === "::") continue;
    const segments: RustAccessSegment[] = [];
    let cursor = index + 1;
    while (tokens[cursor]?.text === ".") {
      const member = tokens[cursor + 1];
      if (!member || !/^[A-Za-z_][A-Za-z0-9_]*$/u.test(member.text)) break;
      if (tokens[cursor + 2]?.text === "(") {
        const closeParen = matchingTokenDelimiter(tokens, cursor + 2, "(", ")");
        if (closeParen === undefined) break;
        segments.push({ name: member.text, method: true });
        cursor = closeParen + 1;
      } else {
        segments.push({ name: member.text, method: false });
        cursor += 2;
      }
    }
    paths.push({ root: root.text, segments });
  }
  return paths;
}

function rawPropertyExpressionKind(
  access: RustAccessPath,
  rawBindings: ReadonlySet<string>,
  propertyBindings: ReadonlySet<string>,
  bindingTypes: ReadonlyMap<string, string>,
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
  _arbitraryMethodsPreserveOrigin = false,
): boolean {
  let rawProperty = rawBindings.has(access.root) && propertyBindings.has(access.root);
  const rootType = bindingTypes.get(access.root);
  for (const segment of access.segments) {
    if (!segment.method) {
      rawProperty = Boolean(rootType && carrierFields.get(rootType)?.has(segment.name));
      continue;
    }
    if (!rawProperty) return false;
  }
  return rawProperty;
}

function operationSideTokens(
  tokens: readonly RustSemanticToken[],
  operatorIndex: number,
  direction: "left" | "right",
): readonly RustSemanticToken[] {
  const boundaries = new Set([";", "{", "}", ",", "&&", "||", "=>", "=", "|"]);
  const keywordBoundaries = new Set(["if", "let", "match", "return", "while"]);
  if (direction === "left") {
    let depth = 0;
    let start = operatorIndex;
    for (let index = operatorIndex - 1; index >= 0; index -= 1) {
      const token = tokens[index].text;
      if ([")", "]"].includes(token)) depth += 1;
      else if (["(", "["].includes(token)) {
        if (depth === 0) break;
        depth -= 1;
      }
      if (depth === 0 && (boundaries.has(token) || keywordBoundaries.has(token))) break;
      start = index;
    }
    return tokens.slice(start, operatorIndex);
  }

  let depth = 0;
  let end = operatorIndex + 1;
  for (let index = operatorIndex + 1; index < tokens.length; index += 1) {
    const token = tokens[index].text;
    if (["(", "["].includes(token)) depth += 1;
    else if ([")", "]"].includes(token)) {
      if (depth === 0) break;
      depth -= 1;
    }
    if (depth === 0 && (boundaries.has(token) || keywordBoundaries.has(token))) break;
    end = index + 1;
  }
  return tokens.slice(operatorIndex + 1, end);
}

function matchingTokenDelimiter(
  tokens: readonly RustSemanticToken[],
  openIndex: number,
  open: string,
  close: string,
): number | undefined {
  let depth = 0;
  for (let index = openIndex; index < tokens.length; index += 1) {
    if (tokens[index].text === open) depth += 1;
    else if (tokens[index].text === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function firstArgumentTokens(tokens: readonly RustSemanticToken[]): readonly RustSemanticToken[] {
  let depth = 0;
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index].text;
    if (["(", "[", "{"].includes(token)) depth += 1;
    else if ([")", "]", "}"].includes(token)) depth -= 1;
    else if (token === "," && depth === 0) return tokens.slice(0, index);
  }
  return tokens;
}

function rawStringCollectionDeclarations(
  functionSlice: RustFunctionSlice,
): RawStringCollectionDeclaration[] {
  const declarations: RawStringCollectionDeclaration[] = [];
  const start =
    /\b(?:let\s+(?:mut\s+)?)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:(?:[A-Za-z_][A-Za-z0-9_]*::)+)?(?:BTreeMap|BTreeSet|HashMap|HashSet|Vec)\s*</gu;
  for (const match of functionSlice.scannable.matchAll(start)) {
    const openAngle = match.index + match[0].lastIndexOf("<");
    const closeAngle = matchingDelimiter(functionSlice.scannable, openAngle, "<", ">");
    if (closeAngle === undefined) continue;
    const genericArguments = functionSlice.scannable.slice(openAngle + 1, closeAngle);
    if (!rawStringType(firstTopLevelGenericArgument(genericArguments))) continue;
    declarations.push({ binding: match[1], offset: match.index });
  }
  return declarations;
}

function firstTopLevelGenericArgument(argumentsSource: string): string {
  let depth = 0;
  for (let index = 0; index < argumentsSource.length; index += 1) {
    const character = argumentsSource[index];
    if (character === "<" || character === "(" || character === "[") depth += 1;
    else if (character === ">" || character === ")" || character === "]") depth -= 1;
    else if (character === "," && depth === 0) return argumentsSource.slice(0, index).trim();
  }
  return argumentsSource.trim();
}

function rawStringType(typeSource: string): boolean {
  return /^(?:String|&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str)\b/u.test(typeSource.trim());
}

interface RustStructField {
  readonly name: string;
  readonly rustType: string;
  readonly offset: number;
}

function rustStructFields(body: string): readonly RustStructField[] {
  const fields: RustStructField[] = [];
  for (const segment of topLevelSegments(body, ",")) {
    const field = segment.text.match(/\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*/u);
    if (!field || field.index === undefined) continue;
    const rustType = segment.text.slice(field.index + field[0].length).trim();
    if (rustType.length === 0) continue;
    const nameOffset = field[0].lastIndexOf(field[1]);
    fields.push({
      name: field[1],
      rustType,
      offset: segment.offset + field.index + nameOffset,
    });
  }
  return fields;
}

function rawPropertyCarrierType(typeSource: string): boolean {
  const rustType = typeSource.trim();
  if (rawStringType(rustType)) return true;
  if (injectContainerPredicateRevert) return false;
  if (/^(?:(?:[A-Za-z_][A-Za-z0-9_]*::)*)?Cow\s*</u.test(rustType)) {
    const argumentsSource = outerGenericArguments(rustType);
    if (argumentsSource === undefined) return false;
    const argumentsList = topLevelSegments(argumentsSource, ",").map(({ text }) => text.trim());
    return argumentsList.at(-1) === "str";
  }
  if (rustType.startsWith("(") && rustType.endsWith(")")) {
    return topLevelSegments(rustType.slice(1, -1), ",").some(({ text }) =>
      rawPropertyCarrierType(text),
    );
  }

  const genericArguments = outerGenericArguments(rustType);
  if (genericArguments === undefined) return false;
  const argumentsList = topLevelSegments(genericArguments, ",").map(({ text }) => text.trim());
  const typeName = rustType.slice(0, rustType.indexOf("<")).trim().split("::").at(-1);
  if (typeName === "Option") return argumentsList.some(rawPropertyCarrierType);
  if (["Vec", "VecDeque", "BTreeSet", "HashSet"].includes(typeName ?? "")) {
    return argumentsList.length > 0 && rawPropertyCarrierType(argumentsList[0]);
  }
  if (typeName === "Box" && argumentsList.length === 1) {
    const boxed = argumentsList[0];
    return boxed.startsWith("[") && boxed.endsWith("]")
      ? rawPropertyCarrierType(boxed.slice(1, -1))
      : false;
  }
  if (["BTreeMap", "HashMap"].includes(typeName ?? "")) {
    return argumentsList.length > 0 && rawPropertyCarrierType(argumentsList[0]);
  }
  return false;
}

function outerGenericArguments(typeSource: string): string | undefined {
  const openAngle = typeSource.indexOf("<");
  if (openAngle < 0) return undefined;
  const closeAngle = matchingDelimiter(typeSource, openAngle, "<", ">");
  if (closeAngle === undefined || typeSource.slice(closeAngle + 1).trim().length > 0)
    return undefined;
  return typeSource.slice(openAngle + 1, closeAngle);
}

function topLevelSegments(
  source: string,
  separator: string,
): readonly { readonly text: string; readonly offset: number }[] {
  const segments: { text: string; offset: number }[] = [];
  let segmentStart = 0;
  const depths = new Map<string, number>([
    ["<", 0],
    ["(", 0],
    ["[", 0],
    ["{", 0],
  ]);
  const openingByClose = new Map([
    [">", "<"],
    [")", "("],
    ["]", "["],
    ["}", "{"],
  ]);
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    if (depths.has(character)) {
      depths.set(character, (depths.get(character) ?? 0) + 1);
    } else {
      const opening = openingByClose.get(character);
      if (opening !== undefined) {
        depths.set(opening, Math.max(0, (depths.get(opening) ?? 0) - 1));
      }
    }
    if (character === separator && [...depths.values()].every((depth) => depth === 0)) {
      segments.push({ text: source.slice(segmentStart, index), offset: segmentStart });
      segmentStart = index + 1;
    }
  }
  segments.push({ text: source.slice(segmentStart), offset: segmentStart });
  return segments;
}

function isPropertySemanticIdentifier(identifier: string): boolean {
  const words = semanticIdentifierWords(identifier);
  return words.includes("property") || words.includes("properties");
}

function isPropertyIdentityFunctionIdentifier(identifier: string): boolean {
  const words = semanticIdentifierWords(identifier);
  if (!words.includes("property") && !words.includes("properties")) return false;
  return words.some((word) =>
    [
      "canonical",
      "canonicalize",
      "compare",
      "equal",
      "equality",
      "identity",
      "join",
      "key",
      "keys",
      "name",
      "names",
      "normalize",
      "same",
    ].includes(word),
  );
}

function semanticIdentifierWords(identifier: string): string[] {
  return identifier
    .replace(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .toLowerCase()
    .split(/[^a-z0-9]+/u)
    .filter(Boolean);
}

function isPropertyMapBinding(identifier: string): boolean {
  if (!isPropertySemanticIdentifier(identifier)) return false;
  const words = semanticIdentifierWords(identifier);
  return !words.some((word) =>
    ["selector", "selectors", "path", "paths", "uri", "uris", "file", "files"].includes(word),
  );
}

function matchingDelimiter(
  source: string,
  openOffset: number,
  open: string,
  close: string,
): number | undefined {
  let depth = 0;
  for (let index = openOffset; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    else if (source[index] === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function derivesForStruct(source: string, structName: string): string[] {
  const match = source.match(
    new RegExp(
      `#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*(?:#\\s*\\[[^\\]]*\\]\\s*)*pub\\s+struct\\s+${structName}\\b`,
      "u",
    ),
  );
  assert.ok(match, `derive list missing for ${structName}`);
  return match[1]
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function derivesForEnum(source: string, enumName: string): string[] {
  const match = source.match(
    new RegExp(
      `#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*(?:#\\s*\\[[^\\]]*\\]\\s*)*pub\\s+enum\\s+${enumName}\\b`,
      "u",
    ),
  );
  assert.ok(match, `derive list missing for ${enumName}`);
  return match[1]
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function discoverEgressSites(): DiscoveredSite[] {
  const discovered: DiscoveredSite[] = [];
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectEgress) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_egress.rs",
      source:
        "use omena_syntax::ident::ClassNameV0;\nfn injected_identifier_egress(class: &ClassNameV0) { let n: String = class.raw().to_string(); drop(n); }\n",
    });
  }
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const candidatePatterns: readonly (readonly [string, RegExp])[] = [
      ["ClassNameV0::into_raw", /\b[A-Za-z_][A-Za-z0-9_]*\.name\.into_raw\s*\(\s*\)/gu],
      [
        "ClassNameV0::raw",
        source.includes("ClassNameV0") ? /\b[A-Za-z_][A-Za-z0-9_]*\.raw\s*\(\s*\)/gu : /$a/gu,
      ],
      [
        "ClassNameV0::into_inner",
        source.includes("ClassNameV0")
          ? /\b[A-Za-z_][A-Za-z0-9_]*\.into_inner\s*\(\s*\)/gu
          : /$a/gu,
      ],
      [
        "CanonicalClassKeyV0::as_str",
        /\b([A-Za-z_][A-Za-z0-9_]*)\.canonical_key\s*\(\s*\)\s*\.as_str\s*\(\s*\)/gu,
      ],
    ];
    for (const [operation, expression] of candidatePatterns) {
      expression.lastIndex = 0;
      for (const match of scannable.matchAll(expression)) {
        if (
          operation === "CanonicalClassKeyV0::as_str" &&
          match[1]?.toLowerCase().includes("property")
        ) {
          continue;
        }
        discovered.push(siteAt(relativePath, source, scannable, match.index, operation));
      }
    }
    const canonicalVariables = new Set<string>();
    for (const match of scannable.matchAll(
      /\blet\s+([A-Za-z_][A-Za-z0-9_]*)[^=;\n]*=\s*[^;\n]*\.canonical_key\s*\(\s*\)/gu,
    )) {
      if (/property/u.test(match[0])) continue;
      canonicalVariables.add(match[1]);
    }
    for (const match of scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:&\s*)?CanonicalClassKeyV0\b/gu,
    )) {
      canonicalVariables.add(match[1]);
    }
    for (const variable of canonicalVariables) {
      const expression = new RegExp(`\\b${variable}\\.as_str\\s*\\(\\s*\\)`, "gu");
      for (const match of scannable.matchAll(expression)) {
        discovered.push(
          siteAt(relativePath, source, scannable, match.index, "CanonicalClassKeyV0::as_str"),
        );
      }
    }
  }
  return uniqueSites(discovered);
}

function discoverIdiomSites(): DiscoveredSite[] {
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectLabelledComparison) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_comparison.rs",
      source:
        "fn injected(left_class_name: String, right_class_name: String) -> bool { left_class_name == right_class_name }\n",
    });
  }
  if (injectUnlabelledComparison) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_unlabelled_comparison.rs",
      source: "fn injected(a: String, b: String) -> bool { a == b }\n",
    });
  }
  const discovered: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, true);
    const sourceLines = source.split(/\r?\n/u);
    for (const [index, line] of scannable.split(/\r?\n/u).entries()) {
      if (!classBindingLexeme.test(line)) continue;
      for (const [operation, expression] of primitivePatterns) {
        if (!expression.test(line)) continue;
        discovered.push({
          path: relativePath,
          line: index + 1,
          function: enclosingFunctionName(scannable, offsetForLine(scannable, index + 1)),
          operation,
          evidence: sourceLines[index]?.trim().replace(/\s+/gu, " ") ?? "",
        });
      }
    }
  }
  return uniqueSites(discovered);
}

function discoverPredicateCopies(): DiscoveredSite[] {
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectPredicateCopy) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate.rs",
      source:
        "fn unrelated_name(ch: char) -> bool { matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_') }\n",
    });
  }
  if (injectPredicateCopyExplicit) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate_explicit.rs",
      source:
        "fn arbitrary_spelling(codepoint: char) -> bool { codepoint.is_alphanumeric() || codepoint == '-' || codepoint == '_' }\n",
    });
  }
  if (injectPredicateCopyReversed) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate_reversed.rs",
      source:
        "fn another_spelling(byte: u8) -> bool { matches!(byte, b'-' | b'_') || byte.is_ascii_alphanumeric() }\n",
    });
  }
  const discovered: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    for (const functionBody of directCharacterPredicateBodies(source)) {
      discovered.push({
        path: relativePath,
        line: functionBody.line,
        function: functionBody.name,
        operation: "character-predicate-copy",
        evidence: functionBody.evidence,
      });
    }
  }
  return uniqueSites(discovered);
}

function directCharacterPredicateBodies(
  source: string,
): readonly { name: string; line: number; evidence: string }[] {
  const scannable = maskCommentsStringsAndTestItems(source, false);
  const found: { name: string; line: number; evidence: string }[] = [];
  const declaration =
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*:\s*(?:char|u8)\s*\)\s*->\s*bool\s*\{/gu;
  for (const match of scannable.matchAll(declaration)) {
    const openBrace = match.index + match[0].lastIndexOf("{");
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    const body = maskCommentsStringsAndTestItems(
      source.slice(openBrace + 1, closeBrace),
      false,
      true,
    )
      .replace(/\s+/gu, "")
      .replace(/^return/u, "")
      .replace(/;$/u, "");
    const parameter = match[0].match(/\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:char|u8)\s*\)/u)?.[1];
    if (!parameter || !isIdentifierContinuationPredicate(body, parameter)) continue;
    found.push({
      name: match[1],
      line: lineNumberAt(source, match.index),
      evidence: source
        .slice(match.index, closeBrace + 1)
        .trim()
        .replace(/\s+/gu, " "),
    });
  }
  return found;
}

function isIdentifierContinuationPredicate(body: string, parameter: string): boolean {
  const escapedParameter = escapeRegExp(parameter);
  const acceptsHyphen = new RegExp(
    `(?:${escapedParameter}==b?'-'|b?'-'==${escapedParameter}|matches!\\(${escapedParameter},[^)]*b?'-')`,
    "u",
  ).test(body);
  const acceptsUnderscore = new RegExp(
    `(?:${escapedParameter}==b?'_'|b?'_'==${escapedParameter}|matches!\\(${escapedParameter},[^)]*b?'_')`,
    "u",
  ).test(body);
  const usesAlphanumericPredicate = new RegExp(
    `${escapedParameter}\\.is_(?:ascii_)?alphanumeric\\(\\)`,
    "u",
  ).test(body);
  const usesSharedCssPredicate = new RegExp(
    `is_css_name_start\\(${escapedParameter}\\).*${escapedParameter}\\.is_ascii_digit\\(\\)`,
    "u",
  ).test(body);
  const rangeBody = body.match(new RegExp(`matches!\\(${escapedParameter},([^)]*)\\)`, "u"))?.[1];
  const usesAlphaNumericRanges =
    rangeBody !== undefined &&
    /b?'[a-zA-Z0-9]'\.\.=b?'[a-zA-Z0-9]'/u.test(rangeBody) &&
    (rangeBody.match(/b?'[a-zA-Z0-9]'\.\.=b?'[a-zA-Z0-9]'/gu)?.length ?? 0) >= 3;
  return (
    (usesSharedCssPredicate || usesAlphanumericPredicate || usesAlphaNumericRanges) &&
    (usesSharedCssPredicate || (acceptsHyphen && acceptsUnderscore))
  );
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function classifySites(
  discovered: readonly DiscoveredSite[],
  previous: readonly CensusSite[] | undefined,
  exemptions: readonly ExemptionRule[],
  arm: string,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  const previousByKey = new Map((previous ?? []).map((site) => [stableSiteKey(site), site]));
  return discovered.map((site) => {
    const exemption = exemptions.find((rule) => stableSiteKey(rule) === stableSiteKey(site));
    if (exemption) {
      return {
        ...site,
        disposition: exemption.disposition ?? "named-exempt",
        reason: exemption.reason,
      } as const;
    }
    const prior = previousByKey.get(stableSiteKey(site));
    if (prior)
      return {
        ...site,
        disposition: prior.disposition,
        ...(prior.reason ? { reason: prior.reason } : {}),
      };
    if (!previous && writeMode) {
      return {
        ...site,
        disposition: "sanctioned",
        reason: `${arm} site reviewed in the initial authored census.`,
      } as const;
    }
    return { ...site, disposition: "unclassified" } as const;
  });
}

function classifyPredicateSites(
  discovered: readonly DiscoveredSite[],
  previous: readonly CensusSite[] | undefined,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  const previousByKey = new Map((previous ?? []).map((site) => [stableSiteKey(site), site]));
  return discovered.map((site) => {
    const prior = previousByKey.get(stableSiteKey(site));
    if (prior)
      return {
        ...site,
        disposition: prior.disposition,
        ...(prior.reason ? { reason: prior.reason } : {}),
      };
    if (!previous && writeMode) {
      const authority = site.path === "rust/crates/omena-syntax/src/ident.rs";
      return {
        ...site,
        disposition: authority ? "sanctioned" : "named-exempt",
        reason: authority
          ? "The shared syntax crate owns the identifier character predicate."
          : `${site.function} owns a non-class identifier grammar and remains visible as a named exemption.`,
      } as const;
    }
    return { ...site, disposition: "unclassified" } as const;
  });
}

function assertNoAddedSites(
  previous: readonly CensusSite[],
  current: readonly CensusSite[],
  label: string,
): void {
  const previousKeys = new Set(previous.map(stableSiteKey));
  const addedSites = current.filter((site) => !previousKeys.has(stableSiteKey(site)));
  assert.deepEqual(
    addedSites,
    [],
    `the committed authored table cannot adopt new ${label} sites during regeneration`,
  );
}

function siteAt(
  relativePath: string,
  source: string,
  scannable: string,
  offset: number,
  operation: string,
): DiscoveredSite {
  const line = lineNumberAt(source, offset);
  return {
    path: relativePath,
    line,
    function: enclosingFunctionName(scannable, offset),
    operation,
    evidence: source.split(/\r?\n/u)[line - 1]?.trim().replace(/\s+/gu, " ") ?? "",
  };
}

function uniqueSites<T extends DiscoveredSite>(sites: readonly T[]): T[] {
  const byKey = new Map<string, T>();
  for (const site of sites) byKey.set(stableSiteKey(site), site);
  return [...byKey.values()].toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.operation.localeCompare(right.operation),
  );
}

function stableSiteKey(
  site: Pick<DiscoveredSite, "path" | "function" | "operation" | "evidence">,
): string {
  return `${site.path}\u0000${site.function}\u0000${site.operation}\u0000${site.evidence}`;
}

function trackedProductionSources(): string[] {
  const result = spawnSync("git", ["ls-files", "rust/crates/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-files failed: ${(result.stderr ?? "").trim()}`);
  const tracked = result.stdout.split(/\r?\n/u).filter(Boolean);
  const cfgTestModuleFiles = new Set<string>();
  for (const sourcePath of tracked) {
    if (!sourcePath.includes("/src/") || !existsSync(path.join(repoRoot, sourcePath))) continue;
    const source = readFileSync(path.join(repoRoot, sourcePath), "utf8");
    for (const module of source.matchAll(
      /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      const directory = path.posix.dirname(sourcePath);
      cfgTestModuleFiles.add(path.posix.join(directory, `${module[1]}.rs`));
      cfgTestModuleFiles.add(path.posix.join(directory, module[1], "mod.rs"));
    }
  }
  return tracked
    .filter((sourcePath) => sourcePath.includes("/src/"))
    .filter((sourcePath) => !sourcePath.includes("/tests/"))
    .filter((sourcePath) => !sourcePath.endsWith("/tests.rs"))
    .filter((sourcePath) => !sourcePath.endsWith("_test.rs"))
    .filter((sourcePath) => !cfgTestModuleFiles.has(sourcePath))
    .filter((sourcePath) => !sourcePath.includes("/src/bin/"))
    .filter((sourcePath) => !sourcePath.endsWith("_generated.rs"))
    .toSorted();
}

function maskCommentsStringsAndTestItems(
  source: string,
  preserveStringContents: boolean,
  preserveCharacterContents = false,
): string {
  const chars = [...source];
  let blockDepth = 0;
  let lineComment = false;
  let stringQuote = "";
  let escaped = false;
  for (let index = 0; index < chars.length; index += 1) {
    const current = chars[index];
    const next = chars[index + 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      else chars[index] = " ";
      continue;
    }
    if (blockDepth > 0) {
      if (current === "/" && next === "*") {
        chars[index] = chars[index + 1] = " ";
        blockDepth += 1;
        index += 1;
      } else if (current === "*" && next === "/") {
        chars[index] = chars[index + 1] = " ";
        blockDepth -= 1;
        index += 1;
      } else if (current !== "\n") {
        chars[index] = " ";
      }
      continue;
    }
    if (stringQuote) {
      if (!preserveStringContents && current !== "\n") chars[index] = " ";
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === stringQuote) stringQuote = "";
      continue;
    }
    if (current === "/" && next === "/") {
      chars[index] = chars[index + 1] = " ";
      lineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      chars[index] = chars[index + 1] = " ";
      blockDepth = 1;
      index += 1;
      continue;
    }
    if (current === '"') {
      stringQuote = current;
      if (!preserveStringContents) chars[index] = " ";
      continue;
    }
    if (current === "'") {
      const characterEnd = rustCharacterLiteralEnd(chars, index);
      if (characterEnd !== undefined) {
        if (!preserveCharacterContents) {
          for (let cursor = index; cursor <= characterEnd; cursor += 1) {
            if (chars[cursor] !== "\n") chars[cursor] = " ";
          }
        }
        index = characterEnd;
      }
    }
  }
  return maskRustCfgTestItems(chars.join(""));
}

function rustCharacterLiteralEnd(chars: readonly string[], quoteIndex: number): number | undefined {
  let index = quoteIndex + 1;
  if (index >= chars.length || chars[index] === "\n") return undefined;
  if (chars[index] === "\\") {
    index += 1;
    if (chars[index] === "x") {
      index += 3;
    } else if (chars[index] === "u" && chars[index + 1] === "{") {
      index += 2;
      while (index < chars.length && chars[index] !== "}" && chars[index] !== "\n") index += 1;
      if (chars[index] !== "}") return undefined;
      index += 1;
    } else {
      index += 1;
    }
  } else {
    index += 1;
  }
  return chars[index] === "'" ? index : undefined;
}

function enclosingFunctionName(source: string, offset: number): string {
  let functionName = "<module>";
  for (const match of source.slice(0, offset).matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
    functionName = match[1];
  }
  return functionName;
}

function matchingBrace(source: string, openBrace: number): number | undefined {
  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function offsetForLine(source: string, line: number): number {
  let offset = 0;
  for (let index = 1; index < line; index += 1) {
    offset = source.indexOf("\n", offset) + 1;
  }
  return offset;
}

function sourceNeedleLocation(relativePath: string, needle: string): string {
  const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  const offset = source.indexOf(needle);
  assert.ok(offset >= 0, `cannotSee anchor missing: ${relativePath}:${needle}`);
  return `${relativePath}:${lineNumberAt(source, offset)}`;
}

function authoredEscapeCannotSee(unresolvedCallEdgeCount: number): readonly string[] {
  const checkerPath = "scripts/check-rust-omena-identifier-authority-census.ts";
  const scopeAnchor = sourceNeedleLocation(
    checkerPath,
    '.filter((sourcePath) => sourcePath.includes("/src/"))',
  );
  const binaryResult = spawnSync("git", ["ls-files", "rust/crates/**/src/bin/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(binaryResult.status, 0, "failed to enumerate excluded Rust binaries");
  const binaries = binaryResult.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((relativePath) => `${relativePath}:1`)
    .join(", ");
  return [
    `${sourceNeedleLocation(checkerPath, '.filter((sourcePath) => !sourcePath.includes("/tests/"))')}: /tests/ trees are outside the product escape-flow scan.`,
    `${sourceNeedleLocation(checkerPath, '.filter((sourcePath) => !sourcePath.endsWith("/tests.rs"))')}: */tests.rs files are outside the product escape-flow scan.`,
    `${sourceNeedleLocation(checkerPath, '.filter((sourcePath) => !sourcePath.endsWith("_test.rs"))')}: *_test.rs files are outside the product escape-flow scan.`,
    `${sourceNeedleLocation(checkerPath, 'return maskRustCfgTestItems(chars.join(""));')}: #[cfg(test)] items are masked before escape-flow analysis.`,
    `${sourceNeedleLocation(checkerPath, '.filter((sourcePath) => !sourcePath.includes("/src/bin/"))')}: /src/bin/ files are excluded; named binaries: ${binaries || "none"}.`,
    `${sourceNeedleLocation(checkerPath, '.filter((sourcePath) => !sourcePath.endsWith("_generated.rs"))')}: *_generated.rs files are excluded.`,
    `${scopeAnchor}: benches/, examples/, and build.rs are outside the /src/ production-source root.`,
    `${sourceNeedleLocation("rust/crates/omena-diff-test/Cargo.toml", 'name = "omena-diff-test"')}: fixture crate omena-diff-test is outside product escape-flow closure.`,
    `${sourceNeedleLocation("rust/crates/engine-style-parser/Cargo.toml", 'name = "engine-style-parser"')}: fixture crate engine-style-parser is outside product escape-flow closure.`,
    `${sourceNeedleLocation("rust/crates/omena-benchmarks/Cargo.toml", 'name = "omena-benchmarks"')}: fixture crate omena-benchmarks and repository examples are outside product escape-flow closure.`,
    `${sourceNeedleLocation("rust/crates/omena-syntax/src/ident.rs", "impl ClassNameV0 {")}: the ClassNameV0 raw/into_raw/decoded/same_as identity plane belongs to selector authority and is not property identity.`,
    `${sourceNeedleLocation("rust/crates/omena-syntax/src/ident.rs", "pub fn canonical_name(&self) -> &str {")}: canonical_name()/sealed-key as_str() text comparisons are not escape flows; they are canonical-correct only within a kind, and standard("--foo") collides textually with custom("--foo").`,
    `${sourceNeedleLocation(checkerPath, "function discoverAuthoredEscapeClosureAudit(): AuthoredEscapeClosureAuditResult {")}: cross-crate call flow and dyn dispatch are not resolved by the intra-crate text fixpoint.`,
    `${sourceNeedleLocation(checkerPath, "const definitionCounts = new Map<string, number>();")}: ${unresolvedCallEdgeCount} ambiguous intra-crate call edges remain separately enumerated with file:line; same-name definitions, trait/inherent ambiguity, unknown receivers, and indirect function bindings are tainted by default.`,
    `${sourceNeedleLocation(checkerPath, "function discoverResidualRawPropertyCarrierSites(")}: key construction whose input was normalized before the authority constructor is visible only when the residual provenance resolver recognizes that normalization; any unrecognized pre-normalization remains outside the claim.`,
    `${sourceNeedleLocation(checkerPath, "function maskCommentsStringsAndTestItems(")}: proc-macro-generated bodies other than serde derives and unsafe transmute are outside the text scanner.`,
    `${sourceNeedleLocation(checkerPath, 'const generatedFixtureOnly = process.argv.includes("--generated-fixture-only");')}: generated matrix cells are scanner-only and are never compiled.`,
    `${scopeAnchor}: non-Rust consumers of Serialize egress in packages/, N-API, and WASM are outside Rust source-flow closure.`,
  ];
}

function lineNumberAt(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function digest(value: unknown): string {
  return `sha256:${createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}

function readExistingCensus(): IdentifierAuthorityCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as IdentifierAuthorityCensus;
  assert.equal(parsed.schemaVersion, "0", "identifier census schemaVersion");
  assert.equal(parsed.product, "omena.identifier-authority.census", "identifier census product");
  assert.equal(parsed.policy.expectedSide, "committed-authored-table", "expected-side policy");
  if (writeMode) {
    assert.ok(
      ["forbidden", "explicit-review-flag-required"].includes(parsed.policy.addedSiteAdoption),
      "added-site policy",
    );
  } else {
    assert.equal(
      parsed.policy.addedSiteAdoption,
      "explicit-review-flag-required",
      "added-site policy",
    );
  }
  assert.equal(parsed.egress.currentSiteCount, parsed.egress.sites.length, "egress site count");
  assert.equal(parsed.egress.siteDigest, digest(parsed.egress.sites), "egress site digest");
  assert.equal(parsed.idiom.currentSiteCount, parsed.idiom.sites.length, "idiom site count");
  assert.equal(parsed.idiom.siteDigest, digest(parsed.idiom.sites), "idiom site digest");
  assert.equal(
    parsed.predicateCopies.currentSiteCount,
    parsed.predicateCopies.sites.length,
    "predicate-copy site count",
  );
  assert.equal(
    parsed.predicateCopies.siteDigest,
    digest(parsed.predicateCopies.sites),
    "predicate-copy site digest",
  );
  assert.ok(parsed.propertyIdentity, "committed property-identity census is required");
  assert.equal(
    parsed.propertyIdentity.authoritySiteCount,
    parsed.propertyIdentity.sites.length,
    "property authority site count",
  );
  assert.equal(
    parsed.propertyIdentity.siteDigest,
    digest(parsed.propertyIdentity.sites),
    "property authority site digest",
  );
  assert.ok(
    parsed.propertyIdentity.residualRawCarrierSites,
    "committed residual raw property carriers are required",
  );
  assert.equal(
    parsed.propertyIdentity.residualRawCarrierSiteCount,
    parsed.propertyIdentity.residualRawCarrierSites.length,
    "residual raw property carrier site count",
  );
  assert.equal(
    parsed.propertyIdentity.residualRawCarrierSiteDigest,
    digest(parsed.propertyIdentity.residualRawCarrierSites),
    "residual raw property carrier site digest",
  );
  if (parsed.propertyIdentity.residualClassCounts) {
    for (const residualClass of [
      "entry-parameter",
      "static-standard-literal",
      "fixture-crate",
      "canonical-text-carrier",
      "non-property",
    ] as const) {
      assert.equal(
        parsed.propertyIdentity.residualClassCounts[residualClass],
        parsed.propertyIdentity.residualRawCarrierSites.filter(
          (site) => site.residualClass === residualClass,
        ).length,
        `residual class count: ${residualClass}`,
      );
    }
  }
  if (parsed.propertyIdentity.residualProvenanceCounts) {
    for (const provenance of ["p-authored", "p-canonical", "p-non-property"] as const) {
      assert.equal(
        parsed.propertyIdentity.residualProvenanceCounts[provenance],
        parsed.propertyIdentity.residualRawCarrierSites.filter(
          (site) => site.provenance === provenance,
        ).length,
        `residual provenance count: ${provenance}`,
      );
    }
  }
  assert.equal(
    parsed.propertyIdentity.rawStringIdentitySiteCount,
    0,
    "raw property identity site count",
  );
  assert.equal(
    parsed.propertyIdentity.rawStringIdentitySiteDigest,
    digest(parsed.propertyIdentity.rawStringIdentitySites),
    "raw property identity site digest",
  );
  return parsed;
}
