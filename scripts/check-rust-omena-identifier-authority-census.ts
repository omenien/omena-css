import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { assertRustCfgTestMaskContract, maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

assertRustCfgTestMaskContract();

const cfgTestMaskProbe = maskRustCfgTestItems(
  `#[cfg(any(test, feature = "test-support"))]\nfn compound_test_item() { let _ = ("}", b'{'); }\nfn production_after_cfg() {}`,
);
assert.doesNotMatch(
  cfgTestMaskProbe,
  /compound_test_item/u,
  "compound cfg(test) items must be excluded from production authority scans",
);
assert.match(
  cfgTestMaskProbe,
  /production_after_cfg/u,
  "a compound cfg(test) item must not mask adjacent production source",
);
assert.match(
  maskRustCfgTestItems("/// #[cfg(test)]\nfn documented_production_item() {}"),
  /documented_production_item/u,
  "a cfg(test) spelling inside a doc comment must not mask production source",
);
assert.equal(
  hasDebugFormatSpecifier('"{}", fallible()?'),
  false,
  "a try operator in a non-Debug format argument must not become a Debug escape",
);
assert.equal(
  hasDebugFormatSpecifier('"{0:x?}", value'),
  true,
  "a positional hexadecimal Debug format specifier must remain an escape",
);
assert.deepEqual(
  debugFormatOperands('r"prefix, {:?}", value'),
  ["value"],
  "raw Debug format strings and embedded commas must preserve the positional operand",
);

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

type CountedCannotSeeId =
  | "collection-callback-comparison-grammar"
  | "carrier-field-read-spelling-grammar"
  | "carrier-type-qualification-gate"
  | "place-expression-mutation-grammar"
  | "assignment-form-container-write"
  | "non-string-container-scalar-egress";

interface CountedCannotSeeBoundary {
  readonly id: CountedCannotSeeId;
  readonly direction: "decrease-only";
  readonly operations: readonly string[];
  readonly siteCount: number;
  readonly sites: readonly CensusSite[];
  readonly siteDigest: string;
  readonly reason: string;
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
  readonly bindingFormsSearched?: readonly AuthoredOriginId[];
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
  readonly reachingIdentitySinks: readonly string[];
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
  readonly sanctionReason?: string;
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
      readonly disposition:
        | "reviewed-structural-refactor"
        | "reviewed-indexed-custom-property-resolution"
        | "reviewed-cfg-test-inventory-mask";
      readonly reason: string;
    }[];
    readonly inventoryMaskCorrection: {
      readonly disposition: "reviewed-cfg-test-inventory-mask";
      readonly previous: {
        readonly authority: 732;
        readonly authoredEscape: 526;
        readonly writeInto: 70;
        readonly residualConsumer: 255;
      };
      readonly current: {
        readonly authority: number;
        readonly authoredEscape: number;
        readonly writeInto: number;
        readonly residualConsumer: number;
      };
      readonly removedRowCount: number;
      readonly reason: string;
    };
    readonly authoritySiteCount: number;
    readonly sites: readonly PropertyIdentitySite[];
    readonly siteDigest: string;
    readonly residualRawCarrierDirection: "decrease-only";
    readonly residualRawCarrierSiteCount: number;
    readonly residualRawCarrierSites: readonly CensusSite[];
    readonly residualRawCarrierSiteDigest: string;
    readonly residualClassCounts: Readonly<Record<ResidualPropertyClass, number>>;
    readonly residualProvenanceCounts: Readonly<Record<ResidualPropertyProvenance, number>>;
    readonly residualCarrierConsumerDerivation: "resolved-type-access-parameter-and-origin-binding-scan";
    readonly residualCarrierConsumerRows: readonly ResidualPropertyCarrierConsumerRow[];
    readonly residualCarrierConsumerSiteCount: number;
    readonly residualIdentityShapedConsumerCount: 0;
    readonly residualCarrierConsumerDigest: string;
    readonly rawStringIdentitySiteCount: 0;
    readonly rawStringIdentitySites: readonly [];
    readonly rawStringIdentitySiteDigest: string;
    readonly egressHonestyTable: {
      readonly basis: "registered-zero-branch-evidence-gates";
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
      readonly escapeCoveringCellCount: 1900;
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
      readonly unresolvedBindingEdges: readonly CensusSite[];
      readonly unresolvedBindingEdgeCount: number;
      readonly unresolvedBindingEdgeDigest: string;
      readonly carrierAudit: readonly AuthoredIdentityCarrierAudit[];
      readonly carrierAuditDigest: string;
      readonly identityDeriveViolationCount: 0;
      readonly countedCannotSee: readonly CountedCannotSeeBoundary[];
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
const injectResidualBindingFormOmission =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DROP_RESIDUAL_EMPTY_BINDING_FORM === "1";
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
const injectInlineEscapeIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_INLINE_ESCAPE_IDENTITIES === "1";
const injectPatternEscapeIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PATTERN_ESCAPE_IDENTITIES === "1";
const injectNestedEscapeIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_NESTED_ESCAPE_IDENTITIES === "1";
const injectMultilineDebugEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MULTILINE_DEBUG_ESCAPE_IDENTITY === "1";
const injectEscapeAliasAndCallIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_ESCAPE_ALIAS_AND_CALL_IDENTITIES === "1";
const injectUnresolvedWriteIntoSink =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNRESOLVED_WRITE_INTO_SINK === "1";
const injectCfgNotTestEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CFG_NOT_TEST_ESCAPE_IDENTITY === "1";
const injectContainerMutationEscapeIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CONTAINER_MUTATION_ESCAPE_IDENTITIES === "1";
const injectUnsupportedReceiverMutation =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNSUPPORTED_RECEIVER_MUTATION === "1";
const injectCarrierFieldIdentityConsumer =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CARRIER_FIELD_IDENTITY_CONSUMER === "1";
const injectCarrierFieldFlowIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CARRIER_FIELD_FLOW_IDENTITIES === "1";
const injectCarrierDelegationGuardLaundering =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CARRIER_DELEGATION_GUARD_LAUNDERING === "1";
const injectUnqualifiedCarrierEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNQUALIFIED_CARRIER_ESCAPE_IDENTITY === "1";
const injectOutParameterEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_OUT_PARAMETER_ESCAPE_IDENTITY === "1";
const injectArgumentParameterEscapeIdentity =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_ARGUMENT_PARAMETER_ESCAPE_IDENTITY === "1";
const injectWriteMacroDebugEscapeIdentities =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_WRITE_MACRO_DEBUG_ESCAPE_IDENTITIES === "1";
const generatedFixtureManifestPath =
  process.env.OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST;
const generatedFixtureOnly = process.argv.includes("--generated-fixture-only");
const injectGeneratedForLoopDetectorDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_PAIRED_DETECTOR_FOR_LOOP_ARM === "1";
const injectGeneratedArgumentReturnDetectorDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_PAIRED_DETECTOR_ARGUMENT_RETURN_ARM === "1";
const injectGeneratedEntryPointIdDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ENTRY_POINT_ID === "1";
const injectUnregisteredSerdeFrontend =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNREGISTERED_SERDE_FRONTEND === "1";
const injectZeroBranchGateRegistryDeletion =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_ZERO_BRANCH_GATE_REGISTRY_ENTRY === "1";
const deleteInventoryBuildCfgMask =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_DELETE_INVENTORY_BUILD_CFG_MASK === "1";

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

interface ResidualOriginBindingSearcher {
  readonly id: AuthoredOriginId;
  readonly pattern: RegExp;
}

const residualOriginBindingSearchers: readonly ResidualOriginBindingSearcher[] = [
  { id: "field-access", pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\b/u },
  { id: "tuple-field-access", pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*\.\s*\d+\b/u },
  {
    id: "bare-parameter",
    pattern: /\bfn\s+[A-Za-z_][A-Za-z0-9_]*[^({]*\([^)]*\b[A-Za-z_][A-Za-z0-9_]*\s*:/u,
  },
  {
    id: "fqn-parameter",
    pattern:
      /\b[A-Za-z_][A-Za-z0-9_]*\s*:\s*&?\s*[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+/u,
  },
  { id: "alias-parameter", pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*:\s*&?\s*[A-Z][A-Za-z0-9_]*\b/u },
  { id: "generic-bound-parameter", pattern: /(?:<[^>]*\bAsRef\s*<|\bwhere\b[^{}]*\bAsRef\s*<)/su },
  { id: "self-receiver", pattern: /(?:\bself\b|&\s*(?:mut\s+)?self\b)/u },
  {
    id: "closure-inferred",
    pattern: /\|\s*(?:&\s*)?(?:mut\s+)?[A-Za-z_][A-Za-z0-9_]*(?:\s*,[^|]*)?\s*\|/u,
  },
  { id: "for-loop-binding", pattern: /\bfor\s+[A-Za-z_][A-Za-z0-9_]*\s+in\b/u },
  { id: "while-let-binding", pattern: /\bwhile\s+let\b/u },
  { id: "if-let-binding", pattern: /\bif\s+let\b/u },
  { id: "let-else-binding", pattern: /\blet\s+[^;=]+\s*=\s*[^;]+\s+else\s*\{/u },
  { id: "let-chain-binding", pattern: /\bif\s+let\b[^{}]+&&\s*let\b/su },
  { id: "match-arm-binding", pattern: /\bmatch\b[\s\S]*?=>/u },
  { id: "at-binding", pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*@\s*/u },
  {
    id: "or-pattern-binding",
    pattern:
      /(?:\([^)]*\)|\b[A-Za-z_][A-Za-z0-9_]*)\s*\|\s*(?:\([^)]*\)|\b[A-Za-z_][A-Za-z0-9_]*)\s*(?:if|=>)/u,
  },
  { id: "struct-destructuring", pattern: /\b[A-Z][A-Za-z0-9_:]*\s*\{[^{}]*\}\s*=/u },
  { id: "tuple-destructuring", pattern: /\blet\s*\([^)]*,[^)]*\)\s*=/u },
  { id: "slice-pattern", pattern: /\[(?:[^\]]*,)?\s*\.\.\s*\]/u },
  { id: "two-statement-local", pattern: /\blet\b[^;]+;\s*let\b[^;]+=/su },
  {
    id: "wrapper-function",
    pattern: /\blet\s+[A-Za-z_][A-Za-z0-9_]*\s*=\s*[A-Za-z_][A-Za-z0-9_]*\s*\(/u,
  },
  {
    id: "two-step-wrapper",
    pattern: /=\s*[A-Za-z_][A-Za-z0-9_]*\s*\(\s*[A-Za-z_][A-Za-z0-9_]*\s*\(/u,
  },
  {
    id: "trait-method-return",
    pattern: /\b[A-Z][A-Za-z0-9_:]*\s*::\s*[A-Za-z_][A-Za-z0-9_]*\s*\(/u,
  },
  {
    id: "accessor-return-inline",
    pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\(\s*\)/u,
  },
  { id: "macro-rules-body", pattern: /\b[A-Za-z_][A-Za-z0-9_]*\s*!\s*\(/u },
  {
    id: "named-escape",
    pattern: /\b[A-Za-z_][A-Za-z0-9_]*(?:escape|render|write_into)[A-Za-z0-9_]*\s*[!(]/u,
  },
];
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
const debugFormatMacroGrammars = [
  { macroName: "format", escapeId: "debug-format-spec", formatArgumentIndex: 0 },
  { macroName: "format_args", escapeId: "format-args-debug", formatArgumentIndex: 0 },
  { macroName: "write", escapeId: "write-debug", formatArgumentIndex: 1 },
  { macroName: "writeln", escapeId: "writeln-debug", formatArgumentIndex: 1 },
] as const;
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
  debugFormatMacroGrammars[0].escapeId,
  "debug-fmt-ufcs",
  "dbg-macro",
  debugFormatMacroGrammars[1].escapeId,
  debugFormatMacroGrammars[2].escapeId,
  debugFormatMacroGrammars[3].escapeId,
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

assert.deepEqual(
  derivedEscapeAxisFromWorkspaceManifests(),
  authoredEscapeAxis,
  "serialization entry point has no escape id or the escape grammar shrank",
);

const sourceRoots = ["rust/crates"] as const;
const authorityInventoryDecreaseReviews = [
  {
    previousSiteCount: 732,
    currentSiteCount: 729,
    disposition: "reviewed-cfg-test-inventory-mask",
    reason:
      "The inventory build boundary now masks #[cfg(test)] items before all four scans. Three test-only authority rows are removed as a scope correction while raw-string identity remains zero.",
  },
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
const productionInventorySources: readonly MutableRustSource[] = productionSources.map(
  (relativePath) => ({
    relativePath,
    source: deleteInventoryBuildCfgMask
      ? readFileSync(path.join(repoRoot, relativePath), "utf8")
      : maskRustCfgTestItems(readFileSync(path.join(repoRoot, relativePath), "utf8")),
  }),
);
const authoredEscapeFixtureCrates = new Set([
  "omena-diff-test",
  "engine-style-parser",
  "omena-benchmarks",
]);
const authoredEscapeProductionSources = productionInventorySources.filter(
  (source) => !authoredEscapeFixtureCrates.has(source.relativePath.split("/")[2] ?? ""),
);
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
const residualSweepSources: MutableRustSource[] = productionInventorySources.map((source) => ({
  ...source,
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
const unresolvedTaintedReceiverMutations = authoredEscapeClosureAudit.unresolvedCallEdges.filter(
  (site) => site.operation === "unresolved-tainted-receiver-mutation",
);
assertReferenceSanctionedEscapeSites(authoredEscapeClosureAudit.escapeSites);
assertSerializedWholeValueEquivalenceSanctions(authoredEscapeClosureAudit.identityFlows);
assertInjectedEscapeProbeOutcomes(authoredEscapeClosureAudit);
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
process.stderr.write(`propertyAuthoritySiteCount=${propertyIdentitySites.length}\n`);
assert.deepEqual(
  authoredEscapeIdentityViolations,
  [],
  "authored-bearing escape result reached a property identity operation",
);
assert.deepEqual(
  unresolvedTaintedReceiverMutations,
  [],
  "authored-bearing escape reached a mutation outside the closed std receiver table",
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
process.stderr.write(`unresolvedWriteIntoSiteCount=${unresolvedWriteIntoSites.length}\n`);
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
for (const row of residualCarrierConsumerRows.filter(
  (candidate) => candidate.consumers.length === 0,
)) {
  assert.deepEqual(
    row.bindingFormsSearched,
    authoredOriginAxis,
    "consumerless residual carrier binding-form audit must equal the authored origin axis byte-for-byte",
  );
}
const residualCarrierConsumerSites = residualCarrierConsumerRows.flatMap((row) => row.consumers);
const injectedAuthorityMutationActive =
  !deleteInventoryBuildCfgMask &&
  Object.entries(process.env).some(
    ([name, value]) => name.startsWith("OMENA_IDENTIFIER_AUTHORITY_TEST_") && value === "1",
  );
const existingCountedCannotSeeBoundaries =
  existing?.propertyIdentity.authoredEscapeClosure.countedCannotSee ?? [];
const countedCannotSeeBoundaries = injectedAuthorityMutationActive
  ? existingCountedCannotSeeBoundaries
  : discoverCountedCannotSeeBoundaries(authoredEscapeProductionSources);
process.stderr.write(`residualCarrierConsumerSiteCount=${residualCarrierConsumerSites.length}\n`);
for (const boundary of countedCannotSeeBoundaries) {
  process.stderr.write(`countedCannotSee.${boundary.id}=${boundary.siteCount}\n`);
}
assertInventoryRowsOutsideCfgTestItems(propertyIdentitySites, "property authority");
assertInventoryRowsOutsideCfgTestItems(classifiedEgressSites, "egress");
assertInventoryRowsOutsideCfgTestItems(classifiedIdiomSites, "idiom");
assertInventoryRowsOutsideCfgTestItems(classifiedPredicateSites, "predicate copy");
assertInventoryRowsOutsideCfgTestItems(
  classifiedResidualRawPropertyCarrierSites,
  "residual raw property carrier",
);
assertInventoryRowsOutsideCfgTestItems(authoredEscapeClosureAudit.escapeSites, "authored escape");
assertInventoryRowsOutsideCfgTestItems(
  authoredEscapeClosureAudit.identityFlows,
  "authored escape identity flow",
);
assertInventoryRowsOutsideCfgTestItems(authoredEscapeClosureAudit.writeIntoSites, "write_into");
assertInventoryRowsOutsideCfgTestItems(
  authoredEscapeClosureAudit.unresolvedCallEdges,
  "unresolved escape call",
);
assertInventoryRowsOutsideCfgTestItems(
  authoredEscapeClosureAudit.unresolvedBindingEdges,
  "unresolved escape binding",
);
assertInventoryRowsOutsideCfgTestItems(residualCarrierConsumerSites, "residual consumer");
for (const boundary of countedCannotSeeBoundaries) {
  assertInventoryRowsOutsideCfgTestItems(boundary.sites, `counted boundary ${boundary.id}`);
}
const egressZeroBranchSurfaces = [
  { surface: "omena facts --json", evidenceGate: "rust/omena-cli-json-output-census" },
  {
    surface: "style-hover-candidates",
    evidenceGate: "rust/omena-lsp-server/style-provider-parity",
  },
  { surface: "N-API JSON fixtures", evidenceGate: "contract/parity-v2-golden" },
  { surface: "WASM JSON fixtures", evidenceGate: "contract/parity-v2-golden" },
  { surface: "omena CLI text", evidenceGate: "rust/omena-cli-json-output-census" },
  {
    surface: "LSP wire payloads",
    evidenceGate: "rust/omena-lsp-server/style-provider-parity",
  },
] as const;
assertZeroBranchEvidenceGatesRegistered(egressZeroBranchSurfaces);
const residualIdentityShapedConsumerCount = residualCarrierConsumerRows
  .filter((row) =>
    ["entry-parameter", "static-standard-literal"].includes(row.carrier.residualClass ?? ""),
  )
  .flatMap((row) => row.consumers)
  .filter((site) => site.classification === "identity-shaped").length;
assertCountedCannotSeeBoundariesDecreaseOnly(
  countedCannotSeeBoundaries,
  existing?.propertyIdentity.authoredEscapeClosure.countedCannotSee ?? [],
);
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
    if ("unresolvedBindingEdgeCount" in previousEscapeClosure) {
      assert.ok(
        authoredEscapeClosureAudit.unresolvedCallEdges.length <=
          previousEscapeClosure.unresolvedCallEdgeCount,
        `unresolved call-edge count increased: previous=${previousEscapeClosure.unresolvedCallEdgeCount} current=${authoredEscapeClosureAudit.unresolvedCallEdges.length}`,
      );
      assert.ok(
        authoredEscapeClosureAudit.unresolvedBindingEdges.length <=
          previousEscapeClosure.unresolvedBindingEdgeCount,
        `unresolved binding-edge count increased: previous=${previousEscapeClosure.unresolvedBindingEdgeCount} current=${authoredEscapeClosureAudit.unresolvedBindingEdges.length}; currentRows=${JSON.stringify(authoredEscapeClosureAudit.unresolvedBindingEdges)}`,
      );
    }
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
    const escapeSiteAdditions = addedAuthoredEscapeSites(
      previousEscapeClosure.escapeSites,
      authoredEscapeClosureAudit.escapeSites,
    );
    assert.ok(
      escapeSiteAdditions.length === 0 || acceptInventoryChange,
      `authored escape inventory gained ${escapeSiteAdditions.length} site(s); review each site and rerun with --accept-inventory-change`,
    );
    if (escapeSiteAdditions.length > 0) {
      assert.deepEqual(
        escapeSiteAdditions.filter((site) => site.disposition === "identity-violation"),
        [],
        "--accept-inventory-change cannot adopt an authored escape identity violation",
      );
      for (const site of escapeSiteAdditions) {
        process.stderr.write(
          `acceptedAuthoredEscapeSite=${JSON.stringify({
            path: site.path,
            line: site.line,
            function: site.function,
            escapeId: site.escapeId,
            disposition: site.disposition,
            operandDerivation: site.operandDerivation,
            evidence: site.evidence,
          })}\n`,
        );
      }
    }
    const sanctionedIdentityCount = authoredEscapeClosureAudit.identityFlows.filter(
      (flow) => flow.sanctioned,
    ).length;
    assert.ok(
      previousEscapeClosure.sanctionedIdentityCount === sanctionedIdentityCount ||
        acceptInventoryChange,
      "sanctioned escape inventory changed; rerun only after review with --accept-inventory-change",
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

function authoredEscapeInventoryKey(site: AuthoredEscapeSite): string {
  return [site.path, site.function, site.escapeId, site.evidence].join("\0");
}

function addedAuthoredEscapeSites(
  previous: readonly AuthoredEscapeSite[],
  current: readonly AuthoredEscapeSite[],
): readonly AuthoredEscapeSite[] {
  const availableCounts = new Map<string, number>();
  for (const site of previous) {
    const key = authoredEscapeInventoryKey(site);
    availableCounts.set(key, (availableCounts.get(key) ?? 0) + 1);
  }
  return current.filter((site) => {
    const key = authoredEscapeInventoryKey(site);
    const available = availableCounts.get(key) ?? 0;
    if (available === 0) return true;
    availableCounts.set(key, available - 1);
    return false;
  });
}

const escapeInventoryMultiplicityProbe = {
  path: "probe.rs",
  line: 1,
  function: "probe",
  escapeId: "debug-format-spec",
  evidence: 'format!("{property:?}")',
  operandClasses: ["non-property"],
  operandDerivation: "property=>non-property",
  disposition: "presentation",
  resultBindings: [],
} as const satisfies AuthoredEscapeSite;
assert.equal(
  addedAuthoredEscapeSites(
    [escapeInventoryMultiplicityProbe],
    [escapeInventoryMultiplicityProbe, { ...escapeInventoryMultiplicityProbe, line: 2 }],
  ).length,
  1,
  "authored escape inventory must count duplicate semantic sites instead of collapsing a set key",
);

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
    inventoryMaskCorrection: {
      disposition: "reviewed-cfg-test-inventory-mask",
      previous: {
        authority: 732,
        authoredEscape: 526,
        writeInto: 70,
        residualConsumer: 255,
      },
      current: {
        authority: propertyIdentitySites.length,
        authoredEscape: authoredEscapeClosureAudit.escapeSites.length,
        writeInto: authoredEscapeClosureAudit.writeIntoSites.length,
        residualConsumer: residualCarrierConsumerSites.length,
      },
      removedRowCount:
        732 +
        526 +
        70 +
        255 -
        propertyIdentitySites.length -
        authoredEscapeClosureAudit.escapeSites.length -
        authoredEscapeClosureAudit.writeIntoSites.length -
        residualCarrierConsumerSites.length,
      reason:
        "The four inventory builders now consume the same #[cfg(test)]-masked production source set. Removed rows are test-only scope corrections, not accepted product decreases.",
    },
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
    residualCarrierConsumerDerivation: "resolved-type-access-parameter-and-origin-binding-scan",
    residualCarrierConsumerRows,
    residualCarrierConsumerSiteCount: residualCarrierConsumerSites.length,
    residualIdentityShapedConsumerCount: 0,
    residualCarrierConsumerDigest: digest(residualCarrierConsumerRows),
    rawStringIdentitySiteCount: 0,
    rawStringIdentitySites: [],
    rawStringIdentitySiteDigest: digest([]),
    egressHonestyTable: {
      basis: "registered-zero-branch-evidence-gates",
      orderBranchSurfaces: [],
      valueBranchSurfaces: [],
      zeroBranchSurfaces: egressZeroBranchSurfaces,
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
      escapeCoveringCellCount: 1900,
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
      unresolvedBindingEdges: authoredEscapeClosureAudit.unresolvedBindingEdges,
      unresolvedBindingEdgeCount: authoredEscapeClosureAudit.unresolvedBindingEdges.length,
      unresolvedBindingEdgeDigest: digest(authoredEscapeClosureAudit.unresolvedBindingEdges),
      carrierAudit: authoredEscapeClosureAudit.carrierAudit,
      carrierAuditDigest: digest(authoredEscapeClosureAudit.carrierAudit),
      identityDeriveViolationCount: 0,
      countedCannotSee: countedCannotSeeBoundaries,
      cannotSee: authoredEscapeCannotSee(
        authoredEscapeClosureAudit.unresolvedCallEdges.length,
        authoredEscapeClosureAudit.unresolvedBindingEdges.length,
      ),
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
      !injectUnsupportedReceiverMutation &&
      !injectCarrierFieldFlowIdentities &&
      !injectCarrierDelegationGuardLaundering &&
      !injectUnqualifiedCarrierEscapeIdentity &&
      !deleteInventoryBuildCfgMask &&
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
  for (const inventorySource of productionInventorySources) {
    const relativePath = inventorySource.relativePath;
    let source = inventorySource.source;
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

function coalesceCountedCannotSeeSitesByScopeOperation(sites: readonly CensusSite[]): CensusSite[] {
  const byScopeOperation = new Map<string, CensusSite[]>();
  for (const site of sites) {
    const key = `${site.path}\u0000${site.function}\u0000${site.operation}`;
    const group = byScopeOperation.get(key) ?? [];
    group.push(site);
    byScopeOperation.set(key, group);
  }
  return [...byScopeOperation.values()]
    .map((group) => {
      const first = group.toSorted((left, right) => left.line - right.line)[0];
      assert.ok(first, "counted cannotSee coalescing group must be non-empty");
      return {
        ...first,
        evidence: [...new Set(group.map((site) => site.evidence))].join(" || "),
      };
    })
    .toSorted(
      (left, right) =>
        left.path.localeCompare(right.path) ||
        left.line - right.line ||
        left.operation.localeCompare(right.operation),
    );
}

function discoverRawPropertyIdentitySites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionInventorySources.map((source) => ({ ...source }));
  applyCarrierFieldConsumerProbeMutation(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  return uniqueSites([
    ...discoverTypedRawPropertyIdentitySites(),
    ...discoverAuthoredPropertyIdentityLaunderingSites(),
    ...discoverEscapePopulatedCarrierFieldIdentitySites(sources, structFields, aliasesByPath),
  ]);
}

function discoverAuthoredPropertyIdentityLaunderingSites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionInventorySources.map((source) => ({ ...source }));
  sources.push(
    ...(generatedFixtureManifest?.sources.map((source) => ({
      relativePath: source.relativePath,
      source: source.source,
    })) ?? []),
  );
  applyResidualCarrierProbeMutations(sources);
  applyCarrierFieldConsumerProbeMutation(sources);
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
  const enumVariantPayloadTypes = rustEnumVariantPayloadTypes(sources, aliasesByPath);
  const functionReturnTypes = uniqueRustFunctionReturnTypes(
    sources,
    aliasesByPath,
    structFieldTypes,
  );
  const residualCarrierFields = new Map<string, Set<string>>();
  for (const fieldKey of carrierByField.keys()) {
    const separator = fieldKey.indexOf(".");
    const typeName = fieldKey.slice(0, separator);
    const fields = residualCarrierFields.get(typeName) ?? new Set<string>();
    fields.add(fieldKey.slice(separator + 1));
    residualCarrierFields.set(typeName, fields);
  }
  const authoredReturningFunctions = authoredReturningFunctionNames(sources, residualCarrierFields);
  const consumersByCarrier = new Map<CensusSite, ResidualPropertyConsumerSite[]>();
  const bindingFormsSearched = new Set<AuthoredOriginId>();
  for (const carrier of carriers) consumersByCarrier.set(carrier, []);

  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    for (const functionSlice of rustFunctionSlices(scannable)) {
      for (const origin of searchResidualOriginBindingForms(
        `${functionSlice.signature}{${functionSlice.scannable}}`,
      )) {
        bindingFormsSearched.add(origin);
      }
      const bindings = resolvedRustBindingsForFunction(
        functionSlice,
        scannable,
        aliases,
        structFieldTypes,
        {
          crateName: relativePath.split("/")[2] ?? "<unknown-crate>",
          enumVariantPayloadTypes,
          functionReturnTypes,
        },
      );
      const carriersByBinding = new Map<string, Set<CensusSite>>();
      const addBindingCarriers = (
        binding: string,
        bindingCarriers: ReadonlySet<CensusSite>,
      ): boolean => {
        const prior = carriersByBinding.get(binding) ?? new Set<CensusSite>();
        const merged = new Set([...prior, ...bindingCarriers]);
        if (merged.size === prior.size) return false;
        carriersByBinding.set(binding, merged);
        return true;
      };
      const authorityShadowEnd = (binding: string): number | undefined => {
        const shadow = functionSlice.scannable.match(
          new RegExp(
            String.raw`\blet\s+${escapeRegExp(binding)}\s*=\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*PropertyNameV0\s*::[^;]*\b${escapeRegExp(binding)}\b[^;]*;`,
            "u",
          ),
        );
        return shadow ? (shadow.index ?? 0) + shadow[0].length : undefined;
      };
      const expressionCarriesResidualText = (expression: string): boolean => {
        const value = expression.trim();
        if (sealedAuthorityExpression(value)) return false;
        if (rawStringValueExpression(value)) return true;
        if (
          /^\(?\s*&?[A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*(?:[A-Za-z_][A-Za-z0-9_]*|\d+))*(?:\s*,\s*&?[A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*(?:[A-Za-z_][A-Za-z0-9_]*|\d+))*)+\s*\)?$/u.test(
            value,
          )
        ) {
          return true;
        }
        if (
          /^(?:if\s+let|match\b)[\s\S]*(?:=>|\{)\s*&?[A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*(?:[A-Za-z_][A-Za-z0-9_]*|\d+))*\s*(?:,|\}|else\b)/u.test(
            value,
          )
        ) {
          return true;
        }
        if (
          /^(?:format|dbg|[A-Za-z_][A-Za-z0-9_]*(?:escape|render|write_into)[A-Za-z0-9_]*)\s*!?\s*\(/u.test(
            value,
          )
        ) {
          return true;
        }
        const called = value.match(
          /^\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*\(/u,
        )?.[1];
        return called !== undefined && authoredReturningFunctions.has(called);
      };
      const constructedReceiverType = (binding: string): string | undefined => {
        const pattern = new RegExp(
          String.raw`\b${escapeRegExp(binding)}\s*=\s*(?:Some\s*\(\s*)?(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*\{`,
          "u",
        );
        return functionSlice.scannable.match(pattern)?.[1];
      };
      const carriersForExpression = (
        expression: string,
        expressionOffset = 0,
      ): ReadonlySet<CensusSite> => {
        const expressionCarriers = new Set<CensusSite>();
        for (const [binding, bindingCarriers] of carriersByBinding) {
          const shadowEnd = authorityShadowEnd(binding);
          if (shadowEnd !== undefined && expressionOffset >= shadowEnd) continue;
          if (new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(expression)) {
            for (const carrier of bindingCarriers) expressionCarriers.add(carrier);
          }
        }
        for (const access of expression.matchAll(
          /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\b/gu,
        )) {
          const carrier = [bindings.get(access[1]), constructedReceiverType(access[1])]
            .filter((receiverType): receiverType is string => receiverType !== undefined)
            .map((receiverType) => carrierByField.get(`${receiverType}.${access[2]}`))
            .find((candidate): candidate is CensusSite => candidate !== undefined);
          if (carrier) expressionCarriers.add(carrier);
        }
        return expressionCarriers;
      };

      for (const [key, carrier] of carrierByParameter) {
        const [carrierPath, functionAndParameter] = key.split("\u0000");
        if (carrierPath !== relativePath) continue;
        const separator = functionAndParameter.lastIndexOf(".");
        if (functionAndParameter.slice(0, separator) !== functionSlice.name) continue;
        addBindingCarriers(functionAndParameter.slice(separator + 1), new Set([carrier]));
      }

      const functionText = `${functionSlice.signature}{${functionSlice.scannable}}`;
      for (const [fieldKey, carrier] of carrierByField) {
        const separator = fieldKey.indexOf(".");
        const typeName = fieldKey.slice(0, separator);
        const fieldName = fieldKey.slice(separator + 1);
        const destructuring = new RegExp(
          `\\b${escapeRegExp(typeName)}\\s*\\{([^{}]*)\\}\\s*(?:=|=>|:)`,
          "gu",
        );
        for (const pattern of functionText.matchAll(destructuring)) {
          const field = pattern[1].match(
            new RegExp(
              `(?:^|,)\\s*${escapeRegExp(fieldName)}(?:\\s*:\\s*([A-Za-z_][A-Za-z0-9_]*))?(?:\\s*,|$)`,
              "u",
            ),
          );
          if (!field) continue;
          addBindingCarriers(field[1] ?? fieldName, new Set([carrier]));
        }
      }

      for (let pass = 0; pass <= functionSlice.scannable.length + 1; pass += 1) {
        let changed = false;
        for (const assignment of functionSlice.scannable.matchAll(
          /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;]+)?\s*=\s*([^;]+);/gu,
        )) {
          if (!expressionCarriesResidualText(assignment[2])) continue;
          changed =
            addBindingCarriers(
              assignment[1],
              carriersForExpression(assignment[2], assignment.index),
            ) || changed;
        }
        for (const assignment of functionSlice.scannable.matchAll(
          /\blet\s*\(([^)]*)\)\s*=\s*([^;]+);/gu,
        )) {
          if (!expressionCarriesResidualText(assignment[2])) continue;
          const expressionCarriers = carriersForExpression(assignment[2], assignment.index);
          for (const binding of rustPatternBindings(assignment[1])) {
            changed = addBindingCarriers(binding, expressionCarriers) || changed;
          }
        }
        for (const assignment of functionSlice.scannable.matchAll(
          /\blet\s*\[([^\]]*)\]\s*=\s*([^;]+);/gu,
        )) {
          if (!expressionCarriesResidualText(assignment[2])) continue;
          const expressionCarriers = carriersForExpression(assignment[2], assignment.index);
          for (const binding of rustPatternBindings(assignment[1])) {
            changed = addBindingCarriers(binding, expressionCarriers) || changed;
          }
        }
        for (const binding of functionSlice.scannable.matchAll(
          /\b(?:if|while)\s+let\s+(?:Some\s*\()?([A-Za-z_][A-Za-z0-9_]*)\)?\s*=\s*([^{]+)/gu,
        )) {
          changed =
            addBindingCarriers(binding[1], carriersForExpression(binding[2], binding.index)) ||
            changed;
        }
        for (const binding of functionSlice.scannable.matchAll(
          /\bfor\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([^{]+)/gu,
        )) {
          changed =
            addBindingCarriers(binding[1], carriersForExpression(binding[2], binding.index)) ||
            changed;
        }
        for (const closure of functionSlice.scannable.matchAll(
          /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.(?:iter|into_iter)\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|;{}]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
        )) {
          changed =
            addBindingCarriers(closure[2], carriersForExpression(closure[1], closure.index)) ||
            changed;
        }
        for (const closure of functionSlice.scannable.matchAll(
          /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.(?:iter|into_iter)\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|;{}]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|([^|]+)\|/gu,
        )) {
          const expressionCarriers = carriersForExpression(closure[1], closure.index);
          for (const binding of rustPatternBindings(closure[2])) {
            changed = addBindingCarriers(binding, expressionCarriers) || changed;
          }
        }
        if (!changed) break;
        assert.ok(
          pass < functionSlice.scannable.length,
          `residual carrier binding flow did not converge in ${functionSlice.name}`,
        );
      }

      for (const access of functionSlice.scannable.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\b/gu,
      )) {
        const carrier = [bindings.get(access[1]), constructedReceiverType(access[1])]
          .filter((receiverType): receiverType is string => receiverType !== undefined)
          .map((receiverType) => carrierByField.get(`${receiverType}.${access[2]}`))
          .find((candidate): candidate is CensusSite => candidate !== undefined);
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

      for (const [binding, bindingCarriers] of carriersByBinding) {
        const bindingUse = new RegExp(`\\b${escapeRegExp(binding)}\\b`, "gu");
        const shadowEnd = authorityShadowEnd(binding);
        for (const use of functionSlice.scannable.matchAll(bindingUse)) {
          if (shadowEnd !== undefined && use.index >= shadowEnd) continue;
          const offset = functionSlice.bodyStart + use.index;
          for (const carrier of bindingCarriers) {
            consumersByCarrier
              .get(carrier)
              ?.push(
                residualConsumerSiteAt(
                  relativePath,
                  source,
                  scannable,
                  offset,
                  functionSlice,
                  binding,
                ),
              );
          }
        }
      }
    }
  }

  const searchedBindingForms = residualOriginBindingSearchers
    .map((searcher) => searcher.id)
    .filter((origin) => bindingFormsSearched.has(origin));
  return carriers.map((carrier) => {
    const consumers = uniqueSites(consumersByCarrier.get(carrier) ?? []);
    return consumers.length === 0
      ? { carrier, consumers, bindingFormsSearched: searchedBindingForms }
      : { carrier, consumers };
  });
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

function searchResidualOriginBindingForms(text: string): readonly AuthoredOriginId[] {
  const searchers = injectResidualBindingFormOmission
    ? residualOriginBindingSearchers.slice(1)
    : residualOriginBindingSearchers;
  const searched: AuthoredOriginId[] = [];
  for (const searcher of searchers) {
    searcher.pattern.test(text);
    searched.push(searcher.id);
  }
  return searched;
}

function rustPatternBindings(pattern: string): readonly string[] {
  const reserved = new Set([
    "Some",
    "None",
    "Ok",
    "Err",
    "mut",
    "ref",
    "self",
    "Self",
    "true",
    "false",
  ]);
  return [
    ...new Set(
      (pattern.match(/\b[A-Za-z_][A-Za-z0-9_]*\b/gu) ?? []).filter(
        (binding) => binding !== "_" && !reserved.has(binding) && !/^[A-Z]/u.test(binding),
      ),
    ),
  ];
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
    for (const structure of scannable.matchAll(
      /\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^;]*)\)\s*;/gu,
    )) {
      const fields = structures.get(structure[1]) ?? new Map<string, string>();
      for (const [index, segment] of topLevelSegments(structure[2], ",").entries()) {
        const rustType = segment.text.trim().replace(/^pub(?:\([^)]*\))?\s+/u, "");
        if (rustType) fields.set(`tuple_${index}`, normalizeRustType(rustType, aliases));
      }
      structures.set(structure[1], fields);
    }
    for (const alias of scannable.matchAll(/\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);/gu)) {
      structures.set(alias[1], new Map([["alias_target", normalizeRustType(alias[2], aliases)]]));
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

function rustEnumVariantPayloadTypes(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, string> {
  const variants = new Map<string, string>();
  for (const { relativePath, source } of sources) {
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const enumeration of scannable.matchAll(/\benum\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = enumeration.index + enumeration[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const variant of body.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*([^,)]+)(?:,[^)]*)?\)/gu,
      )) {
        variants.set(`${enumeration[1]}::${variant[1]}`, normalizeRustType(variant[2], aliases));
      }
    }
  }
  return variants;
}

interface RustBindingResolutionContext {
  readonly crateName: string;
  readonly enumVariantPayloadTypes: ReadonlyMap<string, string>;
  readonly functionReturnTypes: ReadonlyMap<string, string>;
}

function resolvedRustBindingsForFunction(
  functionSlice: RustFunctionSlice,
  source: string,
  aliases: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  context?: RustBindingResolutionContext,
): ReadonlyMap<string, string> {
  const bindings = new Map<string, string>();
  const text = `${functionSlice.signature}{${functionSlice.scannable}}`;
  const normalizeResolvedType = (typeSource: string): string => {
    if (!context) return normalizeRustType(typeSource, aliases);
    const identifiers = typeSource.match(/[A-Za-z_][A-Za-z0-9_]*/gu) ?? [];
    for (const identifier of identifiers) {
      const resolved = aliases.get(identifier) ?? identifier;
      if (structFields.has(resolved)) return resolved;
    }
    return normalizeRustType(typeSource, aliases);
  };
  const explicitlyTypedBindingSource = context ? functionSlice.signature : text;
  for (const binding of explicitlyTypedBindingSource.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,)=;{]+(?:<[^;{=]+>)?)/gu,
  )) {
    bindings.set(binding[1], normalizeResolvedType(binding[2]));
  }
  if (context) {
    for (const binding of functionSlice.scannable.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^=;]+)\s*=/gu,
    )) {
      bindings.set(binding[1], normalizeResolvedType(binding[2]));
    }
    for (const binding of functionSlice.scannable.matchAll(
      /(?:\||,)\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,|]+)(?=,|\|)/gu,
    )) {
      bindings.set(binding[1], normalizeResolvedType(binding[2]));
    }
  }
  const selfType = enclosingImplType(source, functionSlice.bodyStart, aliases);
  if (selfType) bindings.set("self", selfType);

  const genericAsRefTargets = new Map<string, string>();
  for (const generic of functionSlice.signature.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*AsRef\s*<\s*([^>]+)>/gu,
  )) {
    genericAsRefTargets.set(generic[1], normalizeResolvedType(generic[2]));
  }

  const inferredExpressionType = (expression: string): string | undefined => {
    const value = expression
      .trim()
      .replace(/^&\s*(?:mut\s+)?/u, "")
      .replace(/\.(?:clone|as_ref)\s*\(\s*\)\s*$/u, "")
      .trim();
    const some = value.match(/^Some\s*\(\s*([\s\S]*)\s*\)$/u);
    if (some) return inferredExpressionType(some[1]);
    const structure = value.match(
      /^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*\{/u,
    )?.[1];
    if (structure) return aliases.get(structure) ?? structure;
    const associatedConstructor = value.match(
      /^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*::\s*(?:default|new)\s*\(/u,
    )?.[1];
    if (associatedConstructor) return aliases.get(associatedConstructor) ?? associatedConstructor;
    const variant = value.match(
      /^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*::\s*([A-Z][A-Za-z0-9_]*)\s*\(/u,
    );
    if (variant) return context?.enumVariantPayloadTypes.get(`${variant[1]}::${variant[2]}`);
    if (/^[A-Za-z_][A-Za-z0-9_]*$/u.test(value)) return bindings.get(value);
    const access = value.match(/^([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)/u)?.[1];
    if (access) {
      const resolved = resolveRustAccessType(access, bindings, structFields);
      if (resolved) return normalizeResolvedType(resolved);
    }
    const asRef = value.match(/^([A-Za-z_][A-Za-z0-9_]*)\.as_ref\s*\(\s*\)/u)?.[1];
    if (asRef) {
      const genericType = bindings.get(asRef);
      if (genericType && genericAsRefTargets.has(genericType)) {
        return genericAsRefTargets.get(genericType);
      }
    }
    const call = value.match(
      /^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*\(/u,
    )?.[1];
    if (call && context) {
      const returnType = context.functionReturnTypes.get(`${context.crateName}\u0000${call}`);
      if (returnType) return normalizeResolvedType(returnType);
    }
    const method = value.match(/\.([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)\s*$/u)?.[1];
    if (method && context) {
      const returnType = context.functionReturnTypes.get(`${context.crateName}\u0000${method}`);
      if (returnType) return normalizeResolvedType(returnType);
    }
    return undefined;
  };

  for (let pass = 0; pass <= functionSlice.scannable.length + 1; pass += 1) {
    let changed = false;
    const setBinding = (binding: string, rustType: string | undefined): void => {
      if (rustType && !bindings.has(binding)) {
        bindings.set(binding, rustType);
        changed = true;
      }
    };

    for (const closure of functionSlice.scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.(?:iter|into_iter)\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|;{}]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
    )) {
      if (!context && closure[0].includes(".into_iter")) continue;
      const receiverType = resolveRustAccessType(closure[1], bindings, structFields);
      const elementType = rustCollectionElementType(receiverType, aliases) ?? receiverType;
      setBinding(closure[2], elementType && normalizeResolvedType(elementType));
    }
    if (context) {
      for (const closure of functionSlice.scannable.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.(?:iter|into_iter)\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|;{}]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|([^|]+)\|/gu,
      )) {
        const receiverType = resolveRustAccessType(closure[1], bindings, structFields);
        const elementType = rustCollectionElementType(receiverType, aliases) ?? receiverType;
        for (const binding of rustPatternBindings(closure[2])) {
          setBinding(binding, elementType && normalizeResolvedType(elementType));
        }
      }
    }
    if (!context) {
      if (!changed) break;
      continue;
    }
    for (const loop of functionSlice.scannable.matchAll(
      /\bfor\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([^{]+)/gu,
    )) {
      setBinding(loop[1], inferredExpressionType(loop[2]));
    }
    for (const assignment of functionSlice.scannable.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;]+)?\s*=\s*([^;]+);/gu,
    )) {
      setBinding(assignment[1], inferredExpressionType(assignment[2]));
    }
    for (const assignment of functionSlice.scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*)\s*=\s*Some\s*\(\s*((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Z][A-Za-z0-9_]*\s*[{])/gu,
    )) {
      setBinding(assignment[1], inferredExpressionType(assignment[2]));
    }
    for (const optional of functionSlice.scannable.matchAll(
      /\b(?:if|while)\s+let\s+Some\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*=\s*([^{]+)/gu,
    )) {
      setBinding(optional[1], inferredExpressionType(optional[2]));
    }
    for (const optional of functionSlice.scannable.matchAll(
      /\blet\s+Some\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*=\s*([^;]+?)(?:\s+else\s*\{|;)/gu,
    )) {
      setBinding(optional[1], inferredExpressionType(optional[2]));
    }
    for (const variant of functionSlice.scannable.matchAll(
      /\b([A-Z][A-Za-z0-9_]*)\s*::\s*([A-Z][A-Za-z0-9_]*)\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)/gu,
    )) {
      setBinding(variant[3], context?.enumVariantPayloadTypes.get(`${variant[1]}::${variant[2]}`));
    }
    for (const destructuring of functionSlice.scannable.matchAll(
      /\b([A-Z][A-Za-z0-9_]*)\s*\{([^{}]*)\}\s*(?:=|=>|:)/gu,
    )) {
      const fields = structFields.get(aliases.get(destructuring[1]) ?? destructuring[1]);
      if (!fields) continue;
      for (const field of destructuring[2].matchAll(
        /(?:^|,)\s*([A-Za-z_][A-Za-z0-9_]*)(?:\s*:\s*([A-Za-z_][A-Za-z0-9_]*))?(?:\s*,|$)/gu,
      )) {
        setBinding(field[2] ?? field[1], fields.get(field[1]));
      }
    }
    for (const tuple of functionSlice.scannable.matchAll(
      /\blet\s*\(([^)]*)\)\s*=\s*\(([^;]*)\)\s*;/gu,
    )) {
      const targets = tuple[1].split(",").map((part) => part.trim());
      const values = tuple[2].split(",").map((part) => part.trim());
      for (const [index, target] of targets.entries()) {
        const binding = target.match(/(?:Some\s*\()?([A-Za-z_][A-Za-z0-9_]*)/u)?.[1];
        if (binding) setBinding(binding, inferredExpressionType(values[index] ?? ""));
      }
    }
    for (const atBinding of functionSlice.scannable.matchAll(
      /\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*@[^=]*=\s*([A-Za-z_][A-Za-z0-9_]*)/gu,
    )) {
      setBinding(atBinding[1], bindings.get(atBinding[2]));
    }
    if (!changed) break;
    assert.ok(
      pass < functionSlice.scannable.length,
      `Rust binding resolution did not converge in ${functionSlice.name}`,
    );
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
  const sources: MutableRustSource[] = productionInventorySources.map((source) => ({ ...source }));
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
      "fn injected_residual_identity_consumer(entry: &SelectorSpecificitySeedCase, expected: &str) -> bool { let SelectorSpecificitySeedCase { property, .. } = entry; *property == expected }",
      "rust/crates/omena-cascade/src/conformance.rs",
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
  let manifestText = manifestResult.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((relativePath) => readFileSync(path.join(repoRoot, relativePath), "utf8"))
    .join("\n");
  if (injectUnregisteredSerdeFrontend) manifestText += '\nserde_future = "1"\n';
  const registeredSerdeFrontends = new Set([
    "serde",
    "serde-wasm-bindgen",
    "serde_ignored",
    "serde_json",
    "serde_yaml_ng",
  ]);
  const discoveredSerdeFrontends = [
    ...new Set(
      [...manifestText.matchAll(/^\s*(serde[A-Za-z0-9_-]*)\s*(?:=|\.)/gmu)].map(
        (match) => match[1],
      ),
    ),
  ].toSorted();
  assert.deepEqual(
    discoveredSerdeFrontends.filter((name) => !registeredSerdeFrontends.has(name)),
    [],
    "serde front-end dependency is not registered as an escape family or a named non-egress helper",
  );
  const available = new Set<AuthoredEscapeId>([
    "write-into-call",
    "write-into-ufcs",
    "write-into-fn-pointer",
    "render-authored-helper",
    "debug-fmt-ufcs",
    "dbg-macro",
    "tracing-debug-sigil",
    "aliased-escape-path",
    ...debugFormatMacroGrammars.map((grammar) => grammar.escapeId),
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
    return injectGeneratedForLoopDetectorDeletion ? undefined : "for-loop-binding";
  if (/\bwhile\s+let\s+Some\s*\(\s*carrier\s*\)/u.test(scannable)) return "while-let-binding";
  if (/(?:^|[;{}])\s*let\s+Some\s*\(\s*carrier\s*\)\s*=.*\belse\b/su.test(scannable))
    return "let-else-binding";
  if (/\bif\s+let\s+Some\s*\(\s*carrier\s*\).*&&\s*let\b/su.test(scannable))
    return "let-chain-binding";
  if (/\bif\s+let\s+Some\s*\(\s*carrier\s*\)/u.test(scannable)) return "if-let-binding";
  if (/\bmatch\s+carriers\s*\.\s*first\s*\(\s*\)/u.test(scannable)) return "match-arm-binding";
  if (/\bauthored_carrier\s*@\s*_/u.test(scannable)) return "at-binding";
  if (
    /\(\s*Some\s*\(\s*bound\s*\)\s*,\s*_\s*\)\s*\|\s*\(\s*_\s*,\s*Some\s*\(\s*bound\s*\)\s*\)/u.test(
      scannable,
    )
  )
    return "or-pattern-binding";
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
    if (id === "argument-position-compare" && injectGeneratedArgumentReturnDetectorDeletion)
      continue;
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
  readonly unresolvedBindingEdges: readonly CensusSite[];
  readonly carrierAudit: readonly AuthoredIdentityCarrierAudit[];
  readonly identityDeriveViolationCount: number;
  readonly identityDeriveViolations: readonly string[];
}

interface AuthoredTaintSourceDraft {
  readonly siteIndex: number;
  readonly bodyOffset: number;
  readonly path: string;
  readonly line: number;
  readonly functionName: string;
  readonly evidence: string;
  readonly matchedText: string;
  readonly operandClasses: readonly AuthoredEscapeOperandClass[];
  readonly operandDerivation: string;
  readonly definiteAuthoredOperand: boolean;
  readonly resultBindings: readonly string[];
  readonly sourceKind: "escape" | "carrier-field-read";
}

interface EscapeOccurrenceDraft extends AuthoredTaintSourceDraft {
  readonly sourceKind: "escape";
  readonly escapeId: AuthoredEscapeId;
  readonly sinkBinding?: string;
}

interface CarrierFieldReadSourceDraft extends AuthoredTaintSourceDraft {
  readonly sourceKind: "carrier-field-read";
  readonly carrierType: string;
  readonly fieldName: "name";
}

interface EscapeFunctionRecord {
  readonly crateName: string;
  readonly implType?: string;
  readonly path: string;
  readonly source: string;
  readonly functionName: string;
  readonly signature: string;
  readonly body: string;
  readonly scannable: string;
  readonly bodyStart: number;
  readonly bindings: ReadonlyMap<string, string>;
  readonly occurrences: readonly AuthoredTaintSourceDraft[];
}

interface FunctionTaintResult {
  readonly bindingSources: ReadonlyMap<string, ReadonlySet<number>>;
  readonly containerSources: ReadonlyMap<string, ReadonlySet<number>>;
  readonly returnSources: ReadonlySet<number>;
  readonly mutatedParameterSources: ReadonlyMap<number, ReadonlySet<number>>;
  readonly unresolvedEdges: readonly CensusSite[];
  readonly shadowedBindings: ReadonlySet<string>;
}

interface EscapeFunctionParameter {
  readonly name: string;
  readonly mutableReference: boolean;
  readonly typeSource: string;
}

interface ResolvedEscapeCall {
  readonly key: string;
  readonly arguments: readonly string[];
}

function discoverAuthoredEscapeClosureAudit(): AuthoredEscapeClosureAuditResult {
  const sources: MutableRustSource[] = authoredEscapeProductionSources.map((source) => ({
    ...source,
  }));
  applyAuthoredEscapeProbeMutations(sources);
  applyCarrierFieldConsumerProbeMutation(sources);
  applyCarrierDelegationGuardProbeMutation(sources);
  assertMechanicalCarrierComparatorDelegations(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  const functionReturnTypes = uniqueRustFunctionReturnTypes(sources, aliasesByPath);
  const escapePopulatedCarrierTypes = escapePopulatedStringCarrierTypes(
    sources,
    structFields,
    aliasesByPath,
    functionReturnTypes,
  );
  const carrierFieldSourceTypes = manuallyIdentifiedEscapeCarrierTypes(
    sources,
    escapePopulatedCarrierTypes,
  );
  const records: EscapeFunctionRecord[] = [];
  const taintSourceDrafts: AuthoredTaintSourceDraft[] = [];
  const escapeOccurrenceDrafts: EscapeOccurrenceDraft[] = [];
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
        escapePopulatedCarrierTypes,
        taintSourceDrafts.length,
      );
      const carrierFieldReads = carrierFieldReadSourcesInFunction(
        relativePath,
        source,
        functionSlice,
        body,
        bindings,
        carrierFieldSourceTypes,
        taintSourceDrafts.length + occurrences.length,
      );
      taintSourceDrafts.push(...occurrences, ...carrierFieldReads);
      escapeOccurrenceDrafts.push(...occurrences);
      records.push({
        crateName: relativePath.split("/")[2] ?? "<unknown-crate>",
        implType: enclosingImplType(source, functionSlice.bodyStart, aliases),
        path: relativePath,
        source,
        functionName: functionSlice.name,
        signature: functionSlice.signature,
        body,
        scannable: functionSlice.scannable,
        bodyStart: functionSlice.bodyStart,
        bindings,
        occurrences: [...occurrences, ...carrierFieldReads],
      });
    }
  }

  const definitionCounts = new Map<string, number>();
  const recordsByCallableKey = new Map<string, EscapeFunctionRecord[]>();
  for (const record of records) {
    const key = escapeCallableKey(record.crateName, record.functionName, record.implType);
    definitionCounts.set(key, (definitionCounts.get(key) ?? 0) + 1);
    const definitions = recordsByCallableKey.get(key) ?? [];
    definitions.push(record);
    recordsByCallableKey.set(key, definitions);
  }
  const returnSummaries = new Map<string, ReadonlySet<number>>();
  const parameterInputSummaries = new Map<string, Map<number, ReadonlySet<number>>>();
  const mutatedParameterSummaries = new Map<string, Map<number, ReadonlySet<number>>>();
  let taintByFunction = new Map<EscapeFunctionRecord, FunctionTaintResult>();
  for (let pass = 0; pass <= records.length + taintSourceDrafts.length + 8; pass += 1) {
    let changed = false;
    const nextTaint = new Map<EscapeFunctionRecord, FunctionTaintResult>();
    for (const record of records) {
      const key = escapeCallableKey(record.crateName, record.functionName, record.implType);
      const taint = computeFunctionTaint(
        record,
        returnSummaries,
        definitionCounts,
        parameterInputSummaries.get(key),
        mutatedParameterSummaries,
      );
      nextTaint.set(record, taint);
      const prior = returnSummaries.get(key) ?? new Set<number>();
      const returnedType = escapeFunctionReturnType(record.signature);
      const returnedSources =
        returnedType && /\b(?:String|str|Value)\b/u.test(returnedType)
          ? taint.returnSources
          : new Set<number>();
      const merged = new Set([...prior, ...returnedSources]);
      if (merged.size !== prior.size) {
        returnSummaries.set(key, merged);
        changed = true;
      }
      const priorMutations = mutatedParameterSummaries.get(key) ?? new Map();
      const nextMutations = new Map(priorMutations);
      for (const [parameterIndex, sources] of taint.mutatedParameterSources) {
        const priorSources = priorMutations.get(parameterIndex) ?? new Set<number>();
        const mergedSources = new Set([...priorSources, ...sources]);
        if (mergedSources.size !== priorSources.size) changed = true;
        nextMutations.set(parameterIndex, mergedSources);
      }
      if (nextMutations.size > 0) mutatedParameterSummaries.set(key, nextMutations);

      for (const call of resolvedEscapeCalls(record.body, record, definitionCounts)) {
        if (definitionCounts.get(call.key) !== 1) continue;
        const callee = recordsByCallableKey.get(call.key)?.[0];
        if (!callee) continue;
        const calleeParameters = escapeFunctionParameters(callee.signature);
        const priorInputs = parameterInputSummaries.get(call.key) ?? new Map();
        const nextInputs = new Map(priorInputs);
        for (const [parameterIndex, argument] of call.arguments.entries()) {
          const parameter = calleeParameters[parameterIndex];
          if (!parameter || !/\b(?:str|String)\b/u.test(parameter.typeSource)) continue;
          const sources = taintedSourcesForText(
            argument,
            taint.bindingSources,
            record,
            returnSummaries,
            definitionCounts,
            record.occurrences,
          );
          const definiteSources = new Set(
            [...sources].filter(
              (sourceIndex) => taintSourceDrafts[sourceIndex]?.definiteAuthoredOperand === true,
            ),
          );
          if (definiteSources.size === 0) continue;
          const priorSources = priorInputs.get(parameterIndex) ?? new Set<number>();
          const mergedSources = new Set([...priorSources, ...definiteSources]);
          if (mergedSources.size !== priorSources.size) changed = true;
          nextInputs.set(parameterIndex, mergedSources);
        }
        if (nextInputs.size > 0) parameterInputSummaries.set(call.key, nextInputs);
      }
    }
    taintByFunction = nextTaint;
    if (!changed) break;
    assert.ok(
      pass < records.length + taintSourceDrafts.length + 8,
      "escape taint fixpoint exceeded the finite function and escape-site set",
    );
  }
  const flows: AuthoredEscapeIdentityFlow[] = [];
  const flowSourceIndexes = new Set<number>();
  const unresolvedBindingEdges: CensusSite[] = [];
  for (const record of records) {
    const taint = taintByFunction.get(record);
    if (!taint) continue;
    for (const statement of rustStatementSlices(record.body)) {
      const absoluteOffset = record.bodyStart + statement.start;
      const flowLine = lineNumberAt(record.source, absoluteOffset);
      const comparisonBindingSources = bindingSourcesForIdentityStatement(
        statement.text,
        taint.bindingSources,
        taint.containerSources,
      );
      const sourceIndexes = new Set(
        [
          ...taintedSourcesForText(
            statement.text,
            taint.bindingSources,
            record,
            returnSummaries,
            definitionCounts,
            record.occurrences,
            taint.shadowedBindings,
          ),
          ...containerIdentitySourcesForText(statement.text, taint.containerSources),
        ].filter((index) => {
          const sourceSite = taintSourceDrafts[index];
          return (
            sourceSite === undefined ||
            sourceSite.path !== record.path ||
            sourceSite.functionName !== record.functionName ||
            sourceSite.line <= flowLine
          );
        }),
      );
      const shadowedBindingsInStatement = [...taint.shadowedBindings].filter(
        (binding) =>
          (taint.bindingSources.get(binding)?.size ?? 0) > 0 &&
          new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(statement.text),
      );
      if (shadowedBindingsInStatement.length > 0) {
        const unfilteredIndexes = taintedSourcesForText(
          statement.text,
          taint.bindingSources,
          record,
          returnSummaries,
          definitionCounts,
          record.occurrences,
        );
        const unfilteredSites = [...unfilteredIndexes]
          .map((index) => taintSourceDrafts[index])
          .filter((site): site is AuthoredTaintSourceDraft => site !== undefined);
        const definiteUnfilteredSites = unfilteredSites.filter(
          (site) => site.definiteAuthoredOperand,
        );
        if (definiteUnfilteredSites.length === 0) continue;
        const unresolvedComparison = productionComparisonId(
          statement.text,
          taint.bindingSources,
          definiteUnfilteredSites.filter((site) => statement.text.includes(site.matchedText)),
          resolvedTaintedCallTexts(statement.text, record, definitionCounts, returnSummaries),
        );
        if (unresolvedComparison) {
          unresolvedBindingEdges.push({
            path: record.path,
            line: flowLine,
            function: record.functionName,
            operation: "unresolved-shadowed-binding",
            evidence: `${shadowedBindingsInStatement.join(",")} in ${statement.text.trim().replace(/\s+/gu, " ").slice(0, 160)}`,
            disposition: "named-exempt",
            reason:
              "The text flow engine cannot assign a reused Rust binding spelling to one lexical scope; this identity-shaped use is not scored as a definite flow.",
          });
        }
      }
      if (sourceIndexes.size === 0) continue;
      const sourceSites = [...sourceIndexes]
        .map((index) => taintSourceDrafts[index])
        .filter((site): site is AuthoredTaintSourceDraft => site !== undefined);
      const comparisonId = productionComparisonId(
        statement.text,
        comparisonBindingSources,
        sourceSites.filter((site) => statement.text.includes(site.matchedText)),
        resolvedTaintedCallTexts(statement.text, record, definitionCounts, returnSummaries),
      );
      if (!comparisonId) continue;
      const carrierFieldSources = sourceSites.filter(
        (site): site is CarrierFieldReadSourceDraft => site.sourceKind === "carrier-field-read",
      );
      if (
        carrierFieldSources.length > 0 &&
        carrierFieldSources.length === sourceSites.length &&
        mechanicalCarrierNameDelegation(
          record.body,
          statement.text,
          statement.start,
          new Set(
            carrierFieldSources.flatMap((site) =>
              Array.from(
                statement.text.matchAll(
                  new RegExp(
                    `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*\\.\\s*${escapeRegExp(site.fieldName)}\\b`,
                    "gu",
                  ),
                ),
                (match) => match[1],
              ),
            ),
          ),
        )
      )
        continue;
      const serializedValueEquivalenceSanction = serializedWholeValueEquivalenceSanction(
        comparisonId,
        statement.text,
        sourceSites,
      );
      const sanctioned =
        serializedValueEquivalenceSanction !== undefined ||
        sourceSites.every(
          (site) =>
            !site.operandClasses.includes("authored-bearing") ||
            (!site.definiteAuthoredOperand &&
              (site.path !== record.path || site.functionName !== record.functionName)),
        );
      for (const index of sourceIndexes) flowSourceIndexes.add(index);
      flows.push({
        path: record.path,
        line: flowLine,
        function: record.functionName,
        comparisonId,
        escapeIds: [
          ...new Set(
            sourceSites.flatMap((site) => (site.sourceKind === "escape" ? [site.escapeId] : [])),
          ),
        ].toSorted(),
        evidence: evidenceLine(record.source, absoluteOffset),
        operandDerivation: sourceSites
          .map((site) => `${site.path}:${site.line}:${site.operandDerivation}`)
          .join("; "),
        sanctioned,
        ...(serializedValueEquivalenceSanction
          ? { sanctionReason: serializedValueEquivalenceSanction }
          : {}),
      });
    }
  }

  const escapeSites: AuthoredEscapeSite[] = escapeOccurrenceDrafts.map((site) => {
    const flowsForSite = flows.filter(
      (flow) =>
        flow.escapeIds.includes(site.escapeId) &&
        flow.operandDerivation.includes(site.operandDerivation),
    );
    const reachesIdentity = flowSourceIndexes.has(site.siteIndex);
    const sanctioned =
      reachesIdentity && flowsForSite.length > 0 && flowsForSite.every((flow) => flow.sanctioned);
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
      reachingIdentitySinks: flowsForSite.map(
        (flow) => `${flow.path}:${flow.line}:${flow.function}:${flow.comparisonId}`,
      ),
    };
  });
  const writeIntoSites = escapeOccurrenceDrafts
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
      const sinkClass = classifyWriteIntoSink(site, record, taint, flowSourceIndexes);
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
  const uniqueUnresolvedBindingEdges = uniqueSites(unresolvedBindingEdges);
  const carrierResult = discoverAuthoredIdentityCarrierAudit(sources, structFields);
  return {
    escapeSites: escapeSites.toSorted(authoredEscapeSiteOrder),
    identityFlows: flows.toSorted(authoredEscapeFlowOrder),
    writeIntoSites: writeIntoSites.toSorted(writeIntoSiteOrder),
    unresolvedCallEdges,
    unresolvedBindingEdges: uniqueUnresolvedBindingEdges,
    carrierAudit: carrierResult.audit,
    identityDeriveViolationCount: carrierResult.deriveViolationCount,
    identityDeriveViolations: carrierResult.deriveViolations,
  };
}

function serializedWholeValueEquivalenceSanction(
  comparisonId: AuthoredComparisonId,
  statement: string,
  sourceSites: readonly AuthoredTaintSourceDraft[],
): string | undefined {
  if (comparisonId !== "binary-eq" || sourceSites.length < 2) return undefined;
  if (
    !sourceSites.every(
      (site) => site.sourceKind === "escape" && site.escapeId === "serde-json-to-vec",
    )
  )
    return undefined;
  if (
    !/\bcached_bytes\s*\.\s*is_ok\s*\(\s*\)/u.test(statement) ||
    !/\bfresh_bytes\s*\.\s*is_ok\s*\(\s*\)/u.test(statement) ||
    !/\bcached_bytes\s*==\s*fresh_bytes\b/u.test(statement)
  )
    return undefined;
  return "The cache shadow oracle compares complete successful serialized values byte-for-byte; its authored-bearing operands remain explicit and the equality is sanctioned as whole-value equivalence, not asserted non-property data.";
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

function assertSerializedWholeValueEquivalenceSanctions(
  flows: readonly AuthoredEscapeIdentityFlow[],
): void {
  const sanctioned = flows.filter((flow) => flow.sanctionReason !== undefined);
  assert.equal(
    sanctioned.length,
    3,
    "the production cache-shadow whole-value equivalence sanction must remain exactly three sites",
  );
  for (const flow of sanctioned) {
    assert.equal(flow.comparisonId, "binary-eq", "whole-value sanction comparison");
    assert.match(
      flow.operandDerivation,
      /=>authored-bearing/u,
      "whole-value sanctions must retain authored-bearing operand provenance",
    );
    assert.doesNotMatch(
      flow.operandDerivation,
      /=>non-property/u,
      "whole-value sanctions must not relabel unqualified carriers as non-property",
    );
  }
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
  if (injectInlineEscapeIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        'fn injected_inline_escape_equality(left: &omena_syntax::ident::AuthoredPropertyTextV0, right: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { format!("{left:?}") == format!("{right:?}") }',
        'fn injected_inline_escape_sort(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut values = vec![property]; values.sort_by_key(|value| format!("{value:?}")); true }',
        'fn injected_inline_escape_map(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut values = std::collections::BTreeMap::new(); values.insert(format!("{property:?}"), 1_u8); true }',
        "fn injected_inline_escape_contains(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let values = std::collections::BTreeSet::<String>::new(); values.contains(&serde_json::to_string(property).unwrap()) }",
        'fn injected_inline_escape_matches(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { matches!(serde_json::to_string(property).unwrap().as_str(), "--probe") }',
      ].join("\n"),
      targetPath,
    );
  }
  if (injectPatternEscapeIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        'fn injected_tuple_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let (value, _) = (serde_json::to_string(property).unwrap(), 0_u8); value == "--probe" }',
        'fn injected_struct_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { struct Pair { value: String } let Pair { value } = Pair { value: serde_json::to_string(property).unwrap() }; value == "--probe" }',
        'fn injected_match_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { match Some(serde_json::to_string(property).unwrap()) { Some(value) => value == "--probe", None => false } }',
        'fn injected_let_else_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let Some(value) = Some(serde_json::to_string(property).unwrap()) else { return false; }; value == "--probe" }',
        'fn injected_if_let_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { if let Some(value) = Some(serde_json::to_string(property).unwrap()) { value == "--probe" } else { false } }',
        'fn injected_slice_escape_binding(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let [value] = [serde_json::to_string(property).unwrap()]; value == "--probe" }',
      ].join("\n"),
      targetPath,
    );
  }
  if (injectNestedEscapeIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        'fn injected_for_block_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool {\n  for _ in 0..1 {\n    let value = serde_json::to_string(property).unwrap();\n    if value == "--probe" { return true; }\n  }\n  false\n}',
        'fn injected_if_let_block_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool {\n  if let Some(property) = Some(property) {\n    let value = serde_json::to_string(property).unwrap();\n    value == "--probe"\n  } else { false }\n}',
        'fn injected_match_block_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool {\n  match Some(property) {\n    Some(property) => {\n      let value = serde_json::to_string(property).unwrap();\n      value == "--probe"\n    },\n    None => false,\n  }\n}',
        'fn injected_bare_block_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool {\n  {\n    let value = serde_json::to_string(property).unwrap();\n    value == "--probe"\n  }\n}',
      ].join("\n"),
      targetPath,
    );
  }
  if (injectMultilineDebugEscapeIdentity) {
    replaceTrackedSourceMutation(
      sources,
      "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs",
      "let candidate_property_key = candidate.property.to_property_name().canonical_key();\n        let scenario_witness_id = format!(",
      "let candidate_property_key = &candidate.property;\n        let scenario_witness_id = format!(",
      "multiline debug escape identity mutation target",
    );
  }
  if (injectEscapeAliasAndCallIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        "use serde_json::{to_string as encode_authored};",
        "use serde_json as json_codec;",
        'fn injected_braced_alias_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let value = encode_authored(property).unwrap(); value == "--probe" }',
        'fn injected_module_alias_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let value = json_codec::to_string(property).unwrap(); value == "--probe" }',
        "fn injected_escape_return(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> String { serde_json::to_string(property).unwrap() }",
        'fn injected_qualified_escape_return_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { self::injected_escape_return(property) == "--probe" }',
        "struct InjectedEscapeMethod;",
        "impl InjectedEscapeMethod { fn render(&self, property: &omena_syntax::ident::AuthoredPropertyTextV0) -> String { serde_json::to_string(property).unwrap() } }",
        'fn injected_method_escape_return_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { InjectedEscapeMethod.render(property) == "--probe" }',
        "fn injected_escape_collect_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let _values = [property].into_iter().map(|value| serde_json::to_string(value).unwrap()).collect::<std::collections::BTreeSet<_>>(); true }",
      ].join("\n"),
      targetPath,
    );
  }
  if (injectUnresolvedWriteIntoSink) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_unresolved_write_into_sink(property: &omena_syntax::ident::AuthoredPropertyTextV0, opaque: impl Fn(String)) { let mut output = String::new(); property.write_into(&mut output).unwrap(); opaque(output); }",
      targetPath,
    );
  }
  if (injectCfgNotTestEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      '#[cfg(not(test))]\nfn injected_cfg_not_test_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); escaped == "--probe" }',
      targetPath,
    );
  }
  if (injectContainerMutationEscapeIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        "fn injected_vec_push_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = Vec::new(); values.push(escaped); values.sort(); values.first().is_some() }",
        "fn injected_vec_extend_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = Vec::new(); values.extend([escaped]); values.dedup(); values.first().is_some() }",
        "fn injected_vec_insert_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = Vec::new(); values.insert(0, escaped); values.contains(&String::new()) }",
        "fn injected_vec_append_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut incoming = Vec::new(); incoming.push(escaped); let mut values = Vec::new(); values.append(&mut incoming); values.binary_search(&String::new()).is_ok() }",
        "fn injected_vec_extend_from_slice_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let incoming = [escaped]; let mut values = Vec::new(); values.extend_from_slice(&incoming); values.sort_unstable(); true }",
        "fn injected_vec_resize_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = Vec::new(); values.resize(2, escaped); values.dedup(); true }",
        "fn injected_annotated_vec_fill_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values: Vec<String> = Vec::new(); values.resize(1, String::new()); values.fill(escaped); values.contains(&String::new()) }",
        "fn injected_vecdeque_push_back_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = std::collections::VecDeque::new(); values.push_back(escaped); values.contains(&String::new()) }",
        "fn injected_vecdeque_insert_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = std::collections::VecDeque::new(); values.insert(0, escaped); values.contains(&String::new()) }",
        "fn injected_string_push_str_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut out = String::new(); out.push_str(&escaped); out == String::new() }",
        "fn injected_string_add_assign_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut out = String::new(); out += &escaped; out == String::new() }",
        "fn injected_string_reborrow_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut out = String::new(); let handle = &mut out; handle.push_str(&escaped); out == String::new() }",
        "fn injected_string_insert_str_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut out = String::new(); out.insert_str(0, &escaped); out == String::new() }",
        'fn injected_string_write_macro_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut out = String::new(); write!(&mut out, "{}", escaped).unwrap(); out == String::new() }',
        "fn injected_btree_map_insert_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = std::collections::BTreeMap::new(); values.insert(escaped, 0_u8); values.contains_key(&String::new()) }",
        "fn injected_btree_set_replace_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); let mut values = std::collections::BTreeSet::new(); values.replace(escaped); values.contains(&String::new()) }",
      ].join("\n"),
      targetPath,
    );
  }
  if (injectUnsupportedReceiverMutation) {
    appendTrackedSourceMutation(
      sources,
      [
        "struct InjectedOpaqueAccumulator;",
        "impl InjectedOpaqueAccumulator { fn push(&mut self, _value: String) {} }",
        "fn injected_opaque_receiver_mutation(property: &omena_syntax::ident::AuthoredPropertyTextV0) { let escaped = serde_json::to_string(property).unwrap(); let mut sink = InjectedOpaqueAccumulator; sink.push(escaped); }",
      ].join("\n"),
      targetPath,
    );
  }
  if (injectOutParameterEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      [
        "fn injected_out_parameter_escape(property: &omena_syntax::ident::AuthoredPropertyTextV0, out: &mut String) { let escaped = serde_json::to_string(property).unwrap(); out.push_str(&escaped); }",
        'fn injected_out_parameter_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut out = String::new(); injected_out_parameter_escape(property, &mut out); out == "--probe" }',
        "fn injected_out_parameter_forward(property: &omena_syntax::ident::AuthoredPropertyTextV0, out: &mut String) { injected_out_parameter_escape(property, out); }",
        'fn injected_out_parameter_composed_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut out = String::new(); let handle = &mut out; injected_out_parameter_forward(property, handle); out == "--probe" }',
        'fn injected_out_parameter_write_macro(property: &omena_syntax::ident::AuthoredPropertyTextV0, out: &mut String) { let escaped = serde_json::to_string(property).unwrap(); write!(out, "{}", escaped).unwrap(); }',
        "fn injected_out_parameter_write_forward(property: &omena_syntax::ident::AuthoredPropertyTextV0, out: &mut String) { let handle = &mut *out; injected_out_parameter_write_macro(property, handle); }",
        'fn injected_out_parameter_write_composed_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut out = String::new(); injected_out_parameter_write_forward(property, &mut out); out == "--probe" }',
      ].join("\n"),
      targetPath,
    );
  }
  if (injectCarrierFieldFlowIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        "#[derive(serde::Serialize, serde::Deserialize)]",
        "struct InjectedEscapeCarrier { name: String }",
        "fn injected_escape_carrier(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> InjectedEscapeCarrier { let mut name = String::new(); property.write_into(&mut name).unwrap(); InjectedEscapeCarrier { name } }",
        "fn injected_escape_carrier_key(value: &InjectedEscapeCarrier) -> omena_syntax::ident::CanonicalCustomPropertyNameV0 { omena_syntax::ident::PropertyNameV0::canonical_custom_key(value.name.clone()) }",
        "fn injected_escape_carrier_copy(value: &InjectedEscapeCarrier) -> String { value.name.clone() }",
        'fn injected_carrier_helper_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let carrier = injected_escape_carrier(property); injected_escape_carrier_copy(&carrier) == "--probe" }',
        "impl InjectedEscapeCarrier { fn authored_name(&self) -> String { self.name.clone() } }",
        'fn injected_carrier_accessor_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let carrier = injected_escape_carrier(property); carrier.authored_name() == "--probe" }',
        'fn injected_carrier_destructure_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let carrier = injected_escape_carrier(property); let InjectedEscapeCarrier { name } = carrier; name == "--probe" }',
        'fn injected_carrier_serde_roundtrip_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let carrier = injected_escape_carrier(property); let bytes = serde_json::to_vec(&carrier).unwrap(); let decoded: InjectedEscapeCarrier = serde_json::from_slice(&bytes).unwrap(); let InjectedEscapeCarrier { name } = decoded; name == "--probe" }',
        "fn injected_carrier_decoy_comparator(left: &InjectedEscapeCarrier, right: &InjectedEscapeCarrier) -> bool { let _left_key = omena_syntax::ident::PropertyNameV0::canonical_custom_key(left.name.clone()); left.name == right.name }",
      ].join("\n"),
      targetPath,
    );
  }
  if (injectUnqualifiedCarrierEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      [
        "#[derive(serde::Serialize)]",
        "struct InjectedUnqualifiedEscapeCarrier { name: String }",
        "fn injected_unqualified_escape_carrier(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> InjectedUnqualifiedEscapeCarrier { let mut name = String::new(); property.write_into(&mut name).unwrap(); InjectedUnqualifiedEscapeCarrier { name } }",
        'fn injected_unqualified_carrier_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let carrier = injected_unqualified_escape_carrier(property); let escaped = serde_json::to_string(&carrier).unwrap(); escaped == "--probe" }',
      ].join("\n"),
      targetPath,
    );
  }
  if (injectArgumentParameterEscapeIdentity) {
    appendTrackedSourceMutation(
      sources,
      [
        'fn injected_argument_parameter_compare(value: &str) -> bool { value == "--probe" }',
        "fn injected_argument_parameter_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let escaped = serde_json::to_string(property).unwrap(); injected_argument_parameter_compare(&escaped) }",
      ].join("\n"),
      targetPath,
    );
  }
  if (injectWriteMacroDebugEscapeIdentities) {
    appendTrackedSourceMutation(
      sources,
      [
        'fn injected_write_debug_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut out = String::new(); write!(&mut out, "{property:?}").unwrap(); out == "--probe" }',
        'fn injected_writeln_debug_escape_identity(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { let mut out = String::new(); writeln!(&mut out, "{property:#?}").unwrap(); out == "--probe" }',
      ].join("\n"),
      targetPath,
    );
  }
}

function applyCarrierFieldConsumerProbeMutation(sources: MutableRustSource[]): void {
  if (!injectCarrierFieldIdentityConsumer) return;
  replaceTrackedSourceMutation(
    sources,
    "rust/crates/omena-lsp-server/src/style_symbol_provider.rs",
    "style_occurrences.sort();\n    style_occurrences.dedup();",
    "style_occurrences.sort_by(|left, right| left.name.cmp(&right.name));\n    style_occurrences.dedup_by(|left, right| left.name == right.name);",
    "escape-populated carrier field consumer mutation target",
  );
}

function applyCarrierDelegationGuardProbeMutation(sources: MutableRustSource[]): void {
  if (!injectCarrierDelegationGuardLaundering) return;
  replaceTrackedSourceMutation(
    sources,
    "rust/crates/omena-query/src/types.rs",
    "    if left_is_custom_property {",
    "    if left.name.len() == right.name.len() {",
    "carrier delegation family-guard laundering target",
  );
}

function assertMechanicalCarrierComparatorDelegations(sources: readonly MutableRustSource[]): void {
  const comparators = [
    {
      path: "rust/crates/omena-lsp-server/src/state.rs",
      functionName: "style_symbol_occurrence_name_cmp",
    },
    {
      path: "rust/crates/omena-query/src/types.rs",
      functionName: "workspace_occurrence_name_cmp",
    },
  ] as const;
  for (const comparator of comparators) {
    const source = sources.find((candidate) => candidate.relativePath === comparator.path)?.source;
    assert.ok(source, `carrier comparator is outside escape-flow scope: ${comparator.path}`);
    const structural = maskCommentsStringsAndTestItems(source, false);
    const functionSlice = rustFunctionSlices(structural).find(
      (candidate) => candidate.name === comparator.functionName,
    );
    assert.ok(
      functionSlice,
      `carrier comparator is absent: ${comparator.path}:${comparator.functionName}`,
    );
    const body = source.slice(functionSlice.bodyStart, functionSlice.bodyEnd);
    const rawComparison = rustStatementSlices(body).find((statement) =>
      /\bleft\s*\.\s*name\s*\.\s*cmp\s*\(\s*&\s*right\s*\.\s*name\s*\)/u.test(statement.text),
    );
    assert.ok(
      rawComparison,
      `carrier comparator raw fallback is absent: ${comparator.path}:${comparator.functionName}`,
    );
    assert.ok(
      mechanicalCarrierNameDelegation(
        body,
        rawComparison.text,
        rawComparison.start,
        new Set(["left", "right"]),
      ),
      `carrier comparator delegation guard does not prove custom-property family selection: ${comparator.path}:${comparator.functionName}`,
    );
  }
}

function assertInjectedEscapeProbeOutcomes(audit: AuthoredEscapeClosureAuditResult): void {
  const requireViolatingFunctions = (enabled: boolean, names: readonly string[], label: string) => {
    if (!enabled) return;
    const violatingFunctions = new Set(
      audit.identityFlows.filter((flow) => !flow.sanctioned).map((flow) => flow.function),
    );
    assert.deepEqual(
      names.filter((name) => !violatingFunctions.has(name)),
      [],
      `${label} counterexamples must reach a production escape-flow identity sink`,
    );
  };
  requireViolatingFunctions(
    injectInlineEscapeIdentities,
    [
      "injected_inline_escape_equality",
      "injected_inline_escape_sort",
      "injected_inline_escape_map",
      "injected_inline_escape_contains",
      "injected_inline_escape_matches",
    ],
    "inline escape",
  );
  requireViolatingFunctions(
    injectPatternEscapeIdentities,
    [
      "injected_tuple_escape_binding",
      "injected_struct_escape_binding",
      "injected_match_escape_binding",
      "injected_let_else_escape_binding",
      "injected_if_let_escape_binding",
      "injected_slice_escape_binding",
    ],
    "pattern escape",
  );
  requireViolatingFunctions(
    injectNestedEscapeIdentities,
    [
      "injected_for_block_escape_identity",
      "injected_if_let_block_escape_identity",
      "injected_match_block_escape_identity",
      "injected_bare_block_escape_identity",
    ],
    "nested escape",
  );
  requireViolatingFunctions(
    injectMultilineDebugEscapeIdentity,
    ["guarded_candidate_seeds_for_pair"],
    "multiline debug escape",
  );
  requireViolatingFunctions(
    injectEscapeAliasAndCallIdentities,
    [
      "injected_braced_alias_escape_identity",
      "injected_module_alias_escape_identity",
      "injected_qualified_escape_return_identity",
      "injected_method_escape_return_identity",
      "injected_escape_collect_identity",
    ],
    "alias and call escape",
  );
  if (injectUnresolvedWriteIntoSink) {
    assert.ok(
      audit.writeIntoSites.some(
        (site) =>
          site.function === "injected_unresolved_write_into_sink" &&
          site.sinkClass === "unresolved",
      ),
      "unrecognized write_into consumers must fail closed as unresolved",
    );
  }
  requireViolatingFunctions(
    injectCfgNotTestEscapeIdentity,
    ["injected_cfg_not_test_escape_identity"],
    "cfg(not(test)) production escape",
  );
  requireViolatingFunctions(
    injectContainerMutationEscapeIdentities,
    [
      "injected_vec_push_escape_identity",
      "injected_vec_extend_escape_identity",
      "injected_vec_insert_escape_identity",
      "injected_vec_append_escape_identity",
      "injected_vec_extend_from_slice_escape_identity",
      "injected_vec_resize_escape_identity",
      "injected_annotated_vec_fill_escape_identity",
      "injected_vecdeque_push_back_escape_identity",
      "injected_vecdeque_insert_escape_identity",
      "injected_string_push_str_escape_identity",
      "injected_string_add_assign_escape_identity",
      "injected_string_reborrow_escape_identity",
      "injected_string_insert_str_escape_identity",
      "injected_string_write_macro_escape_identity",
      "injected_btree_map_insert_escape_identity",
      "injected_btree_set_replace_escape_identity",
    ],
    "container mutation escape",
  );
  requireViolatingFunctions(
    injectOutParameterEscapeIdentity,
    [
      "injected_out_parameter_escape_identity",
      "injected_out_parameter_composed_identity",
      "injected_out_parameter_write_composed_identity",
    ],
    "out-parameter escape",
  );
  requireViolatingFunctions(
    injectCarrierFieldFlowIdentities,
    [
      "injected_carrier_helper_identity",
      "injected_carrier_accessor_identity",
      "injected_carrier_destructure_identity",
      "injected_carrier_serde_roundtrip_identity",
      "injected_carrier_decoy_comparator",
    ],
    "carrier field flow",
  );
  requireViolatingFunctions(
    injectUnqualifiedCarrierEscapeIdentity,
    ["injected_unqualified_carrier_escape_identity"],
    "unqualified carrier escape",
  );
  requireViolatingFunctions(
    injectArgumentParameterEscapeIdentity,
    ["injected_argument_parameter_compare"],
    "argument-to-parameter escape",
  );
  requireViolatingFunctions(
    injectWriteMacroDebugEscapeIdentities,
    ["injected_write_debug_escape_identity", "injected_writeln_debug_escape_identity"],
    "write macro Debug escape",
  );
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
  for (const group of source.matchAll(
    /\buse\s+(serde_json|serde_yaml_ng|toml|serde_wasm_bindgen)\s*::\s*\{([^}]+)\}\s*;/gu,
  )) {
    for (const item of group[2].split(",")) {
      const parsed = item
        .trim()
        .match(/^([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (!parsed) continue;
      const qualifiedPath = `${group[1]}::${parsed[1]}`;
      if (escapeIdForQualifiedPath(qualifiedPath)) {
        aliases.set(parsed[2] ?? parsed[1], "aliased-escape-path");
      }
    }
  }
  for (const moduleAlias of source.matchAll(
    /\buse\s+(serde_json|serde_yaml_ng|toml|serde_wasm_bindgen)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
  )) {
    for (const qualifiedPath of escapeQualifiedPathsForModule(moduleAlias[1])) {
      const member = qualifiedPath.slice(moduleAlias[1].length + 2);
      aliases.set(`${moduleAlias[2]}::${member}`, "aliased-escape-path");
    }
  }
  return aliases;
}

function escapeQualifiedPathsForModule(moduleName: string): readonly string[] {
  return [
    "serde_json::to_string",
    "serde_json::to_string_pretty",
    "serde_json::to_vec",
    "serde_json::to_vec_pretty",
    "serde_json::to_writer",
    "serde_json::to_writer_pretty",
    "serde_json::to_value",
    "serde_yaml_ng::to_string",
    "serde_yaml_ng::to_writer",
    "serde_yaml_ng::to_value",
    "toml::to_string",
    "toml::to_string_pretty",
    "serde_wasm_bindgen::to_value",
  ].filter((qualifiedPath) => qualifiedPath.startsWith(`${moduleName}::`));
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
  for (const closure of functionSlice.scannable.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*sort_by(?:_cached)?_key\s*\(\s*\|\s*([A-Za-z_][A-Za-z0-9_]*)\s*\|/gu,
  )) {
    const receiverType = bindings.get(closure[1]);
    if (receiverType) bindings.set(closure[2], receiverType);
  }
  return bindings;
}

function uniqueRustFunctionReturnTypes(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
  structFields?: ReadonlyMap<string, ReadonlyMap<string, string>>,
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
      const identifiers = returnType.match(/[A-Za-z_][A-Za-z0-9_]*/gu) ?? [];
      const declaredType = structFields
        ? identifiers
            .map((identifier) => aliases.get(identifier) ?? identifier)
            .find((identifier) => structFields.has(identifier))
        : undefined;
      types.add(declaredType ?? normalizeRustType(returnType, aliases));
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
  const debugScannable = maskCommentsStringsAndTestItems(body, false);
  for (const grammar of debugFormatMacroGrammars) {
    const debugMacroPattern = new RegExp(`\\b${escapeRegExp(grammar.macroName)}\\s*!\\s*\\(`, "gu");
    for (const match of body.matchAll(debugMacroPattern)) {
      const openParen = match.index + match[0].lastIndexOf("(");
      const closeParen = matchingDelimiter(debugScannable, openParen, "(", ")");
      if (closeParen === undefined) continue;
      const evidence = body.slice(openParen + 1, closeParen);
      const segments = topLevelSegments(maskCommentsStringsAndTestItems(evidence, false), ",");
      const formatSegment = segments[grammar.formatArgumentIndex];
      if (!formatSegment) continue;
      const formatAndOperands = evidence.slice(formatSegment.offset);
      if (!hasDebugFormatSpecifier(formatAndOperands)) continue;
      const operands = debugFormatOperands(evidence, grammar.formatArgumentIndex);
      const sinkSegment = grammar.formatArgumentIndex === 1 ? segments[0] : undefined;
      const sinkBinding = sinkSegment
        ? evidence
            .slice(sinkSegment.offset, sinkSegment.offset + sinkSegment.text.length)
            .trim()
            .match(/^&\s*mut\s+([A-Za-z_][A-Za-z0-9_]*)\b/u)?.[1]
        : undefined;
      candidates.push({
        offset: match.index,
        escapeId: grammar.escapeId,
        matchedText: body.slice(match.index, closeParen + 1),
        operands,
        sinkBinding,
      });
      debugMacroPattern.lastIndex = closeParen + 1;
    }
  }
  for (const [alias] of useAliases) {
    const pattern = new RegExp(
      `\\b${escapeRegExp(alias).replaceAll("\\ ", "\\s*")}\\s*\\(\\s*&?\\s*([^,)]+)`,
      "gu",
    );
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
  escapePopulatedCarrierTypes: ReadonlySet<string>,
  firstSiteIndex: number,
): EscapeOccurrenceDraft[] {
  const candidates = escapeCandidatesInBody(body, useAliases, functionSlice);
  return candidates.map((candidate, localIndex) => {
    const statement = rustStatementAt(body, candidate.offset);
    const operandContext = body.slice(
      Math.max(0, candidate.offset - 800),
      candidate.offset + candidate.matchedText.length,
    );
    const resultBindings = resultBindingsForEscape(
      statement.text,
      candidate.sinkBinding,
      candidate.matchedText,
    );
    const operandClasses = candidate.operands.map((operand) =>
      selectorGuardedAuthoredOperand(`${operandContext}\n${statement.text}`, operand)
        ? "non-property"
        : classifyAuthoredEscapeOperand(
            operand,
            bindings,
            structFields,
            escapePopulatedCarrierTypes,
          ),
    );
    const definiteAuthoredOperand = candidate.operands.some(
      (operand) =>
        !selectorGuardedAuthoredOperand(`${operandContext}\n${statement.text}`, operand) &&
        isDefinitelyAuthoredEscapeOperand(
          operand,
          bindings,
          structFields,
          escapePopulatedCarrierTypes,
        ),
    );
    const absoluteOffset = functionSlice.bodyStart + candidate.offset;
    return {
      siteIndex: firstSiteIndex + localIndex,
      bodyOffset: candidate.offset,
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
      definiteAuthoredOperand,
      resultBindings,
      sourceKind: "escape",
      sinkBinding: candidate.sinkBinding,
    };
  });
}

function carrierFieldReadSourcesInFunction(
  relativePath: string,
  source: string,
  functionSlice: RustFunctionSlice,
  body: string,
  bindings: ReadonlyMap<string, string>,
  carrierTypes: ReadonlySet<string>,
  firstSiteIndex: number,
): CarrierFieldReadSourceDraft[] {
  const sources: Omit<CarrierFieldReadSourceDraft, "siteIndex">[] = [];
  const structural = maskCommentsStringsAndTestItems(body, false);
  const add = (
    offset: number,
    matchedText: string,
    resultBindings: readonly string[],
    carrierType: string,
  ): void => {
    const absoluteOffset = functionSlice.bodyStart + offset;
    sources.push({
      bodyOffset: offset,
      path: relativePath,
      line: lineNumberAt(source, absoluteOffset),
      functionName: functionSlice.name,
      evidence: evidenceLine(source, absoluteOffset),
      matchedText,
      operandClasses: ["authored-bearing"],
      operandDerivation: `${matchedText}=>escape-populated-carrier:${carrierType}.name`,
      definiteAuthoredOperand: true,
      resultBindings,
      sourceKind: "carrier-field-read",
      carrierType,
      fieldName: "name",
    });
  };

  for (const access of structural.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*name\b/gu)) {
    const carrierType = bindings.get(access[1]);
    if (!carrierType || !carrierTypes.has(carrierType)) continue;
    const after = structural.slice(access.index + access[0].length);
    if (/^\s*=/u.test(after) && !/^\s*==/u.test(after)) continue;
    const statement = rustStatementAt(body, access.index);
    add(
      access.index,
      body.slice(access.index, access.index + access[0].length),
      resultBindingsForEscape(statement.text, undefined, access[0]),
      carrierType,
    );
  }

  for (const destructuring of structural.matchAll(
    /\b(?:let\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*\{([^{}]*)\}\s*=\s*([A-Za-z_][A-Za-z0-9_]*)/gu,
  )) {
    const declaredType = destructuring[1];
    const sourceType = bindings.get(destructuring[3]);
    const carrierType = carrierTypes.has(declaredType)
      ? declaredType
      : sourceType && carrierTypes.has(sourceType)
        ? sourceType
        : undefined;
    if (!carrierType) continue;
    const nameField = destructuring[2].match(
      /(?:^|,)\s*name(?:\s*:\s*([A-Za-z_][A-Za-z0-9_]*))?\s*(?=,|$)/u,
    );
    if (!nameField) continue;
    add(destructuring.index, destructuring[0], [nameField[1] ?? "name"], carrierType);
  }

  const unique = new Map<string, Omit<CarrierFieldReadSourceDraft, "siteIndex">>();
  for (const entry of sources) {
    unique.set(
      `${entry.bodyOffset}\0${entry.matchedText}\0${entry.resultBindings.join(",")}`,
      entry,
    );
  }
  return [...unique.values()].map((entry, index) => ({
    ...entry,
    siteIndex: firstSiteIndex + index,
  }));
}

function selectorGuardedAuthoredOperand(statement: string, operand: string): boolean {
  const access = operand
    .trim()
    .replace(/^&\s*/u, "")
    .match(/^([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*name\b/u);
  if (!access) return false;
  const root = escapeRegExp(access[1]);
  if (new RegExp(`\\b${root}\\s*\\.\\s*kind\\s*==\\s*"selector"`, "u").test(statement)) {
    return true;
  }
  if (
    new RegExp(
      `\\bif\\s+[^{}]*\\b${root}\\s*\\.\\s*kind\\s*!=\\s*"selector"\\s*\\{[\\s\\S]*?\\breturn\\b`,
      "u",
    ).test(statement)
  ) {
    return true;
  }
  if (
    new RegExp(
      `\\bsass_symbol_kind_from_candidate_kind\\s*\\(\\s*${root}\\s*\\.\\s*kind\\s*\\)\\s*\\?`,
      "u",
    ).test(statement)
  ) {
    return true;
  }
  const customPropertyBranch = new RegExp(
    `\\bmatches\\s*!\\s*\\(\\s*${root}\\s*\\.\\s*kind\\s*,[\\s\\S]*?"customProperty(?:Declaration|Reference)"[\\s\\S]*?\\)`,
    "u",
  );
  return (
    customPropertyBranch.test(statement) &&
    new RegExp(`\\b${root}\\s*\\.\\s*property_key\\b`, "u").test(statement) &&
    /\belse\s*\{/u.test(statement)
  );
}

function debugFormatOperands(formatArguments: string, formatArgumentIndex = 0): readonly string[] {
  const operands = new Set<string>();
  for (const interpolation of formatArguments.matchAll(
    /\{\s*((?:[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*|\d+))?\s*:[^}]*\?[^}]*\}/gu,
  )) {
    if (interpolation[1] && !/^\d+$/u.test(interpolation[1])) operands.add(interpolation[1]);
  }
  const structuralArguments = maskCommentsStringsAndTestItems(formatArguments, false);
  const positionalArguments = topLevelSegments(structuralArguments, ",")
    .slice(formatArgumentIndex + 1)
    .map((segment) => formatArguments.slice(segment.offset, segment.offset + segment.text.length));
  for (const argument of positionalArguments) {
    const candidate = argument
      .trim()
      .replace(/^&/u, "")
      .match(/^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*/u)?.[0];
    if (candidate) operands.add(candidate);
  }
  if (operands.size === 0) operands.add("<unresolved-debug-operand>");
  return [...operands];
}

function hasDebugFormatSpecifier(formatArguments: string): boolean {
  return /\{\s*(?:(?:[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*|\d+))?\s*:[^}]*\?[^}]*\}/u.test(
    formatArguments,
  );
}

function classifyAuthoredEscapeOperand(
  expression: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  escapePopulatedCarrierTypes: ReadonlySet<string> = new Set(),
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
      escapePopulatedCarrierTypes,
    );
  }
  const access = normalized.match(/^([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)/u)?.[1];
  const rustType = access ? resolveRustAccessType(access, bindings, structFields) : undefined;
  return classifyAuthoredEscapeType(rustType, structFields, new Set(), escapePopulatedCarrierTypes);
}

function isDefinitelyAuthoredEscapeOperand(
  expression: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  escapePopulatedCarrierTypes: ReadonlySet<string> = new Set(),
): boolean {
  const normalized = expression.trim().replace(/^&\s*/u, "");
  if (/\.(?:authored_text|to_property_name)\s*\(/u.test(normalized)) return true;
  const structLiteralType = normalized.match(
    /^((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Z][A-Za-z0-9_]*)\s*\{/u,
  )?.[1];
  if (structLiteralType) {
    return isDefinitelyAuthoredEscapeType(
      structLiteralType.split(/\s*::\s*/u).at(-1),
      structFields,
      new Set(),
      escapePopulatedCarrierTypes,
    );
  }
  const access = normalized.match(/^([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)/u)?.[1];
  const rustType = access ? resolveRustAccessType(access, bindings, structFields) : undefined;
  return isDefinitelyAuthoredEscapeType(
    rustType,
    structFields,
    new Set(),
    escapePopulatedCarrierTypes,
  );
}

function isDefinitelyAuthoredEscapeType(
  rustType: string | undefined,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  seen: ReadonlySet<string>,
  escapePopulatedCarrierTypes: ReadonlySet<string> = new Set(),
): boolean {
  if (!rustType) return false;
  if (/\b(?:AuthoredPropertyTextV0|PropertyNameV0)\b/u.test(rustType)) return true;
  const typeName = rustType.match(/[A-Z][A-Za-z0-9_]*$/u)?.[0] ?? rustType;
  if (escapePopulatedCarrierTypes.has(typeName)) return true;
  if (seen.has(typeName)) return false;
  const fields = structFields.get(typeName);
  if (!fields) return false;
  const nextSeen = new Set(seen).add(typeName);
  return [...fields.values()].some((fieldType) =>
    isDefinitelyAuthoredEscapeType(fieldType, structFields, nextSeen, escapePopulatedCarrierTypes),
  );
}

function classifyAuthoredEscapeType(
  rustType: string | undefined,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  seen: ReadonlySet<string>,
  escapePopulatedCarrierTypes: ReadonlySet<string> = new Set(),
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
  if (escapePopulatedCarrierTypes.has(typeName)) return "authored-bearing";
  if (externalLeafTypes.some((entry) => entry.typeName === typeName)) return "non-property";
  if (seen.has(typeName)) return "non-property";
  const fields = structFields.get(typeName);
  if (!fields) return "authored-bearing";
  const nextSeen = new Set(seen).add(typeName);
  const classes = [...fields.values()].map((fieldType) =>
    classifyAuthoredEscapeType(fieldType, structFields, nextSeen, escapePopulatedCarrierTypes),
  );
  if (classes.includes("authored-bearing")) return "authored-bearing";
  if (classes.includes("key-bearing")) return "key-bearing";
  return "non-property";
}

function resultBindingsForEscape(
  statement: string,
  sinkBinding: string | undefined,
  matchedText: string,
): readonly string[] {
  const bindings = new Set<string>();
  const occurrenceOffset = statement.indexOf(matchedText);
  for (const assigned of statement.matchAll(
    /\blet\s+([^=;{}]+?)\s*=|\b(?:if|while)\s+let\s+([^=;{}]+?)\s*=/gu,
  )) {
    if (occurrenceOffset < assigned.index + assigned[0].length) continue;
    for (const binding of rustPatternBindings(assigned[1] ?? assigned[2])) bindings.add(binding);
  }
  const matchExpression = statement.match(/\bmatch\s+[\s\S]*?\{/u);
  if (
    matchExpression &&
    occurrenceOffset >= 0 &&
    occurrenceOffset < (matchExpression.index ?? 0) + matchExpression[0].length
  ) {
    for (const arm of statement
      .slice((matchExpression.index ?? 0) + matchExpression[0].length)
      .matchAll(/(?:^|[,{}])\s*([^,{}]+?)\s*(?:if\s+[^=]+)?=>/gu)) {
      for (const binding of rustPatternBindings(arm[1])) bindings.add(binding);
    }
  }
  if (sinkBinding) bindings.add(sinkBinding);
  return [...bindings];
}

interface RustTaintBindingAssignment {
  readonly bindings: readonly string[];
  readonly expression: string;
}

interface RustContainerTaintMutation {
  readonly receiver: string;
  readonly expression: string;
  readonly operation: string;
  readonly bodyOffset: number;
  readonly receiverClass?: StdTaintReceiverClass;
  readonly supported: boolean;
}

type StdTaintReceiverClass = "Vec" | "VecDeque" | "String" | "BTreeMap" | "BTreeSet";

function stdReceiverMutationTable(): Record<
  StdTaintReceiverClass,
  Readonly<Record<string, readonly number[]>>
> {
  return {
    Vec: {
      push: [0],
      insert: [1],
      extend: [0],
      extend_from_slice: [0],
      append: [0],
      resize: [1],
      resize_with: [1],
      splice: [1],
      fill: [0],
      clone_from_slice: [0],
      copy_from_slice: [0],
      clone_from: [0],
    },
    VecDeque: {
      push_front: [0],
      push_back: [0],
      insert: [1],
      append: [0],
      extend: [0],
      resize: [1],
      resize_with: [1],
      clone_from: [0],
    },
    String: {
      push: [0],
      push_str: [0],
      insert: [1],
      insert_str: [1],
      replace_range: [1],
      extend: [0],
      clone_from: [0],
      write_str: [0],
      write_char: [0],
      write_fmt: [0],
    },
    BTreeMap: {
      insert: [0],
      append: [0],
      extend: [0],
      clone_from: [0],
    },
    BTreeSet: {
      insert: [0],
      replace: [0],
      append: [0],
      extend: [0],
      clone_from: [0],
    },
  };
}

function isStdReceiverMutationMethod(method: string): boolean {
  return Object.values(stdReceiverMutationTable()).some((methods) => method in methods);
}

function rustMutableBindingAliases(body: string): ReadonlyMap<string, string> {
  const aliases = new Map<string, string>();
  const structural = maskCommentsStringsAndTestItems(body, false);
  for (let pass = 0; pass <= structural.length + 1; pass += 1) {
    let changed = false;
    for (const binding of structural.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:\s*&\s*mut\s+[^=;]+)?\s*=\s*&\s*mut\s+(?:\*\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      const root = aliases.get(binding[2]) ?? binding[2];
      if (aliases.get(binding[1]) === root) continue;
      aliases.set(binding[1], root);
      changed = true;
    }
    for (const binding of structural.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      const root = aliases.get(binding[2]);
      if (!root || aliases.get(binding[1]) === root) continue;
      aliases.set(binding[1], root);
      changed = true;
    }
    if (!changed) break;
    assert.ok(pass < structural.length, "mutable binding alias resolution did not converge");
  }
  return aliases;
}

function mutableBindingRoot(binding: string, aliases: ReadonlyMap<string, string>): string {
  let current = binding;
  const seen = new Set<string>();
  while (aliases.has(current) && !seen.has(current)) {
    seen.add(current);
    current = aliases.get(current) ?? current;
  }
  return current;
}

function stdReceiverClasses(
  record: EscapeFunctionRecord,
  aliases: ReadonlyMap<string, string>,
): ReadonlyMap<string, StdTaintReceiverClass> {
  const classes = new Map<string, StdTaintReceiverClass>();
  const structural = maskCommentsStringsAndTestItems(`${record.signature}\n${record.body}`, false);
  const classPattern = "(VecDeque|Vec|String|BTreeMap|BTreeSet)";
  for (const typed of structural.matchAll(
    new RegExp(
      `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*:\\s*&?\\s*(?:mut\\s+)?(?:std\\s*::\\s*(?:collections\\s*::\\s*)?)?${classPattern}\\b`,
      "gu",
    ),
  )) {
    classes.set(typed[1], typed[2] as StdTaintReceiverClass);
  }
  for (const constructed of structural.matchAll(
    new RegExp(
      `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)\\s*=\\s*(?:std\\s*::\\s*(?:collections\\s*::\\s*)?)?${classPattern}\\s*::\\s*(?:new|default|with_capacity|from|from_iter)\\s*\\(`,
      "gu",
    ),
  )) {
    classes.set(constructed[1], constructed[2] as StdTaintReceiverClass);
  }
  for (const vector of structural.matchAll(
    /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*vec\s*!\s*[[(]/gu,
  )) {
    classes.set(vector[1], "Vec");
  }
  for (const [binding, rustType] of record.bindings) {
    if (rustType === "String" && !classes.has(binding)) classes.set(binding, "String");
  }
  for (const [alias, target] of aliases) {
    const targetClass = classes.get(mutableBindingRoot(target, aliases));
    if (targetClass) classes.set(alias, targetClass);
  }
  return classes;
}

function rustContainerTaintMutations(
  record: EscapeFunctionRecord,
  aliases: ReadonlyMap<string, string>,
): readonly RustContainerTaintMutation[] {
  const mutations: RustContainerTaintMutation[] = [];
  const body = record.body;
  const scannable = maskCommentsStringsAndTestItems(body, false);
  const receiverClasses = stdReceiverClasses(record, aliases);
  for (const call of scannable.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    if (!isStdReceiverMutationMethod(call[2])) continue;
    const openParenthesis = call.index + call[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(scannable, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const receiver = mutableBindingRoot(call[1], aliases);
    const receiverClass = receiverClasses.get(call[1]) ?? receiverClasses.get(receiver);
    const methodTable = receiverClass ? stdReceiverMutationTable()[receiverClass] : undefined;
    const argumentIndexes = methodTable
      ? (methodTable as Readonly<Record<string, readonly number[]>>)[call[2]]
      : undefined;
    const rawArguments = body.slice(openParenthesis + 1, closeParenthesis);
    const structuralArguments = scannable.slice(openParenthesis + 1, closeParenthesis);
    const argumentsByIndex = topLevelSegments(structuralArguments, ",").map((segment) =>
      rawArguments.slice(segment.offset, segment.offset + segment.text.length),
    );
    mutations.push({
      receiver,
      operation: call[2],
      bodyOffset: call.index,
      expression: (argumentIndexes ?? argumentsByIndex.map((_, index) => index))
        .map((index) => argumentsByIndex[index] ?? "")
        .join(", "),
      receiverClass,
      supported: argumentIndexes !== undefined,
    });
  }
  for (const macro of scannable.matchAll(/\b(write|writeln)\s*!\s*\(/gu)) {
    const openParenthesis = macro.index + macro[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(scannable, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const rawArguments = body.slice(openParenthesis + 1, closeParenthesis);
    const structuralArguments = scannable.slice(openParenthesis + 1, closeParenthesis);
    const segments = topLevelSegments(structuralArguments, ",");
    const sink = segments[0]
      ? rawArguments.slice(segments[0].offset, segments[0].offset + segments[0].text.length)
      : "";
    const sinkBinding =
      sink.match(/&\s*mut\s+(?:\*\s*)?([A-Za-z_][A-Za-z0-9_]*)/u)?.[1] ??
      sink.trim().match(/^([A-Za-z_][A-Za-z0-9_]*)$/u)?.[1];
    if (!sinkBinding) continue;
    const receiver = mutableBindingRoot(sinkBinding, aliases);
    const receiverClass = receiverClasses.get(sinkBinding) ?? receiverClasses.get(receiver);
    mutations.push({
      receiver,
      operation: `${macro[1]}!`,
      bodyOffset: macro.index,
      expression: segments
        .slice(1)
        .map((segment) => rawArguments.slice(segment.offset, segment.offset + segment.text.length))
        .join(", "),
      receiverClass,
      supported: receiverClass === "String",
    });
  }
  for (const assignment of scannable.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\+=\s*([^;]+);/gu)) {
    const receiver = mutableBindingRoot(assignment[1], aliases);
    const receiverClass = receiverClasses.get(assignment[1]) ?? receiverClasses.get(receiver);
    mutations.push({
      receiver,
      operation: "+=",
      bodyOffset: assignment.index,
      expression: body.slice(
        assignment.index + assignment[0].indexOf(assignment[2]),
        assignment.index + assignment[0].lastIndexOf(";"),
      ),
      receiverClass,
      supported: receiverClass === "String",
    });
  }
  return mutations;
}

function rustTaintBindingAssignments(body: string): readonly RustTaintBindingAssignment[] {
  const assignments: RustTaintBindingAssignment[] = [];
  const add = (pattern: string, expression: string): void => {
    const bindings = rustPatternBindings(pattern);
    if (bindings.length > 0) assignments.push({ bindings, expression });
  };
  for (const binding of body.matchAll(
    /\blet\s+((?:[A-Z][A-Za-z0-9_]*\s*)?\{[^}]+\})\s*=\s*([^;]+);/gu,
  )) {
    add(binding[1], binding[2]);
  }
  for (const binding of body.matchAll(/\blet\s+([^=;{}]+?)\s*=\s*([^;]+);/gu)) {
    add(binding[1], binding[2]);
  }
  for (const binding of body.matchAll(/\b(?:if|while)\s+let\s+([^=]+?)\s*=\s*([^{}]+)\{/gu)) {
    add(binding[1], binding[2]);
  }
  for (const binding of body.matchAll(/\bfor\s+([^{}]+?)\s+in\s+([^{}]+)\{/gu)) {
    add(binding[1], binding[2]);
  }
  const scannable = maskCommentsStringsAndTestItems(body, false);
  for (const matched of scannable.matchAll(/\bmatch\s+([^{}]+)\{/gu)) {
    const openBrace = matched.index + matched[0].lastIndexOf("{");
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    const matchBody = body.slice(openBrace + 1, closeBrace);
    for (const arm of matchBody.matchAll(/(?:^|[,{}])\s*([^,{}]+?)\s*(?:if\s+[^=]+)?=>/gu)) {
      add(arm[1], matched[1]);
    }
  }
  return assignments;
}

function computeFunctionTaint(
  record: EscapeFunctionRecord,
  returnSummaries: ReadonlyMap<string, ReadonlySet<number>>,
  definitionCounts: ReadonlyMap<string, number>,
  parameterInputs: ReadonlyMap<number, ReadonlySet<number>> | undefined,
  mutatedParameterSummaries: ReadonlyMap<string, ReadonlyMap<number, ReadonlySet<number>>>,
): FunctionTaintResult {
  const bindings = new Map<string, Set<number>>();
  const containerSources = new Map<string, Set<number>>();
  const unresolvedEdges: CensusSite[] = [];
  const assignments = rustTaintBindingAssignments(record.body);
  const mutableAliases = rustMutableBindingAliases(record.body);
  const containerMutations = rustContainerTaintMutations(record, mutableAliases);
  const composedParameterSources = new Map<number, Set<number>>();
  const assignmentCounts = new Map<string, number>();
  for (const assignment of assignments) {
    for (const binding of assignment.bindings) {
      assignmentCounts.set(binding, (assignmentCounts.get(binding) ?? 0) + 1);
    }
  }
  const shadowedBindings = new Set(
    [...assignmentCounts].filter(([, count]) => count > 1).map(([binding]) => binding),
  );
  const parameters = escapeFunctionParameters(record.signature);
  for (const [parameterIndex, sources] of parameterInputs ?? []) {
    const parameter = parameters[parameterIndex];
    if (!parameter) continue;
    bindings.set(parameter.name, new Set(sources));
  }
  for (const occurrence of record.occurrences) {
    for (const binding of occurrence.resultBindings) {
      const sources = bindings.get(binding) ?? new Set<number>();
      sources.add(occurrence.siteIndex);
      bindings.set(binding, sources);
    }
  }
  for (let pass = 0; pass <= bindings.size + assignments.length + 8; pass += 1) {
    let changed = false;
    for (const assignment of assignments) {
      const sources = sealedAuthorityExpression(assignment.expression)
        ? new Set<number>()
        : taintedSourcesForText(
            assignment.expression,
            bindings,
            record,
            returnSummaries,
            definitionCounts,
            record.occurrences,
          );
      for (const binding of assignment.bindings) {
        const prior = bindings.get(binding) ?? new Set<number>();
        const merged = new Set([...prior, ...sources]);
        if (merged.size !== prior.size) {
          bindings.set(binding, merged);
          changed = true;
        }
      }
      for (const call of unresolvedTaintedCalls(
        assignment.expression,
        bindings,
        record,
        definitionCounts,
      )) {
        unresolvedEdges.push(call);
      }
    }
    for (const mutation of containerMutations) {
      const identityExpression = identityBearingContainerMutationText(mutation);
      const candidateSources = new Set([
        ...taintedSourcesForText(
          identityExpression,
          bindings,
          record,
          returnSummaries,
          definitionCounts,
          record.occurrences,
        ),
        ...containerSourcesForText(identityExpression, containerSources),
      ]);
      const sources = new Set(
        [...candidateSources].filter((sourceIndex) => {
          const localOccurrence = record.occurrences.find(
            (occurrence) => occurrence.siteIndex === sourceIndex,
          );
          return !localOccurrence || localOccurrence.definiteAuthoredOperand;
        }),
      );
      if (!mutation.supported && sources.size > 0) {
        unresolvedEdges.push({
          path: record.path,
          line: lineNumberAt(record.source, record.bodyStart + mutation.bodyOffset),
          function: record.functionName,
          operation: "unresolved-tainted-receiver-mutation",
          evidence: `${mutation.receiver}.${mutation.operation}(${mutation.expression})`
            .replace(/\s+/gu, " ")
            .slice(0, 180),
          disposition: "named-exempt",
          reason:
            "A tainted value reached a mutation outside the closed Vec, VecDeque, String, BTreeMap, and BTreeSet receiver table.",
        });
        continue;
      }
      const prior = containerSources.get(mutation.receiver) ?? new Set<number>();
      const merged = new Set([...prior, ...sources]);
      if (merged.size !== prior.size) {
        containerSources.set(mutation.receiver, merged);
        changed = true;
      }
      if (mutation.receiverClass === "String") {
        const priorBinding = bindings.get(mutation.receiver) ?? new Set<number>();
        const mergedBinding = new Set([...priorBinding, ...sources]);
        if (mergedBinding.size !== priorBinding.size) {
          bindings.set(mutation.receiver, mergedBinding);
          changed = true;
        }
      }
    }
    for (const call of resolvedEscapeCalls(record.body, record, definitionCounts)) {
      const calleeMutations = mutatedParameterSummaries.get(call.key);
      if (!calleeMutations) continue;
      for (const [parameterIndex, sources] of calleeMutations) {
        const actualBinding = mutableArgumentBinding(
          call.arguments[parameterIndex],
          mutableAliases,
        );
        if (!actualBinding) continue;
        const prior = bindings.get(actualBinding) ?? new Set<number>();
        const merged = new Set([...prior, ...sources]);
        if (merged.size !== prior.size) {
          bindings.set(actualBinding, merged);
          changed = true;
        }
        const callerParameterIndex = parameters.findIndex(
          (parameter) =>
            parameter.mutableReference &&
            parameter.name === mutableBindingRoot(actualBinding, mutableAliases),
        );
        if (callerParameterIndex >= 0) {
          const priorParameterSources =
            composedParameterSources.get(callerParameterIndex) ?? new Set<number>();
          const mergedParameterSources = new Set([...priorParameterSources, ...sources]);
          if (mergedParameterSources.size !== priorParameterSources.size) changed = true;
          composedParameterSources.set(callerParameterIndex, mergedParameterSources);
        }
      }
    }
    if (!changed) break;
    assert.ok(
      pass < bindings.size + assignments.length + 8,
      `local taint did not converge in ${record.functionName}`,
    );
  }
  const returnSources = new Set<number>();
  for (const returned of record.body.matchAll(/\breturn\s+([^;]+);/gu)) {
    for (const source of taintedSourcesForText(
      returned[1],
      bindings,
      record,
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
    record,
    returnSummaries,
    definitionCounts,
    record.occurrences,
  ))
    returnSources.add(source);
  const mutatedParameterSources = new Map<number, ReadonlySet<number>>();
  for (const [parameterIndex, parameter] of parameters.entries()) {
    if (!parameter.mutableReference) continue;
    const sources = new Set<number>(composedParameterSources.get(parameterIndex) ?? []);
    for (const expression of outParameterWriteExpressions(
      record.body,
      parameter.name,
      mutableAliases,
      containerMutations,
    )) {
      for (const source of taintedSourcesForText(
        expression,
        bindings,
        record,
        returnSummaries,
        definitionCounts,
        record.occurrences,
      )) {
        const localOccurrence = record.occurrences.find(
          (occurrence) => occurrence.siteIndex === source,
        );
        if (localOccurrence && !localOccurrence.definiteAuthoredOperand) continue;
        sources.add(source);
      }
    }
    if (sources.size > 0) mutatedParameterSources.set(parameterIndex, sources);
  }
  return {
    bindingSources: bindings,
    containerSources,
    returnSources,
    mutatedParameterSources,
    unresolvedEdges,
    shadowedBindings,
  };
}

function identityBearingContainerMutationText(mutation: RustContainerTaintMutation): string {
  if (mutation.receiverClass === "String") return mutation.expression;
  const literals = rustStructLiterals(mutation.expression);
  if (literals.length === 0) return mutation.expression;
  const identityValues: string[] = [];
  for (const literal of literals) {
    const structuralFields = maskCommentsStringsAndTestItems(literal.fields, false);
    for (const segment of topLevelSegments(structuralFields, ",")) {
      const raw = literal.fields.slice(segment.offset, segment.offset + segment.text.length);
      const field = raw.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([\s\S]+)$/u);
      if (!field || !/(?:name|property|key|identity|selector|class)/u.test(field[1])) continue;
      identityValues.push(field[2]);
    }
  }
  return identityValues.join(", ");
}

function escapeFunctionParameters(signature: string): readonly EscapeFunctionParameter[] {
  const openParenthesis = signature.indexOf("(");
  if (openParenthesis < 0) return [];
  const structural = maskCommentsStringsAndTestItems(signature, false);
  const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
  if (closeParenthesis === undefined) return [];
  const parameterText = signature.slice(openParenthesis + 1, closeParenthesis);
  return topLevelSegments(maskCommentsStringsAndTestItems(parameterText, false), ",")
    .map((segment) =>
      parameterText.slice(segment.offset, segment.offset + segment.text.length).trim(),
    )
    .filter(Boolean)
    .flatMap((parameter): EscapeFunctionParameter[] => {
      if (/^(?:&\s*)?(?:mut\s+)?self\b/u.test(parameter)) {
        return [
          {
            name: "self",
            mutableReference: /^&\s*mut\s+self\b/u.test(parameter),
            typeSource: "Self",
          },
        ];
      }
      const parsed = parameter.match(/^(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([\s\S]+)$/u);
      if (!parsed) return [];
      return [
        {
          name: parsed[1],
          mutableReference: /^\s*&\s*mut\b/u.test(parsed[2]),
          typeSource: parsed[2].trim(),
        },
      ];
    });
}

function mutableArgumentBinding(
  argument: string | undefined,
  aliases: ReadonlyMap<string, string>,
): string | undefined {
  if (!argument) return undefined;
  const normalized = argument
    .trim()
    .replace(/^\((.*)\)$/su, "$1")
    .trim();
  const borrowed = normalized.match(/^&\s*mut\s+(?:\*\s*)?([A-Za-z_][A-Za-z0-9_]*)\b/u)?.[1];
  if (borrowed) return mutableBindingRoot(borrowed, aliases);
  const binding = normalized.match(/^([A-Za-z_][A-Za-z0-9_]*)$/u)?.[1];
  if (binding) return mutableBindingRoot(binding, aliases);
  return undefined;
}

function outParameterWriteExpressions(
  body: string,
  parameter: string,
  aliases: ReadonlyMap<string, string>,
  mutations: readonly RustContainerTaintMutation[],
): readonly string[] {
  const expressions: string[] = [];
  for (const mutation of mutations) {
    if (mutation.supported && mutableBindingRoot(mutation.receiver, aliases) === parameter) {
      expressions.push(mutation.expression);
    }
  }
  const parameterAliases = new Set<string>([parameter]);
  for (const [alias, root] of aliases) {
    if (mutableBindingRoot(root, aliases) === parameter) parameterAliases.add(alias);
  }
  for (const binding of parameterAliases) {
    const assignmentPattern = new RegExp(`\\*\\s*${escapeRegExp(binding)}\\s*=\\s*([^;]+)`, "gu");
    for (const assignment of body.matchAll(assignmentPattern)) expressions.push(assignment[1]);
  }
  return expressions;
}

function containerSourcesForText(
  text: string,
  containerSources: ReadonlyMap<string, ReadonlySet<number>>,
): ReadonlySet<number> {
  const sources = new Set<number>();
  for (const [binding, bindingSources] of containerSources) {
    if (!new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text)) continue;
    for (const source of bindingSources) sources.add(source);
  }
  return sources;
}

function containerIdentitySourcesForText(
  text: string,
  containerSources: ReadonlyMap<string, ReadonlySet<number>>,
): ReadonlySet<number> {
  const sources = new Set<number>();
  for (const [binding, bindingSources] of containerSources) {
    const directElementIdentity = new RegExp(
      `\\b${escapeRegExp(binding)}\\s*\\.\\s*(?:sort|sort_unstable|dedup|sort_by|sort_by_key|sort_by_cached_key|sort_unstable_by|sort_unstable_by_key|dedup_by|dedup_by_key|binary_search|binary_search_by|binary_search_by_key)\\s*\\(`,
      "u",
    );
    const directSearch = new RegExp(
      `\\b${escapeRegExp(binding)}\\s*\\.\\s*(?:contains|contains_key|get|entry|remove)\\s*\\(`,
      "u",
    );
    if (!directElementIdentity.test(text) && !directSearch.test(text)) continue;
    for (const source of bindingSources) sources.add(source);
  }
  return sources;
}

function bindingSourcesForIdentityStatement(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
  containerSources: ReadonlyMap<string, ReadonlySet<number>>,
): ReadonlyMap<string, ReadonlySet<number>> {
  const combined = new Map(bindingSources);
  const identitySources = containerIdentitySourcesForText(text, containerSources);
  if (identitySources.size === 0) return combined;
  for (const [binding, sources] of containerSources) {
    if (!new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text)) continue;
    combined.set(binding, new Set([...(combined.get(binding) ?? []), ...sources]));
  }
  return combined;
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
  record: EscapeFunctionRecord,
  returnSummaries: ReadonlyMap<string, ReadonlySet<number>>,
  definitionCounts: ReadonlyMap<string, number>,
  localOccurrences: readonly AuthoredTaintSourceDraft[],
  excludedBindings: ReadonlySet<string> = new Set(),
): ReadonlySet<number> {
  const sources = new Set<number>();
  for (const occurrence of localOccurrences) {
    if (
      text.includes(occurrence.matchedText) &&
      !escapeInsideNonValueClosure(text, occurrence.matchedText)
    )
      sources.add(occurrence.siteIndex);
  }
  for (const [binding, bindingTaint] of bindingSources) {
    if (excludedBindings.has(binding)) continue;
    if (
      new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text) &&
      !bindingAppearsOnlyInNonValuePredicate(text, binding)
    ) {
      for (const source of bindingTaint) sources.add(source);
    }
  }
  for (const key of resolvedEscapeCallKeys(text, record, definitionCounts)) {
    if (definitionCounts.get(key) !== 1) continue;
    for (const source of returnSummaries.get(key) ?? []) sources.add(source);
  }
  return sources;
}

function escapeInsideNonValueClosure(text: string, matchedText: string): boolean {
  const occurrenceOffset = text.indexOf(matchedText);
  if (occurrenceOffset < 0) return false;
  const scannable = maskCommentsStringsAndTestItems(text, false);
  const precedingArm = scannable.lastIndexOf("=>", occurrenceOffset);
  if (
    precedingArm >= 0 &&
    /\b(?:return|break|continue)\b/u.test(scannable.slice(precedingArm + 2, occurrenceOffset))
  )
    return true;
  for (const closure of scannable.matchAll(/\.(?:ok_or_else|map_err)\s*\(/gu)) {
    const openParenthesis = closure.index + closure[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(scannable, openParenthesis, "(", ")");
    if (
      closeParenthesis !== undefined &&
      occurrenceOffset > openParenthesis &&
      occurrenceOffset < closeParenthesis
    )
      return true;
  }
  return false;
}

function bindingAppearsOnlyInNonValuePredicate(text: string, binding: string): boolean {
  const structural = maskCommentsStringsAndTestItems(text, false);
  const bindingOffsets = Array.from(
    structural.matchAll(new RegExp(`\\b${escapeRegExp(binding)}\\b`, "gu")),
    (match) => match.index,
  );
  if (bindingOffsets.length === 0) return false;
  const predicateRanges: { readonly start: number; readonly end: number }[] = [];
  for (const predicate of structural.matchAll(
    /\.(?:filter|find|any|all|position|rposition|take_while|skip_while|is_some_and)\s*\(/gu,
  )) {
    const openParenthesis = predicate.index + predicate[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const closure = structural.slice(openParenthesis + 1, closeParenthesis);
    if (!/\|[^|]*\|/u.test(closure)) continue;
    predicateRanges.push({ start: openParenthesis + 1, end: closeParenthesis });
  }
  return bindingOffsets.every((offset) =>
    predicateRanges.some((range) => range.start <= offset && offset < range.end),
  );
}

function escapeCallableKey(crateName: string, name: string, implType?: string): string {
  return `${crateName}\0${implType ? `method:${implType}` : "free"}\0${name}`;
}

function resolvedEscapeCallKeys(
  text: string,
  record: EscapeFunctionRecord,
  definitionCounts: ReadonlyMap<string, number>,
): readonly string[] {
  return [...new Set(resolvedEscapeCalls(text, record, definitionCounts).map((call) => call.key))];
}

function resolvedEscapeCalls(
  text: string,
  record: EscapeFunctionRecord,
  definitionCounts: ReadonlyMap<string, number>,
): readonly ResolvedEscapeCall[] {
  const calls = new Map<string, ResolvedEscapeCall>();
  const structural = maskCommentsStringsAndTestItems(text, false);
  const reserved = new Set(["if", "while", "for", "match", "Some", "None", "Ok", "Err"]);
  for (const call of structural.matchAll(
    /(?<!\.)\b((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*)([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    if (reserved.has(call[2])) continue;
    if (/\.\s*$/u.test(structural.slice(0, call.index))) continue;
    const openParenthesis = call.index + call[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const pathSegments = call[1]
      .split("::")
      .map((segment) => segment.trim())
      .filter(Boolean);
    const associatedType = pathSegments.findLast((segment) => /^[A-Z]/u.test(segment));
    const associatedKey = associatedType
      ? escapeCallableKey(record.crateName, call[2], associatedType)
      : undefined;
    const key =
      associatedKey && definitionCounts.has(associatedKey)
        ? associatedKey
        : escapeCallableKey(record.crateName, call[2]);
    calls.set(`${call.index}\0${key}`, {
      key,
      arguments: rustCallArguments(text, structural, openParenthesis, closeParenthesis),
    });
  }
  for (const method of structural.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    const root = method[1].split(".")[0];
    const receiverType = /^[A-Z]/u.test(root) ? root : record.bindings.get(root);
    if (!receiverType) continue;
    const openParenthesis = method.index + method[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const key = escapeCallableKey(record.crateName, method[2], receiverType);
    calls.set(`${method.index}\0${key}`, {
      key,
      arguments: [
        text.slice(method.index, method.index + method[1].length),
        ...rustCallArguments(text, structural, openParenthesis, closeParenthesis),
      ],
    });
  }
  return [...calls.values()];
}

function rustCallArguments(
  text: string,
  structural: string,
  openParenthesis: number,
  closeParenthesis: number,
): readonly string[] {
  const rawArguments = text.slice(openParenthesis + 1, closeParenthesis);
  if (rawArguments.trim().length === 0) return [];
  const structuralArguments = structural.slice(openParenthesis + 1, closeParenthesis);
  return topLevelSegments(structuralArguments, ",").map((segment) =>
    rawArguments.slice(segment.offset, segment.offset + segment.text.length),
  );
}

function unresolvedTaintedCalls(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
  record: EscapeFunctionRecord,
  definitionCounts: ReadonlyMap<string, number>,
): readonly CensusSite[] {
  const sites: CensusSite[] = [];
  for (const call of text.matchAll(
    /\b((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*)([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/gu,
  )) {
    const pathSegments = call[1]
      .split("::")
      .map((segment) => segment.trim())
      .filter(Boolean);
    const associatedType = pathSegments.findLast((segment) => /^[A-Z]/u.test(segment));
    const key = escapeCallableKey(record.crateName, call[2], associatedType);
    if (definitionCounts.get(key) === 1) continue;
    if (!definitionCounts.has(key)) continue;
    const receivesTaint = [...bindingSources].some(
      ([binding, sources]) =>
        sources.size > 0 && new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(call[3]),
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
  readonly end: number;
}

function rustStatementSlices(body: string): readonly RustStatementSlice[] {
  const statements: RustStatementSlice[] = [];
  const scannable = maskCommentsStringsAndTestItems(body, false);
  let start = 0;
  let bracketDepth = 0;
  let parenthesisDepth = 0;
  for (let index = 0; index < body.length; index += 1) {
    switch (scannable[index]) {
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
      bracketDepth !== 0 ||
      parenthesisDepth !== 0
    )
      continue;
    const raw = body.slice(start, index + 1);
    const leading = raw.length - raw.trimStart().length;
    const text = raw.trim();
    if (text) statements.push({ text, start: start + leading, end: index + 1 });
    start = index + 1;
  }
  const rawTail = body.slice(start);
  const tail = rawTail.trim();
  if (tail)
    statements.push({
      text: tail,
      start: start + rawTail.length - rawTail.trimStart().length,
      end: body.length,
    });
  return statements;
}

function rustStatementAt(body: string, offset: number): RustStatementSlice {
  const statement = rustStatementSlices(body).find(
    (candidate) => candidate.start <= offset && offset < candidate.end,
  );
  if (statement) return statement;
  const beforeSemicolon = body.lastIndexOf(";", offset - 1);
  const beforeNewline = body.lastIndexOf("\n", offset - 1);
  const start = Math.max(beforeSemicolon, beforeNewline) + 1;
  const afterSemicolon = body.indexOf(";", offset);
  const end = afterSemicolon < 0 ? body.length : afterSemicolon + 1;
  return { text: body.slice(start, end), start, end };
}

function productionComparisonId(
  text: string,
  bindingSources: ReadonlyMap<string, ReadonlySet<number>>,
  inlineEscapes: readonly AuthoredTaintSourceDraft[] = [],
  taintedCalls: readonly string[] = [],
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
    inlineEscapes.length === 0 &&
    taintedCalls.length === 0 &&
    !taintedBindings.some((binding) => new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(text))
  )
    return undefined;
  let normalized = text;
  for (const binding of taintedBindings)
    normalized = normalized.replaceAll(new RegExp(`\\b${escapeRegExp(binding)}\\b`, "gu"), "value");
  for (const escape of inlineEscapes)
    normalized = normalized.replaceAll(escape.matchedText, "value");
  for (const call of taintedCalls) normalized = normalized.replaceAll(call, "value");
  const generated = detectGeneratedComparison(normalized);
  if (
    generated &&
    [
      "map-insert",
      "map-get",
      "map-entry",
      "map-contains-key",
      "map-remove",
      "set-insert",
      "set-get",
      "set-contains",
      "set-remove",
    ].includes(generated) &&
    !identityCallFirstArgumentContainsValue(normalized) &&
    !identityCallReceiverContainsValue(normalized)
  )
    return undefined;
  if (generated) return generated;
  if (
    /\.(?:insert|entry)\s*\(/u.test(normalized) &&
    (identityCallFirstArgumentContainsValue(normalized) ||
      identityCallReceiverContainsValue(normalized))
  )
    return "map-insert";
  if (
    /\.(?:get|contains_key|contains|remove)\s*\(/u.test(normalized) &&
    (identityCallFirstArgumentContainsValue(normalized) ||
      identityCallReceiverContainsValue(normalized))
  )
    return "map-get";
  if (
    /\bmatch\s+value\s*\.\s*as_str\s*\(\s*\)\s*\{[\s\S]*?"(?:[^"\\]|\\.)*"\s*=>/u.test(normalized)
  )
    return "match-literal";
  if (
    /\.collect\s*::\s*<[^;]*(?:BTreeSet|BTreeMap|HashSet|HashMap)[^;]*>\s*\(\s*\)/u.test(
      normalized,
    ) &&
    /\bvalue\b/u.test(normalized)
  )
    return "map-to-string-collect-sort";
  return undefined;
}

function resolvedTaintedCallTexts(
  text: string,
  record: EscapeFunctionRecord,
  definitionCounts: ReadonlyMap<string, number>,
  returnSummaries: ReadonlyMap<string, ReadonlySet<number>>,
): readonly string[] {
  const calls = new Set<string>();
  const consider = (call: string): void => {
    if (
      resolvedEscapeCallKeys(call, record, definitionCounts).some(
        (key) => (returnSummaries.get(key)?.size ?? 0) > 0,
      )
    )
      calls.add(call);
  };
  for (const call of text.matchAll(
    /(?<!\.)\b(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Za-z_][A-Za-z0-9_]*\s*\([^()]*\)/gu,
  )) {
    consider(call[0]);
  }
  for (const call of text.matchAll(
    /\b[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)+\s*\([^()]*\)/gu,
  )) {
    consider(call[0]);
  }
  return [...calls];
}

function identityCallFirstArgumentContainsValue(text: string): boolean {
  const scannable = maskCommentsStringsAndTestItems(text, true);
  for (const call of scannable.matchAll(
    /\.(?:insert|get|entry|contains_key|contains|remove)\s*\(/gu,
  )) {
    const openParenthesis = call.index + call[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(scannable, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const firstArgument = topLevelSegments(
      text.slice(openParenthesis + 1, closeParenthesis),
      ",",
    )[0]?.text;
    if (firstArgument && /\bvalue\b/u.test(firstArgument)) return true;
  }
  return false;
}

function identityCallReceiverContainsValue(text: string): boolean {
  const scannable = maskCommentsStringsAndTestItems(text, true);
  return /\bvalue\s*\.\s*(?:insert|get|entry|contains_key|contains|remove)\s*\(/u.test(scannable);
}

function classifyWriteIntoSink(
  site: EscapeOccurrenceDraft & { readonly sinkBinding: string },
  record: EscapeFunctionRecord,
  taint: FunctionTaintResult,
  identitySources: ReadonlySet<number>,
): WriteIntoSinkClass {
  const binding = site.sinkBinding;
  if (/^(?:f|fmt|formatter)$/u.test(binding)) return "formatter";
  if (identitySources.has(site.siteIndex)) return "unresolved";
  const callableParameters = new Set<string>();
  for (const parameter of record.signature.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:impl\s+)?Fn(?:Once|Mut)?\b/gu,
  )) {
    callableParameters.add(parameter[1]);
  }
  const occurrenceOffset = site.bodyOffset;
  const remainder = record.body.slice(occurrenceOffset + site.matchedText.length);
  for (const callable of callableParameters) {
    const invocation = new RegExp(
      `\\b${escapeRegExp(callable)}\\s*\\([^)]*\\b${escapeRegExp(binding)}\\b`,
      "u",
    );
    if (invocation.test(remainder)) return "unresolved";
  }
  const returnType = escapeFunctionReturnType(record.signature);
  if (returnType && returnType !== "()" && writeSinkReachesFunctionReturn(site, record, taint)) {
    return "returned-to-emitter";
  }
  if (recognizedEmitterUse(site, record.body, remainder)) return "emitter-output";
  return "unresolved";
}

function recognizedEmitterUse(
  site: EscapeOccurrenceDraft & { readonly sinkBinding: string },
  body: string,
  remainder: string,
): boolean {
  const binding = site.sinkBinding;
  const escapedBinding = escapeRegExp(binding);
  const patterns = [
    new RegExp(`\\b[A-Za-z_][A-Za-z0-9_]*\\s*:\\s*${escapedBinding}\\b`, "u"),
    new RegExp(
      `\\b(?:format|format_args|write|writeln|print|println|json)\\s*!\\s*\\([^;]*\\b${escapedBinding}\\b`,
      "u",
    ),
    new RegExp(`\\b(?:Some|Ok|Arc::new|Box::new)\\s*\\(\\s*${escapedBinding}\\b`, "u"),
    new RegExp(
      `\\b${escapedBinding}\\s*\\.\\s*(?:as_str|as_bytes|into_bytes|into_boxed_str)\\s*\\(`,
      "u",
    ),
    new RegExp(
      `\\b${escapedBinding}\\s*\\.\\s*(?:push|push_str|extend|write_str|write_fmt)\\s*\\(`,
      "u",
    ),
    new RegExp(
      `\\b(?:ClassNameV0|PropertyNameV0)\\s*::\\s*(?:new|from_authored|custom|standard)\\s*\\(\\s*&?\\s*${escapedBinding}\\b`,
      "u",
    ),
  ];
  if (patterns.some((pattern) => pattern.test(remainder))) return true;
  if (writeSinkIsNestedValue(site, body)) return true;
  return balancedEmitterCallUses(binding, remainder);
}

function rustFunctionTailExpression(body: string): string {
  const structural = maskCommentsStringsAndTestItems(body, false);
  let parenthesisDepth = 0;
  let bracketDepth = 0;
  let braceDepth = 0;
  let lastTopLevelSemicolon = -1;
  for (let index = 0; index < structural.length; index += 1) {
    switch (structural[index]) {
      case "(":
        parenthesisDepth += 1;
        break;
      case ")":
        parenthesisDepth -= 1;
        break;
      case "[":
        bracketDepth += 1;
        break;
      case "]":
        bracketDepth -= 1;
        break;
      case "{":
        braceDepth += 1;
        break;
      case "}":
        braceDepth -= 1;
        break;
      case ";":
        if (parenthesisDepth === 0 && bracketDepth === 0 && braceDepth === 0) {
          lastTopLevelSemicolon = index;
        }
        break;
    }
  }
  return body
    .slice(lastTopLevelSemicolon + 1)
    .trim()
    .replace(/;+$/u, "");
}

function writeSinkReachesFunctionReturn(
  site: EscapeOccurrenceDraft & { readonly sinkBinding: string },
  record: EscapeFunctionRecord,
  taint: FunctionTaintResult,
): boolean {
  if (taint.returnSources.has(site.siteIndex)) return true;
  const returnedExpressions = [
    ...Array.from(record.body.matchAll(/\breturn\s+([^;]+);/gu), (match) => match[1]),
    rustFunctionTailExpression(record.body),
  ];
  for (const expression of returnedExpressions) {
    if (
      expression.includes(site.matchedText) &&
      !escapeInsideNonValueClosure(expression, site.matchedText)
    )
      return true;
    for (const [binding, sources] of taint.bindingSources) {
      if (!sources.has(site.siteIndex)) continue;
      if (new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(expression)) return true;
    }
    for (const [binding, sources] of taint.containerSources) {
      if (!sources.has(site.siteIndex)) continue;
      if (new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(expression)) return true;
    }
  }
  return false;
}

function writeSinkIsNestedValue(
  site: EscapeOccurrenceDraft & { readonly sinkBinding: string },
  body: string,
): boolean {
  const structural = maskCommentsStringsAndTestItems(body, false);
  const escapedBinding = escapeRegExp(site.sinkBinding);
  const siteEnd = site.bodyOffset + site.matchedText.length;
  const blockStack: number[] = [];
  for (let index = 0; index < structural.length; index += 1) {
    if (structural[index] === "{") {
      blockStack.push(index);
      continue;
    }
    if (structural[index] !== "}") continue;
    const openBrace = blockStack.pop();
    if (openBrace === undefined || openBrace >= site.bodyOffset || index <= siteEnd) continue;
    const block = body.slice(openBrace + 1, index);
    const siteInBlock = site.bodyOffset - openBrace - 1;
    if (siteInBlock < 0 || siteInBlock >= block.length) continue;
    const tail = rustFunctionTailExpression(block).replace(/,\s*$/u, "").trim();
    if (new RegExp(`^${escapedBinding}$`, "u").test(tail)) return true;
  }
  return false;
}

function balancedEmitterCallUses(binding: string, remainder: string): boolean {
  const structural = maskCommentsStringsAndTestItems(remainder, false);
  const escapedBinding = escapeRegExp(binding);
  for (const call of structural.matchAll(
    /\b((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    const openParenthesis = call.index + call[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    const argumentsSource = remainder.slice(openParenthesis + 1, closeParenthesis);
    if (!new RegExp(`\\b${escapedBinding}\\b`, "u").test(argumentsSource)) continue;
    const callee = call[1].replace(/\s+/gu, "");
    if (/^(?:fmt::Debug::fmt|fmt::Display::fmt)$/u.test(callee)) return true;
    if (/&\s*mut\s+[A-Za-z_][A-Za-z0-9_]*/u.test(argumentsSource)) return true;
  }
  for (const method of structural.matchAll(
    /\b[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\s*\.\s*(push|push_str|insert|extend)\s*\(/gu,
  )) {
    const openParenthesis = method.index + method[0].lastIndexOf("(");
    const closeParenthesis = matchingDelimiter(structural, openParenthesis, "(", ")");
    if (closeParenthesis === undefined) continue;
    if (
      new RegExp(`\\b${escapedBinding}\\b`, "u").test(
        remainder.slice(openParenthesis + 1, closeParenthesis),
      )
    )
      return true;
  }
  return false;
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
  for (const typeName of escapePopulatedStringCarrierTypes(sources, structFields)) {
    directAuthoredTypes.add(typeName);
  }
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

function escapePopulatedStringCarrierTypes(
  sources: readonly MutableRustSource[],
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  suppliedAliasesByPath?: ReadonlyMap<string, ReadonlyMap<string, string>>,
  suppliedFunctionReturnTypes?: ReadonlyMap<string, string>,
): ReadonlySet<string> {
  const aliasesByPath = suppliedAliasesByPath ?? rustTypeAliasesByPath(sources);
  const functionReturnTypes =
    suppliedFunctionReturnTypes ?? uniqueRustFunctionReturnTypes(sources, aliasesByPath);
  const carriers = new Set<string>();
  const functions: {
    readonly relativePath: string;
    readonly source: string;
    readonly slice: RustFunctionSlice;
    readonly body: string;
    readonly useAliases: ReadonlyMap<string, AuthoredEscapeId>;
    readonly bindings: ReadonlyMap<string, string>;
  }[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const useAliases = rustEscapeAliases(source);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    for (const slice of rustFunctionSlices(scannable)) {
      functions.push({
        relativePath,
        source,
        slice,
        body: source.slice(slice.bodyStart, slice.bodyEnd),
        useAliases,
        bindings: inferredEscapeBindings(
          slice,
          source,
          aliases,
          structFields,
          relativePath.split("/")[2] ?? "<unknown-crate>",
          functionReturnTypes,
        ),
      });
    }
  }
  const namedStringCarrier = (typeName: string): boolean =>
    structFields.get(typeName)?.get("name") === "String";
  for (const { slice, body, useAliases, bindings } of functions) {
    const sinkBindings = new Set(
      escapeCandidatesInBody(body, useAliases, slice)
        .filter((candidate) =>
          candidate.operands.some((operand) =>
            isDefinitelyAuthoredEscapeOperand(operand, bindings, structFields),
          ),
        )
        .map((candidate) => candidate.sinkBinding)
        .filter((binding): binding is string => binding !== undefined),
    );
    if (sinkBindings.size === 0) continue;
    for (const literal of rustStructLiterals(body)) {
      if (!namedStringCarrier(literal.typeName)) continue;
      if (
        [...sinkBindings].some((binding) =>
          new RegExp(
            `(?:^|,)\\s*name\\s*(?::\\s*${escapeRegExp(binding)}(?:\\s*\\.\\s*clone\\s*\\(\\s*\\))?)?\\s*(?=,|$)`,
            "u",
          ).test(literal.fields),
        )
      )
        carriers.add(literal.typeName);
    }
  }
  for (let pass = 0; pass <= structFields.size; pass += 1) {
    let changed = false;
    for (const { slice, body } of functions) {
      const carrierBindings = new Set<string>();
      for (const typeName of carriers) {
        const pattern = new RegExp(
          `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*:\\s*&?(?:mut\\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\\s*::\\s*)*${escapeRegExp(typeName)}\\b`,
          "gu",
        );
        for (const parameter of slice.signature.matchAll(pattern))
          carrierBindings.add(parameter[1]);
      }
      if (carrierBindings.size === 0) continue;
      for (const literal of rustStructLiterals(body)) {
        if (carriers.has(literal.typeName) || !namedStringCarrier(literal.typeName)) continue;
        const populated = [...carrierBindings].some((binding) =>
          new RegExp(
            `(?:^|,)\\s*name\\s*:\\s*${escapeRegExp(binding)}\\s*\\.\\s*name(?:\\s*\\.\\s*clone\\s*\\(\\s*\\))?\\s*(?=,|$)`,
            "u",
          ).test(literal.fields),
        );
        if (!populated) continue;
        carriers.add(literal.typeName);
        changed = true;
      }
    }
    if (!changed) break;
    assert.ok(pass < structFields.size, "escape-populated carrier closure did not converge");
  }
  return carriers;
}

function manuallyIdentifiedEscapeCarrierTypes(
  sources: readonly MutableRustSource[],
  escapePopulatedCarrierTypes: ReadonlySet<string>,
): ReadonlySet<string> {
  const manual = new Set<string>();
  for (const { source } of sources) {
    const structural = maskCommentsStringsAndTestItems(source, false);
    for (const slice of rustFunctionSlices(structural)) {
      const body = structural.slice(slice.bodyStart, slice.bodyEnd);
      if (!/\bPropertyNameV0\s*::\s*canonical_custom_key\s*\(/u.test(body)) continue;
      const signatureTypes = new Set(
        Array.from(slice.signature.matchAll(/\b([A-Z][A-Za-z0-9_]*)\b/gu), (match) => match[1]),
      );
      const implType = enclosingImplType(structural, slice.bodyStart, new Map());
      if (implType) signatureTypes.add(implType);
      for (const typeName of signatureTypes) {
        if (escapePopulatedCarrierTypes.has(typeName)) manual.add(typeName);
      }
    }
  }
  return manual;
}

function discoverEscapePopulatedCarrierFieldIdentitySites(
  sources: readonly MutableRustSource[],
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
): readonly DiscoveredSite[] {
  const carrierTypes = escapePopulatedStringCarrierTypes(sources, structFields);
  const functionReturnTypes = uniqueRustFunctionReturnTypes(sources, aliasesByPath, structFields);
  const producerNamesByCrate = new Map<string, Set<string>>();
  for (const [key, returnType] of functionReturnTypes) {
    if (!carrierTypes.has(returnType)) continue;
    const [crateName, functionName] = key.split("\0");
    if (!crateName || !functionName) continue;
    const names = producerNamesByCrate.get(crateName) ?? new Set<string>();
    names.add(functionName);
    producerNamesByCrate.set(crateName, names);
  }

  const sites: DiscoveredSite[] = [];
  const rawNameIdentity =
    /\b[A-Za-z_][A-Za-z0-9_]*\s*\.\s*name\s*(?:(?:==|!=)|\.\s*(?:cmp|partial_cmp|eq|ne)\s*\()[\s\S]*?\b[A-Za-z_][A-Za-z0-9_]*\s*\.\s*name\b/u;
  for (const { relativePath, source } of sources) {
    const crateName = relativePath.split("/")[2] ?? "<unknown-crate>";
    const producers = producerNamesByCrate.get(crateName) ?? new Set<string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const body = source.slice(functionSlice.bodyStart, functionSlice.bodyEnd);
      const bodyStructural = maskCommentsStringsAndTestItems(body, false);
      const carrierParameters = new Set<string>();
      for (const carrierType of carrierTypes) {
        const parameterPattern = new RegExp(
          `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*:\\s*[^,)]*\\b${escapeRegExp(carrierType)}\\b`,
          "gu",
        );
        for (const parameter of functionSlice.signature.matchAll(parameterPattern)) {
          carrierParameters.add(parameter[1]);
        }
      }

      const carrierCollections = new Set<string>();
      for (const carrierType of carrierTypes) {
        const annotatedCollection = new RegExp(
          `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)\\s*:\\s*[^=;]*\\b${escapeRegExp(carrierType)}\\b`,
          "gu",
        );
        for (const binding of bodyStructural.matchAll(annotatedCollection)) {
          carrierCollections.add(binding[1]);
        }
        const literalPush = new RegExp(
          `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*\\.\\s*push\\s*\\(\\s*${escapeRegExp(carrierType)}\\s*\\{`,
          "gu",
        );
        for (const push of bodyStructural.matchAll(literalPush)) carrierCollections.add(push[1]);
      }
      for (const producerName of producers) {
        const producerPush = new RegExp(
          `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*\\.\\s*push\\s*\\(\\s*(?:(?:[A-Za-z_][A-Za-z0-9_]*)\\s*::\\s*)*${escapeRegExp(producerName)}\\s*\\(`,
          "gu",
        );
        for (const push of bodyStructural.matchAll(producerPush)) carrierCollections.add(push[1]);
      }

      for (let pass = 0; pass <= carrierCollections.size + 2; pass += 1) {
        let changed = false;
        for (const assignment of bodyStructural.matchAll(
          /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
        )) {
          if (!carrierCollections.has(assignment[2]) || carrierCollections.has(assignment[1]))
            continue;
          carrierCollections.add(assignment[1]);
          changed = true;
        }
        if (!changed) break;
      }

      for (const binding of carrierCollections) {
        const identityCall = new RegExp(
          `\\b${escapeRegExp(binding)}\\s*\\.\\s*(sort_by(?:_cached)?_key|sort_by|dedup_by_key|dedup_by)\\s*\\(`,
          "gu",
        );
        for (const call of bodyStructural.matchAll(identityCall)) {
          const openParenthesis = call.index + call[0].lastIndexOf("(");
          const closeParenthesis = matchingDelimiter(bodyStructural, openParenthesis, "(", ")");
          if (closeParenthesis === undefined) continue;
          const argumentsText = body.slice(openParenthesis + 1, closeParenthesis);
          if (!rawNameIdentity.test(argumentsText)) continue;
          if (/\b(?:canonical_custom_key|canonical_key|property_key)\b/u.test(argumentsText))
            continue;
          sites.push(
            siteAt(
              relativePath,
              source,
              scannable,
              functionSlice.bodyStart + call.index,
              `escape-populated-carrier-field-${call[1]}`,
            ),
          );
        }
      }

      if (carrierParameters.size === 0) continue;
      for (const statement of rustStatementSlices(body)) {
        if (!rawNameIdentity.test(statement.text)) continue;
        if (
          ![...carrierParameters].some((parameter) =>
            new RegExp(`\\b${escapeRegExp(parameter)}\\s*\\.\\s*name\\b`, "u").test(statement.text),
          )
        )
          continue;
        if (
          mechanicalCarrierNameDelegation(body, statement.text, statement.start, carrierParameters)
        )
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + statement.start,
            "escape-populated-carrier-field-direct",
          ),
        );
      }
    }
  }
  return uniqueSites(sites);
}

function mechanicalCarrierNameDelegation(
  body: string,
  rawComparison: string,
  rawComparisonOffset: number,
  carrierBindings: ReadonlySet<string>,
): boolean {
  const rawBindings = new Set(
    Array.from(
      rawComparison.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*name\b/gu),
      (match) => match[1],
    ).filter((binding) => carrierBindings.has(binding)),
  );
  if (rawBindings.size < 2) return false;
  const prefix = body.slice(0, rawComparisonOffset);
  const canonicalKeyOffsets: number[] = [];
  for (const binding of rawBindings) {
    const keyPattern = new RegExp(
      `\\bPropertyNameV0\\s*::\\s*canonical_custom_key\\s*\\(\\s*${escapeRegExp(binding)}\\s*\\.\\s*name\\b`,
      "gu",
    );
    const keyMatches = [...prefix.matchAll(keyPattern)];
    if (keyMatches.length === 0) return false;
    canonicalKeyOffsets.push(keyMatches.at(-1)?.index ?? -1);
  }
  const canonicalStart = Math.min(...canonicalKeyOffsets);
  if (canonicalStart < 0) return false;
  const canonicalTail = prefix.slice(canonicalStart);
  if (!/\.\s*cmp\s*\(/u.test(canonicalTail)) return false;
  const returnOffset = prefix.lastIndexOf("return", canonicalStart);
  if (returnOffset < 0 || canonicalStart - returnOffset > 120) return false;
  const guardOffset = [...prefix.slice(0, returnOffset).matchAll(/\bif\b/gu)].at(-1)?.index ?? -1;
  if (guardOffset < 0) return false;
  const guardOpen = prefix.indexOf("{", guardOffset);
  if (guardOpen < 0 || guardOpen > returnOffset) return false;
  const guardClose = matchingBrace(maskCommentsStringsAndTestItems(prefix, false), guardOpen);
  if (guardClose === undefined || guardClose < canonicalStart) return false;
  const guardCondition = prefix.slice(guardOffset + 2, guardOpen);
  return customPropertyFamilyGuardSelectsBindings(
    prefix.slice(0, guardOffset),
    guardCondition,
    rawBindings,
  );
}

function customPropertyFamilyGuardSelectsBindings(
  beforeGuard: string,
  guardCondition: string,
  rawBindings: ReadonlySet<string>,
): boolean {
  const familyExpression = (binding: string): string =>
    `\\b${escapeRegExp(binding)}\\s*\\.\\s*kind\\s*\\.\\s*family\\s*\\(\\s*\\)\\s*==\\s*OmenaWorkspaceOccurrenceFamilyV0\\s*::\\s*CustomProperty\\b`;
  const bindings = [...rawBindings];
  if (bindings.length === 2) {
    const [leftBinding, rightBinding] = bindings;
    const leftFamily = familyExpression(leftBinding);
    const rightFamily = familyExpression(rightBinding);
    if (
      new RegExp(
        `(?:${leftFamily})\\s*&&\\s*(?:${rightFamily})|(?:${rightFamily})\\s*&&\\s*(?:${leftFamily})`,
        "u",
      ).test(guardCondition)
    ) {
      return true;
    }
  }

  const familyFlags = new Map<string, string>();
  for (const binding of rawBindings) {
    const declarations = [
      ...beforeGuard.matchAll(
        new RegExp(
          `\\blet\\s+([A-Za-z_][A-Za-z0-9_]*)\\s*=\\s*${familyExpression(binding)}\\s*;`,
          "gu",
        ),
      ),
    ];
    const flag = declarations.at(-1)?.[1];
    if (!flag) return false;
    familyFlags.set(binding, flag);
  }
  const flags = [...familyFlags.values()];
  if (flags.length !== 2) return false;
  const [leftFlag, rightFlag] = flags.map(escapeRegExp);
  if (
    new RegExp(
      `(?:\\b${leftFlag}\\b)\\s*&&\\s*(?:\\b${rightFlag}\\b)|(?:\\b${rightFlag}\\b)\\s*&&\\s*(?:\\b${leftFlag}\\b)`,
      "u",
    ).test(guardCondition)
  ) {
    return true;
  }
  const selectedFlags = flags.filter((flag) =>
    new RegExp(`\\b${escapeRegExp(flag)}\\b`, "u").test(guardCondition),
  );
  if (selectedFlags.length !== 1) return false;

  const relationDeclarations = [
    ...beforeGuard.matchAll(
      new RegExp(
        `\\blet\\s+([A-Za-z_][A-Za-z0-9_]*)\\s*=\\s*(?:${escapeRegExp(flags[0])}\\s*\\.\\s*cmp\\s*\\(\\s*&\\s*${escapeRegExp(flags[1])}|${escapeRegExp(flags[1])}\\s*\\.\\s*cmp\\s*\\(\\s*&\\s*${escapeRegExp(flags[0])})\\s*\\)\\s*;`,
        "gu",
      ),
    ),
  ];
  const relation = relationDeclarations.at(-1)?.[1];
  if (!relation) return false;
  return new RegExp(
    `\\bif\\s+${escapeRegExp(relation)}\\s*!=\\s*(?:[A-Za-z_][A-Za-z0-9_]*\\s*::\\s*)?Equal\\s*\\{[\\s\\S]*?\\breturn\\s+${escapeRegExp(relation)}\\s*;[\\s\\S]*?\\}`,
    "u",
  ).test(beforeGuard);
}

function rustStructLiterals(body: string): readonly { typeName: string; fields: string }[] {
  const literals: { typeName: string; fields: string }[] = [];
  const scannable = maskCommentsStringsAndTestItems(body, false);
  for (const candidate of scannable.matchAll(
    /\b(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9_]*)\s*\{/gu,
  )) {
    const openBrace = candidate.index + candidate[0].lastIndexOf("{");
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    literals.push({
      typeName: candidate[1],
      fields: body.slice(openBrace + 1, closeBrace),
    });
  }
  return literals;
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

function assertInventoryRowsOutsideCfgTestItems(
  rows: readonly {
    readonly path: string;
    readonly line: number;
    readonly function: string;
    readonly operation?: string;
    readonly escapeId?: AuthoredEscapeId;
  }[],
  label: string,
): void {
  const maskedByPath = new Map<string, string[]>();
  const originalByPath = new Map<string, string[]>();
  const retainedOnMaskedLines: string[] = [];
  for (const row of rows) {
    if (!existsSync(path.join(repoRoot, row.path))) continue;
    if (!maskedByPath.has(row.path)) {
      const original = readFileSync(path.join(repoRoot, row.path), "utf8");
      originalByPath.set(row.path, original.split(/\r?\n/u));
      maskedByPath.set(row.path, maskRustCfgTestItems(original).split(/\r?\n/u));
    }
    const originalLine = originalByPath.get(row.path)?.[row.line - 1] ?? "";
    const maskedLine = maskedByPath.get(row.path)?.[row.line - 1] ?? "";
    if (originalLine.trim().length > 0 && maskedLine.trim().length === 0) {
      retainedOnMaskedLines.push(
        `${row.path}:${row.line}:${row.function}:${row.operation ?? row.escapeId ?? "site"}`,
      );
    }
  }
  assert.deepEqual(
    retainedOnMaskedLines,
    [],
    `${label} inventory retained rows on #[cfg(test)]-masked lines`,
  );
}

function discoverCountedCannotSeeBoundaries(
  sources: readonly MutableRustSource[],
): readonly CountedCannotSeeBoundary[] {
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  const functionReturnTypes = uniqueRustFunctionReturnTypes(sources, aliasesByPath);
  const escapePopulatedCarrierTypes = escapePopulatedStringCarrierTypes(
    sources,
    structFields,
    aliasesByPath,
    functionReturnTypes,
  );
  const qualifiedCarrierTypes = manuallyIdentifiedEscapeCarrierTypes(
    sources,
    escapePopulatedCarrierTypes,
  );
  const records = boundaryEscapeFunctionRecords(
    sources,
    aliasesByPath,
    structFields,
    functionReturnTypes,
  );
  return [
    countedCannotSeeBoundary(
      "collection-callback-comparison-grammar",
      [
        "binary_search_by",
        "binary_search_by_key",
        "retain",
        "max_by",
        "max_by_key",
        "min_by",
        "min_by_key",
        "chunk_by",
        "iter().any",
      ],
      discoverCollectionCallbackComparisonGrammarSites(sources),
      "The flow engine scans tainted field reads and comparisons inside these callbacks, but it does not type-resolve each collection callback binder as a distinct container-identity edge. The inventory is restricted to the product escape-flow closure.",
    ),
    countedCannotSeeBoundary(
      "carrier-field-read-spelling-grammar",
      ["field-name-not-name", "match-pattern", "closure-binder", "deref-field"],
      discoverCarrierFieldReadSpellingGrammarSites(sources),
      "Carrier-source discovery is bound to a literal String field named `name` and direct `.name` or `Type { name }` reads. Property-shaped alternative fields plus match-arm, closure-binder, and explicit Deref field reads are counted lexical upper bounds, not affirmed non-property flows.",
    ),
    countedCannotSeeBoundary(
      "carrier-type-qualification-gate",
      ["canonical-custom-key-text-qualification"],
      discoverCarrierTypeQualificationGateSites(
        sources,
        escapePopulatedCarrierTypes,
        qualifiedCarrierTypes,
      ),
      "The carrier-field source plane currently admits an escape-populated carrier only when a function associated with that type contains a canonical custom-property key call. Excluded carrier types remain authored-bearing in operand classification and are never affirmed non-property.",
    ),
    countedCannotSeeBoundary(
      "place-expression-mutation-grammar",
      ["complex-method-receiver", "complex-mutable-argument"],
      discoverPlaceExpressionMutationGrammarSites(sources),
      "The mutable-flow resolver follows bare bindings and named aliases. Index, field, call-chain, dereference, and other place-expression receivers or mutable arguments are counted as a conservative product-closure upper bound.",
    ),
    countedCannotSeeBoundary(
      "assignment-form-container-write",
      ["non-string-container-assignment"],
      discoverAssignmentFormContainerWriteSites(records),
      "The container mutation table models method and selected macro/add-assign writes, not reassignment or indexed/place assignment into a known non-String container.",
    ),
    countedCannotSeeBoundary(
      "non-string-container-scalar-egress",
      [
        "first",
        "last",
        "get",
        "get_mut",
        "pop",
        "pop_front",
        "pop_back",
        "remove",
        "swap_remove",
        "drain",
        "iter",
        "iter_mut",
        "into_iter",
        "keys",
        "values",
        "values_mut",
      ],
      discoverNonStringContainerScalarEgressSites(records),
      "The flow engine tracks authored sources into known non-String containers but does not generally transfer a container source back into scalars returned by these egress methods.",
    ),
  ];
}

function countedCannotSeeBoundary(
  id: CountedCannotSeeId,
  operations: readonly string[],
  sites: readonly CensusSite[],
  reason: string,
): CountedCannotSeeBoundary {
  return {
    id,
    direction: "decrease-only",
    operations,
    siteCount: sites.length,
    sites,
    siteDigest: digest(sites),
    reason,
  };
}

function discoverCollectionCallbackComparisonGrammarSites(
  sources: readonly MutableRustSource[],
): CensusSite[] {
  const patterns = [
    ["binary_search_by_key", /\.\s*binary_search_by_key\s*\(/gu],
    ["binary_search_by", /\.\s*binary_search_by\s*\(/gu],
    ["retain", /\.\s*retain\s*\(/gu],
    ["max_by_key", /\.\s*max_by_key\s*\(/gu],
    ["max_by", /\.\s*max_by\s*\(/gu],
    ["min_by_key", /\.\s*min_by_key\s*\(/gu],
    ["min_by", /\.\s*min_by\s*\(/gu],
    ["chunk_by", /\.\s*chunk_by\s*\(/gu],
    ["iter().any", /\.\s*iter\s*\(\s*\)\s*\.\s*any\s*\(/gu],
  ] as const;
  const sites: CensusSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const [operation, pattern] of patterns) {
      for (const match of scannable.matchAll(pattern)) {
        const openParenthesis = match.index + match[0].lastIndexOf("(");
        const closeParenthesis = matchingDelimiter(scannable, openParenthesis, "(", ")");
        if (closeParenthesis === undefined) continue;
        const callback = source.slice(openParenthesis + 1, closeParenthesis);
        if (!/\.\s*(?:name|property|property_name|key|identity)\b/u.test(callback)) continue;
        if (!/(?:==|!=|\.\s*(?:cmp|partial_cmp|eq|ne|contains)\s*\()/u.test(callback)) continue;
        const site = siteAt(
          relativePath,
          source,
          scannable,
          match.index,
          `collection-callback-comparison:${operation}`,
        );
        sites.push({
          ...site,
          evidence: source
            .slice(match.index, closeParenthesis + 1)
            .trim()
            .replace(/\s+/gu, " ")
            .slice(0, 240),
          disposition: "named-exempt",
          reason:
            "Counted upper bound for collection callback syntax whose closure binder is not type-resolved as a distinct container-identity edge.",
        });
      }
    }
  }
  return coalesceCountedCannotSeeSitesByScopeOperation(sites);
}

function discoverCarrierFieldReadSpellingGrammarSites(
  sources: readonly MutableRustSource[],
): CensusSite[] {
  const sites: CensusSite[] = [];
  const propertyField = /(?:property|custom|declaration|reference|identity|selector|class|key)/u;
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const structure of scannable.matchAll(/\bstruct\s+[A-Za-z_][A-Za-z0-9_]*[^;{]*\{/gu)) {
      const openBrace = structure.index + structure[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:std\s*::\s*string\s*::\s*)?String\b/gu,
      )) {
        if (field[1] === "name" || !propertyField.test(field[1])) continue;
        sites.push(
          countedCannotSeeSite(
            relativePath,
            source,
            scannable,
            openBrace + 1 + field.index,
            "carrier-field-spelling:field-name-not-name",
            "A property-shaped String field with a spelling other than `name` is outside the carrier-source recognizer's field-name bound.",
          ),
        );
      }
    }
    for (const matchPattern of scannable.matchAll(
      /\bmatch\b[\s\S]{0,240}?\b[A-Z][A-Za-z0-9_]*\s*\{[^{}]*(?:name|property|property_name)[^{}]*\}\s*=>/gu,
    )) {
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          matchPattern.index,
          "carrier-field-spelling:match-pattern",
          "A field read introduced by a match-arm pattern is not resolved as a carrier-source binding.",
        ),
      );
    }
    for (const closure of scannable.matchAll(/\|([^|\n]+)\|/gu)) {
      const binders = rustPatternBindings(closure[1]);
      if (binders.length === 0) continue;
      const tail = scannable.slice(closure.index + closure[0].length, closure.index + 600);
      if (
        !binders.some((binding) =>
          new RegExp(
            `\\b${escapeRegExp(binding)}\\s*\\.\\s*(?:name|property|property_name)\\b`,
            "u",
          ).test(tail),
        )
      )
        continue;
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          closure.index,
          "carrier-field-spelling:closure-binder",
          "A closure binder's carrier type is not independently resolved before a property-shaped field read.",
        ),
      );
    }
    for (const deref of scannable.matchAll(
      /(?:\(\s*\*\s*[A-Za-z_][A-Za-z0-9_]*\s*\)|(?:std\s*::\s*ops\s*::\s*)?Deref\s*::\s*deref\s*\([^)]*\))\s*\.\s*(?:name|property|property_name)\b/gu,
    )) {
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          deref.index,
          "carrier-field-spelling:deref-field",
          "An explicit Deref place before a property-shaped field is outside the direct `.name` access grammar.",
        ),
      );
    }
  }
  return uniqueSites(sites);
}

function discoverCarrierTypeQualificationGateSites(
  sources: readonly MutableRustSource[],
  escapePopulatedCarrierTypes: ReadonlySet<string>,
  qualifiedCarrierTypes: ReadonlySet<string>,
): CensusSite[] {
  const sites: CensusSite[] = [];
  for (const carrierType of [...escapePopulatedCarrierTypes].toSorted()) {
    if (qualifiedCarrierTypes.has(carrierType)) continue;
    for (const { relativePath, source } of sources) {
      const scannable = maskCommentsStringsAndTestItems(source, false);
      const declaration = new RegExp(`\\bstruct\\s+${escapeRegExp(carrierType)}\\b`, "u").exec(
        scannable,
      );
      if (!declaration) continue;
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          declaration.index,
          `carrier-type-qualification:${carrierType}`,
          "This escape-populated carrier is not a carrier-field taint source because no associated function satisfies the canonical-custom-key text qualification gate.",
        ),
      );
      break;
    }
  }
  return uniqueSites(sites);
}

function discoverPlaceExpressionMutationGrammarSites(
  sources: readonly MutableRustSource[],
): CensusSite[] {
  const sites: CensusSite[] = [];
  const mutationMethods = new Set(
    Object.values(stdReceiverMutationTable()).flatMap((methods) => Object.keys(methods)),
  );
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const tokens = rustSemanticTokens(scannable);
    for (let index = 1; index + 2 < tokens.length; index += 1) {
      if (tokens[index].text !== ".") continue;
      const method = tokens[index + 1];
      if (!mutationMethods.has(method.text) || tokens[index + 2].text !== "(") continue;
      const receiverTail = tokens[index - 1];
      const receiverIsBare =
        /^[A-Za-z_][A-Za-z0-9_]*$/u.test(receiverTail.text) && tokens[index - 2]?.text !== ".";
      if (receiverIsBare) continue;
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          receiverTail.start,
          `place-expression-receiver:${method.text}`,
          "A known mutation method is invoked through a non-bare place expression that the receiver resolver cannot bind.",
        ),
      );
    }
    for (let index = 0; index + 3 < tokens.length; index += 1) {
      if (tokens[index].text !== "&" || tokens[index + 1].text !== "mut") continue;
      const first = tokens[index + 2];
      const next = tokens[index + 3];
      const bare =
        /^[A-Za-z_][A-Za-z0-9_]*$/u.test(first.text) && [",", ")", ";"].includes(next.text);
      if (bare) continue;
      sites.push(
        countedCannotSeeSite(
          relativePath,
          source,
          scannable,
          tokens[index].start,
          "place-expression-argument:&mut",
          "A mutable argument is a field, index, dereference, or call-chain place rather than a bare binding or named alias.",
        ),
      );
    }
  }
  return uniqueSites(sites);
}

function boundaryEscapeFunctionRecords(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
  functionReturnTypes: ReadonlyMap<string, string>,
): readonly EscapeFunctionRecord[] {
  const records: EscapeFunctionRecord[] = [];
  for (const { relativePath, source } of sources) {
    const structural = maskCommentsStringsAndTestItems(source, false);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    for (const slice of rustFunctionSlices(structural)) {
      const body = source.slice(slice.bodyStart, slice.bodyEnd);
      records.push({
        crateName: relativePath.split("/")[2] ?? "<unknown-crate>",
        implType: enclosingImplType(structural, slice.bodyStart, aliases),
        path: relativePath,
        source,
        functionName: slice.name,
        signature: slice.signature,
        body,
        scannable: slice.scannable,
        bodyStart: slice.bodyStart,
        bindings: inferredEscapeBindings(
          slice,
          source,
          aliases,
          structFields,
          relativePath.split("/")[2] ?? "<unknown-crate>",
          functionReturnTypes,
        ),
        occurrences: [],
      });
    }
  }
  return records;
}

function discoverAssignmentFormContainerWriteSites(
  records: readonly EscapeFunctionRecord[],
): CensusSite[] {
  const sites: CensusSite[] = [];
  for (const record of records) {
    const aliases = rustMutableBindingAliases(record.body);
    const classes = stdReceiverClasses(record, aliases);
    const tokens = rustSemanticTokens(maskCommentsStringsAndTestItems(record.body, false));
    for (let index = 0; index < tokens.length; index += 1) {
      if (tokens[index].text !== "=") continue;
      let start = index - 1;
      while (start >= 0 && ![";", "{", "}", "=>"].includes(tokens[start].text)) start -= 1;
      const left = tokens.slice(start + 1, index);
      if (left.some((token) => ["let", "if", "while", "const", "static"].includes(token.text)))
        continue;
      const root = left.find((token) => /^[A-Za-z_][A-Za-z0-9_]*$/u.test(token.text))?.text;
      if (!root) continue;
      const receiver = mutableBindingRoot(root, aliases);
      const receiverClass = classes.get(root) ?? classes.get(receiver);
      if (!receiverClass || receiverClass === "String") continue;
      sites.push(
        countedCannotSeeSite(
          record.path,
          record.source,
          maskCommentsStringsAndTestItems(record.source, false),
          record.bodyStart + tokens[index].start,
          `assignment-form-container-write:${receiverClass}`,
          "A known non-String container is written through assignment syntax, which is outside the method/macro mutation table.",
        ),
      );
    }
  }
  return uniqueSites(sites);
}

function discoverNonStringContainerScalarEgressSites(
  records: readonly EscapeFunctionRecord[],
): CensusSite[] {
  const egressMethods = new Set([
    "first",
    "last",
    "get",
    "get_mut",
    "pop",
    "pop_front",
    "pop_back",
    "remove",
    "swap_remove",
    "drain",
    "iter",
    "iter_mut",
    "into_iter",
    "keys",
    "values",
    "values_mut",
  ]);
  const sites: CensusSite[] = [];
  for (const record of records) {
    const aliases = rustMutableBindingAliases(record.body);
    const classes = stdReceiverClasses(record, aliases);
    const structural = maskCommentsStringsAndTestItems(record.body, false);
    for (const call of structural.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
    )) {
      if (!egressMethods.has(call[2])) continue;
      const receiver = mutableBindingRoot(call[1], aliases);
      const receiverClass = classes.get(call[1]) ?? classes.get(receiver);
      if (!receiverClass || receiverClass === "String") continue;
      sites.push(
        countedCannotSeeSite(
          record.path,
          record.source,
          maskCommentsStringsAndTestItems(record.source, false),
          record.bodyStart + call.index,
          `non-string-container-egress:${receiverClass}.${call[2]}`,
          "A known non-String container method can return an element or iterator, but container taint is not generally transferred to that scalar result.",
        ),
      );
    }
  }
  return uniqueSites(sites);
}

function countedCannotSeeSite(
  relativePath: string,
  source: string,
  scannable: string,
  offset: number,
  operation: string,
  reason: string,
): CensusSite {
  return {
    ...siteAt(relativePath, source, scannable, offset, operation),
    disposition: "named-exempt",
    reason,
  };
}

function assertCountedCannotSeeBoundariesDecreaseOnly(
  current: readonly CountedCannotSeeBoundary[],
  previous: readonly CountedCannotSeeBoundary[],
): void {
  const currentById = new Map(current.map((boundary) => [boundary.id, boundary]));
  for (const boundary of current) {
    assert.equal(boundary.siteCount, boundary.sites.length, `${boundary.id} site count`);
    assert.equal(boundary.siteDigest, digest(boundary.sites), `${boundary.id} site digest`);
  }
  for (const baseline of previous) {
    const boundary = currentById.get(baseline.id);
    assert.ok(boundary, `counted cannotSee boundary was removed: ${baseline.id}`);
    assert.ok(
      boundary.siteCount <= baseline.siteCount,
      `counted cannotSee boundary increased: ${baseline.id} baseline=${baseline.siteCount} current=${boundary.siteCount}`,
    );
    const baselineMultiplicity = countedCannotSeeSiteMultiplicity(baseline.sites);
    const currentMultiplicity = countedCannotSeeSiteMultiplicity(boundary.sites);
    for (const [key, count] of currentMultiplicity) {
      assert.ok(
        count <= (baselineMultiplicity.get(key) ?? 0),
        `counted cannotSee boundary gained a new site: ${baseline.id}:${key}`,
      );
    }
  }
}

function countedCannotSeeSiteMultiplicity(
  sites: readonly CensusSite[],
): ReadonlyMap<string, number> {
  const multiplicity = new Map<string, number>();
  for (const site of sites) {
    const key = `${site.path}\u0000${site.function}\u0000${site.operation}`;
    multiplicity.set(key, (multiplicity.get(key) ?? 0) + 1);
  }
  return multiplicity;
}

function assertZeroBranchEvidenceGatesRegistered(
  surfaces: readonly { readonly surface: string; readonly evidenceGate: string }[],
): void {
  const registrySource = readFileSync(
    path.join(repoRoot, "packages/check-orchestrator/CHECKS.md"),
    "utf8",
  );
  const registeredGateIds = new Set(
    [...registrySource.matchAll(/^\|\s*`([^`]+)`\s*\|/gmu)].map((match) => match[1]),
  );
  if (injectZeroBranchGateRegistryDeletion) {
    registeredGateIds.delete(surfaces[0]?.evidenceGate ?? "");
  }
  assert.deepEqual(
    surfaces.filter((row) => !registeredGateIds.has(row.evidenceGate)),
    [],
    "zero-branch evidence gate is absent from the generated check registry",
  );
}

function authoredEscapeCannotSee(
  unresolvedCallEdgeCount: number,
  unresolvedBindingEdgeCount: number,
): readonly string[] {
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
    `${sourceNeedleLocation(checkerPath, "function discoverAuthoredEscapeClosureAudit(): AuthoredEscapeClosureAuditResult {")}: cross-crate calls, dyn dispatch, unknown method receivers, and indirect function bindings are not resolved by the intra-crate text fixpoint; module-qualified free calls and uniquely typed inherent methods are resolved.`,
    `${sourceNeedleLocation(checkerPath, "const definitionCounts = new Map<string, number>();")}: ${unresolvedCallEdgeCount} known-but-ambiguous intra-crate call edges remain separately enumerated with file:line and do not contribute return taint.`,
    `${sourceNeedleLocation(checkerPath, "const shadowedBindings = new Set(")}: ${unresolvedBindingEdgeCount} identity-shaped uses reached only through reused binding spellings remain separately enumerated with file:line; the text engine declines to infer through them because it cannot prove lexical scope.`,
    `${sourceNeedleLocation(checkerPath, "function discoverResidualRawPropertyCarrierSites(")}: key construction whose input was normalized before the authority constructor is visible only when the residual provenance resolver recognizes that normalization; any unrecognized pre-normalization remains outside the claim.`,
    `${sourceNeedleLocation(checkerPath, "function maskCommentsStringsAndTestItems(")}: proc-macro-generated bodies other than serde derives and unsafe transmute are outside the text scanner.`,
    `${sourceNeedleLocation(checkerPath, 'const generatedFixtureOnly = process.argv.includes("--generated-fixture-only");')}: generated matrix cells are scanner-only and are never compiled; their ORIGIN scores come from a paired fixture detector, not the production flow fixpoint.`,
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
