const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-value-module";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (specifier) => `Cannot resolve imported @value module '${specifier}'.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-value-module",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
