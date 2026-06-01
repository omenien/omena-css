const missingModuleRule = require("./missing-module.cjs");
const invalidClassReferenceRule = require("./invalid-class-reference.cjs");

module.exports = {
  meta: {
    ...invalidClassReferenceRule.meta,
    docs: {
      description: "Run all source-side Omena CSS Modules semantic checks.",
    },
  },

  create(context) {
    const missingModuleVisitors = missingModuleRule.create(context);
    const invalidClassVisitors = invalidClassReferenceRule.create(context);

    return {
      "Program:exit"() {
        missingModuleVisitors["Program:exit"]?.();
        invalidClassVisitors["Program:exit"]?.();
      },
    };
  },
};
