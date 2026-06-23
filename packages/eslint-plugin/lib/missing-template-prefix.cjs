const {
  SOURCE_FILE_PATTERN,
  formatQueryDiagnostic,
  getRuleOptions,
  runSourceChecks,
  toEslintLoc,
} = require("./_shared.cjs");

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "Report template literal class prefixes that match no selector candidates.",
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
          includeCodes: ["missing-template-prefix"],
        }).filter((finding) => finding.code === "missing-template-prefix");

        for (const finding of findings) {
          context.report({
            loc: toEslintLoc(finding.range),
            message: formatQueryDiagnostic(finding, ruleOptions.workspaceRoot),
          });
        }
      },
    };
  },
};
