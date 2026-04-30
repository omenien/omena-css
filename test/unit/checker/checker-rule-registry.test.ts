import { describe, expect, it } from "vitest";
import {
  getCheckerRuleDescriptor,
  isCheckerRuleCode,
  listCheckerRuleCodes,
  listCheckerRuleDescriptors,
} from "../../../server/engine-core-ts/src/core/checker";
import { listCheckerCodeBundles } from "../../../server/engine-core-ts/src/core/checker/checker-code-bundles";

describe("checker rule registry", () => {
  it("lists every current checker code with category, severity, fixability, and preset metadata", () => {
    expect(listCheckerRuleCodes()).toEqual([
      "missing-module",
      "missing-static-class",
      "missing-template-prefix",
      "missing-resolved-class-values",
      "missing-resolved-class-domain",
      "unused-selector",
      "missing-composed-module",
      "missing-composed-selector",
      "missing-value-module",
      "missing-imported-value",
      "missing-keyframes",
      "missing-custom-property",
      "missing-sass-symbol",
    ]);

    for (const descriptor of listCheckerRuleDescriptors()) {
      expect(descriptor.description.length).toBeGreaterThan(20);
      expect(["source", "style"]).toContain(descriptor.category);
      expect(["warning", "hint"]).toContain(descriptor.defaultSeverity);
      expect(["none", "codeAction", "autofix"]).toContain(descriptor.fixability);
      expect(descriptor.presets.length).toBeGreaterThan(0);
      expect(getCheckerRuleDescriptor(descriptor.code)).toBe(descriptor);
      expect(isCheckerRuleCode(descriptor.code)).toBe(true);
    }
    expect(isCheckerRuleCode("not-a-rule")).toBe(false);
  });

  it("keeps named checker bundles backed by registered rule codes", () => {
    const registered = new Set(listCheckerRuleCodes());
    for (const bundle of listCheckerCodeBundles()) {
      expect(bundle.codes.length).toBeGreaterThan(0);
      expect(bundle.codes.every((code) => registered.has(code))).toBe(true);
    }
  });
});
