import type { Position, Range, ShowReferencesArgs, ShowReferencesLocation } from "@omena/shared";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isPosition(value: unknown): value is Position {
  if (!isRecord(value)) return false;
  return typeof value.line === "number" && typeof value.character === "number";
}

function isRange(value: unknown): value is Range {
  if (!isRecord(value)) return false;
  return isPosition(value.start) && isPosition(value.end);
}

function isShowReferencesLocation(value: unknown): value is ShowReferencesLocation {
  if (!isRecord(value)) return false;
  return typeof value.uri === "string" && isRange(value.range);
}

export function isShowReferencesArgs(value: readonly unknown[]): value is ShowReferencesArgs {
  if (value.length !== 3) return false;
  const [uri, position, locations] = value;
  if (typeof uri !== "string") return false;
  if (!isPosition(position)) return false;
  if (!Array.isArray(locations)) return false;
  return locations.every((loc) => isShowReferencesLocation(loc));
}
