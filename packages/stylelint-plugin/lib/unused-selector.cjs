const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/unused-selector";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (selectorName) => `Selector '.${selectorName}' is declared but never used.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "unused-selector",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
