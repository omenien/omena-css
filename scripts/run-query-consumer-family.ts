// g131-S6: single driver for the query-consumer thin-driver family — one invocation =
// one former member (former script slug), gate ids and outputs unchanged.
import { QUERY_CONSUMER_FAMILY as FAMILY } from "./lib/query-consumer-family";

const slug = process.argv[2];
if (!slug || !FAMILY[slug]) {
  process.stderr.write(
    `usage: run-query-consumer-family.ts <member>\nmembers:\n${Object.keys(FAMILY)
      .map((name) => `  ${name}`)
      .join("\n")}\n`,
  );
  process.exit(2);
}
FAMILY[slug]().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
