// g131-S6: single driver for the contract-parity-smoke thin-driver family — one invocation =
// one former member (former script slug), gate ids and outputs unchanged.
import { CONTRACT_PARITY_SMOKE_FAMILY as FAMILY } from "./lib/contract-parity-smoke-family";

const slug = process.argv[2];
// Remove the slug so member bodies that parse argv see their own args only.
process.argv.splice(2, 1);
if (!slug || !FAMILY[slug]) {
  process.stderr.write(
    `usage: run-contract-parity-smoke-family.ts <member>\nmembers:\n${Object.keys(FAMILY)
      .map((name) => `  ${name}`)
      .join("\n")}\n`,
  );
  process.exit(2);
}
FAMILY[slug]().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
