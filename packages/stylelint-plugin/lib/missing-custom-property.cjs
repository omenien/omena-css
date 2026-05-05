const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "css-module-explainer/missing-custom-property";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (propertyName) => `CSS custom property '${propertyName}' not found.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-custom-property",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
