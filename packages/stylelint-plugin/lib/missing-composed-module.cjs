const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-composed-module";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (specifier) => `Cannot resolve composed CSS Module '${specifier}'.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-composed-module",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
