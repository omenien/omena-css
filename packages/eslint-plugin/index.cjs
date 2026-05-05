const sourceCheckRule = require("./lib/source-check.cjs");
const missingModuleRule = require("./lib/missing-module.cjs");
const missingStaticClassRule = require("./lib/missing-static-class.cjs");
const missingTemplatePrefixRule = require("./lib/missing-template-prefix.cjs");
const missingResolvedClassValuesRule = require("./lib/missing-resolved-class-values.cjs");
const missingResolvedClassDomainRule = require("./lib/missing-resolved-class-domain.cjs");
const invalidClassReferenceRule = require("./lib/invalid-class-reference.cjs");
const noUnknownDynamicClassRule = require("./lib/no-unknown-dynamic-class.cjs");
const noImpossibleSelectorRule = require("./lib/no-impossible-selector.cjs");
const noImpreciseValueRule = require("./lib/no-imprecise-value.cjs");

const FOCUSED_SOURCE_RULES = {
  "css-module-explainer/missing-module": "error",
  "css-module-explainer/missing-static-class": "error",
  "css-module-explainer/missing-template-prefix": "error",
  "css-module-explainer/missing-resolved-class-values": "error",
  "css-module-explainer/missing-resolved-class-domain": "error",
};

const plugin = {
  meta: {
    name: "eslint-plugin-css-module-explainer",
    version: "0.0.1",
  },
  rules: {
    "missing-module": missingModuleRule,
    "missing-static-class": missingStaticClassRule,
    "missing-template-prefix": missingTemplatePrefixRule,
    "missing-resolved-class-values": missingResolvedClassValuesRule,
    "missing-resolved-class-domain": missingResolvedClassDomainRule,
    "invalid-class-reference": invalidClassReferenceRule,
    "no-unknown-dynamic-class": noUnknownDynamicClassRule,
    "no-impossible-selector": noImpossibleSelectorRule,
    "no-imprecise-value": noImpreciseValueRule,
    "source-check": sourceCheckRule,
  },
};

plugin.configs = {
  recommended: [
    {
      plugins: {
        "css-module-explainer": plugin,
      },
      rules: {
        "css-module-explainer/source-check": "error",
      },
    },
  ],
  focused: [
    {
      plugins: {
        "css-module-explainer": plugin,
      },
      rules: {
        ...FOCUSED_SOURCE_RULES,
      },
    },
  ],
  dynamicMoat: [
    {
      plugins: {
        "css-module-explainer": plugin,
      },
      rules: {
        "css-module-explainer/no-unknown-dynamic-class": "error",
      },
    },
  ],
  mTier: [
    {
      plugins: {
        "css-module-explainer": plugin,
      },
      rules: {
        "css-module-explainer/no-impossible-selector": "error",
        "css-module-explainer/no-imprecise-value": "error",
      },
    },
  ],
};

module.exports = plugin;
