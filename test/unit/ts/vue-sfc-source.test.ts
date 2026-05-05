import { describe, expect, it } from "vitest";
import ts from "typescript";
import { projectVueSfcScriptToTypeScriptSource } from "../../../server/engine-core-ts/src/core/ts/vue-sfc-source";

describe("projectVueSfcScriptToTypeScriptSource", () => {
  it("keeps script content at original Vue SFC line and character positions", () => {
    const source = `<template>
  <button />
</template>
<script setup lang="ts">
import { useCssModule } from "vue";
const styles = useCssModule();
const className = styles.button;
</script>
<style module>
.button {}
</style>
`;

    const projected = projectVueSfcScriptToTypeScriptSource(source);
    const sourceFile = ts.createSourceFile(
      "/fake/Component.vue",
      projected,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const offset = projected.indexOf("styles.button");
    const position = sourceFile.getLineAndCharacterOfPosition(offset);

    expect(projected).toHaveLength(source.length);
    expect(position).toEqual({ line: 6, character: 18 });
    expect(projected.slice(0, source.indexOf("<script"))).not.toContain("<button");
  });
});
