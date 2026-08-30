import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  RUST_PRODUCT_TEST_SHARDS,
  resolveRustProductTestShard,
  rustProductTestCargoInvocations,
} from "./lib/rust-product-test-plan";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const forwardedArgs = process.argv.slice(2);
if (forwardedArgs[0] === "--") {
  forwardedArgs.shift();
}
const requestedShard = forwardedArgs[0] ?? "all";
const unexpectedArgs = forwardedArgs.slice(1);

if (unexpectedArgs.length > 0) {
  throw new Error(`Unexpected Rust product-test arguments: ${unexpectedArgs.join(" ")}`);
}

const shards =
  requestedShard === "all"
    ? RUST_PRODUCT_TEST_SHARDS
    : [resolveRustProductTestShard(requestedShard)];
const failedShards: string[] = [];

for (const shard of shards) {
  let shardFailed = false;
  for (const invocation of rustProductTestCargoInvocations(shard)) {
    process.stdout.write(
      `${JSON.stringify({
        schemaVersion: "1",
        product: "rust.product-test-execution",
        shard: shard.id,
        invocation: invocation.id,
        command: ["cargo", ...invocation.args],
      })}\n`,
    );

    const result = spawnSync("cargo", invocation.args, {
      cwd: repoRoot,
      env: process.env,
      stdio: "inherit",
      shell: false,
    });
    if (result.error) {
      throw result.error;
    }
    if (result.signal) {
      throw new Error(
        `Rust product-test shard "${shard.id}" invocation "${invocation.id}" terminated by ${result.signal}`,
      );
    }
    if (result.status !== 0) {
      shardFailed = true;
    }
  }
  if (shardFailed) {
    failedShards.push(shard.id);
  }
}

if (failedShards.length > 0) {
  throw new Error(`Rust product-test shard failure: ${failedShards.join(", ")}`);
}
