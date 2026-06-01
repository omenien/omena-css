import { test, expect } from "../_fixtures/protocol";
import type { ResolvedType } from "@omena/shared";
import type { TypeResolver } from "../../server/engine-core-ts/src/core/ts/type-resolver";

const UNKNOWN_DOCUMENT_POSITION = { line: 0, character: 0 };

test("returns capabilities including definitionProvider on initialize", async ({ makeClient }) => {
  const client = makeClient();
  const result = await client.initialize();
  expect(result.capabilities.definitionProvider).toBe(true);
  expect(result.capabilities.hoverProvider).toBe(true);
  expect(result.serverInfo?.name).toBe("omena-css");
});

test("advertises completionProvider with every trigger character", async ({ makeClient }) => {
  const client = makeClient();
  const result = await client.initialize();
  const provider = result.capabilities.completionProvider;
  expect(provider).toBeDefined();
  expect(provider?.triggerCharacters).toEqual(["'", '"', "`", ",", ".", "$", "@", "-"]);
  expect(provider?.resolveProvider).toBe(false);
});

test("completes the initialize → initialized → shutdown handshake cleanly", async ({
  makeClient,
}) => {
  const client = makeClient();
  await client.initialize();
  client.initialized();
  await client.shutdown();
  client.exit();
});

test("disposes workspace runtime state during shutdown", async ({ makeClient }) => {
  let clearCalls = 0;
  const typeResolver: TypeResolver = {
    resolve(): ResolvedType {
      return { kind: "unresolvable", values: [] };
    },
    invalidate() {},
    clear() {
      clearCalls += 1;
    },
  };
  const client = makeClient({ typeResolver });

  await client.initialize();
  client.initialized();
  await client.shutdown();

  expect(clearCalls).toBe(1);
  client.exit();
});

test("returns null for a definition request on an unknown document", async ({ makeClient }) => {
  const client = makeClient();
  // No initialize, no didOpen — the server neither crashes nor
  // throws a JSON-RPC error. `null` is the agreed-upon
  // "nothing to say" shape (spec §2.8 + §4.2). This exercises
  // composition-root's toCursorParams unknown-doc branch, not
  // the pre-initialize `!deps` guard.
  const result = await client.definition({
    textDocument: { uri: "file:///never/opened.tsx" },
    position: UNKNOWN_DOCUMENT_POSITION,
  });
  expect(result).toBeNull();
});
