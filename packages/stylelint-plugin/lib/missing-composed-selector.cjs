const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-composed-selector";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (selectorName) => `Selector '.${selectorName}' not found in composed module.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-composed-selector",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
