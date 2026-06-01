module.exports = {
  plugins: ["@omena/stylelint-plugin"],
  rules: {
    "omena/unused-selector": [true],
    "omena/missing-composed-module": [true],
    "omena/missing-composed-selector": [true],
    "omena/missing-value-module": [true],
    "omena/missing-imported-value": [true],
    "omena/missing-keyframes": [true],
    "omena/missing-custom-property": [true],
    "omena/missing-sass-symbol": [true],
  },
};
