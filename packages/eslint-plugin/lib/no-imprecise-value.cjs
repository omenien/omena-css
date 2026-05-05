const {
  SOURCE_FILE_PATTERN,
  formatCheckerFinding,
  getRuleOptions,
  runSourceChecks,
  toEslintLoc,
} = require("./_shared.cjs");

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Report dynamic CSS Module class domains that cannot be proven against known selectors.",
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
        }).filter((finding) => finding.code === "missing-resolved-class-domain");

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
