const stylelint = require("stylelint");
const { createFindingRule } = require("./_shared.cjs");

const ruleName = "omena/missing-keyframes";
const messages = stylelint.utils.ruleMessages(ruleName, {
  rejected: (animationName) => `@keyframes '${animationName}' not found in this file.`,
});

const plugin = createFindingRule({
  stylelint,
  ruleName,
  code: "missing-keyframes",
});

plugin.ruleName = ruleName;
plugin.messages = messages;

module.exports = plugin;
module.exports.ruleName = ruleName;
module.exports.messages = messages;
