const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-sass-symbol";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (symbolName) => `Sass symbol '${symbolName}' not found.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-sass-symbol",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
