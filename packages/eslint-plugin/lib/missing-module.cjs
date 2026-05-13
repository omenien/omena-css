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
      description: "Report unresolved CSS Module imports.",
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
          includeMissingModule: { type: "boolean" },
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
          includeMissingModule: true,
          includeCodes: ["missing-module"],
        }).filter((finding) => finding.code === "missing-module");

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
