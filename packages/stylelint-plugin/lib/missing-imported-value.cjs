const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-imported-value";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (valueName) => `Imported @value '${valueName}' was not found in the target module.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-imported-value",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
