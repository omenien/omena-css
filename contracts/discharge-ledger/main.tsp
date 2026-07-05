import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.DischargeLedger;

alias DischargeLedgerSchemaVersionV1 = "1";

enum DischargeVerdictV1 {
  accepted: "accepted",
  rejected: "rejected",
  unknown: "unknown",
}

enum DischargeSatResultV1 {
  sat: "sat",
  unsat: "unsat",
  unknown: "unknown",
}

enum DischargeBoundednessKindV1 {
  exact: "exact",
  boundedK: "boundedK",
}

model DischargeBoundednessV1 {
  kind: DischargeBoundednessKindV1;
  k?: int32;
}

model DischargeLedgerPinsV1 {
  theorySignatureHash: string;
  specDigest: string;
  encoderContentHash: string;
  solverVersion: string;
}

model DischargeLedgerEntryV1 {
  obligationFamily: string;
  cellFamily: string;
  obligationId: string;
  l1Primitive: string;
  cellKey: string;
  canonicalTermCount: int32;
  canonicalTerms: string[];
  verdict: DischargeVerdictV1;
  boundedness: DischargeBoundednessV1;
  referenceKind: string;
  referenceSatResult: DischargeSatResultV1;
  solverKind: string;
  solverSatResult: DischargeSatResultV1;
  referenceMatchesSolver: boolean;
  pins: DischargeLedgerPinsV1;
}

model DischargeLedgerCoverageV1 {
  obligationFamily: string;
  cellFamily: string;
  cellCount: int32;
  exhaustive: boolean;
  bound?: string;
}

model DischargeLedgerV1 {
  schemaVersion: DischargeLedgerSchemaVersionV1;
  product: string;
  pins: DischargeLedgerPinsV1;
  coverage: DischargeLedgerCoverageV1[];
  entries: DischargeLedgerEntryV1[];
}
