module.exports = {
  plugins: ["stylelint-plugin-css-module-explainer"],
  rules: {
    "css-module-explainer/unused-selector": [true],
    "css-module-explainer/missing-composed-module": [true],
    "css-module-explainer/missing-composed-selector": [true],
    "css-module-explainer/missing-value-module": [true],
    "css-module-explainer/missing-imported-value": [true],
    "css-module-explainer/missing-keyframes": [true],
    "css-module-explainer/missing-custom-property": [true],
    "css-module-explainer/missing-sass-symbol": [true],
  },
};
