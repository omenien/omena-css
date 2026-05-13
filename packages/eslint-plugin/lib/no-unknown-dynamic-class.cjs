const {
  SOURCE_FILE_PATTERN,
  formatCheckerFinding,
  getRuleOptions,
  runSourceChecks,
  toEslintLoc,
} = require("./_shared.cjs");

const DYNAMIC_FINDING_CODES = new Set([
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
]);

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Report dynamic CSS Module class expressions whose resolved values do not map to known selectors.",
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          workspaceRoot: { type: "string" },
          classnameTransform: {
            enum: ["asIs", "camelCase", "camelCaseOnly", "dashes", "dashesOnly"],
          },
          pathAlias: {
            type: "object",
            additionalProperties: { type: "string" },
          },
        },
      },
    ],
  },

  create(context) {
    const filename = context.filename;
    if (!filename || filename === "<input>" || !SOURCE_FILE_PATTERN.test(filename)) return {};

    return {
      "Program:exit"() {
        const ruleOptions = getRuleOptions(context);
        const findings = runSourceChecks(context, {
          ...ruleOptions,
          includeMissingModule: false,
          includeCodes: [...DYNAMIC_FINDING_CODES],
        }).filter((finding) => DYNAMIC_FINDING_CODES.has(finding.code));

        for (const finding of findings) {
          context.report({
            loc: toEslintLoc(finding.range),
            message: formatCheckerFinding(finding, ruleOptions.workspaceRoot),
          });
        }
      },
    };
  },
};
