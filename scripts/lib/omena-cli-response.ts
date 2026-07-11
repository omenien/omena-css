import { strict as assert } from "node:assert";

interface OmenaCliResponseEnvelope {
  readonly schemaVersion: "0";
  readonly product: string;
  readonly configContentDigest?: string;
  readonly payload: unknown;
}

export function parseOmenaCliResponse<T>(stdout: string, expectedProduct: string): T {
  const value = JSON.parse(stdout) as unknown;
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  const envelope = value as Partial<OmenaCliResponseEnvelope>;
  assert.equal(envelope.schemaVersion, "0", `${expectedProduct} envelope version drifted`);
  assert.equal(envelope.product, expectedProduct, `${expectedProduct} envelope product drifted`);
  assert.ok("payload" in envelope, `${expectedProduct} envelope payload is missing`);
  if (envelope.configContentDigest !== undefined) {
    assert.equal(typeof envelope.configContentDigest, "string");
  }
  return envelope.payload as T;
}
