// g131-S6: the single driver for the collapsed rust-shadow thin-driver
// family. One invocation = one former member, addressed by its former script
// slug, so every gate id, command surface, and output stream is unchanged —
// only the 42 single-file drivers are retired into the table.
import { RUST_SHADOW_FAMILY } from "./lib/rust-shadow-family";

const slug = process.argv[2];
if (!slug) {
  const rows = Object.entries(RUST_SHADOW_FAMILY)
    .map(([name, row]) => `  ${name} (${row.corpus})`)
    .join("\n");
  process.stderr.write(`usage: run-rust-shadow-family.ts <member-slug>\nmembers:\n${rows}\n`);
  process.exit(2);
}
const member = RUST_SHADOW_FAMILY[slug];
if (!member) {
  process.stderr.write(`unknown rust-shadow family member "${slug}"\n`);
  process.exit(2);
}
member.run().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
