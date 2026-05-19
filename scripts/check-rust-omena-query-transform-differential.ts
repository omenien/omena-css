import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { transform as lightningTransform } from "lightningcss";

interface TransformExecuteSummaryV0 {
  readonly product: string;
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly product: string;
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly mutationCount: number;
    readonly provenancePreserved: boolean;
    readonly passPlan: {
      readonly violatedDagEdgeCount: number;
      readonly allRequestedRegistered: boolean;
    };
  };
}

interface DifferentialFixture {
  readonly label: string;
  readonly source: string;
}

const passIds = [
  "whitespace-strip",
  "comment-strip",
  "number-compression",
  "unit-normalization",
  "color-compression",
  "url-quote-strip",
  "string-quote-normalize",
  "selector-is-where-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "media-static-eval",
  "calc-reduction",
  "print-css",
] as const;

const fixtures: readonly DifferentialFixture[] = [
  {
    label: "token-value-minification",
    source:
      '.a { color: #FFFFFF; opacity: 1.0; background: url("x.svg"); width: 0.50rem; margin: 0px; }',
  },
  {
    label: "integer-leading-zero-number",
    source: ".a { z-index: 001; opacity: 000.50; }",
  },
  {
    label: "selector-list-and-spacing",
    source: '.a , .b { color : #FFFFFF ; opacity: 1.0; background: url("x.svg"); }',
  },
  {
    label: "adjacent-duplicate-color-declarations",
    source: ".a { color: rgb(255 0 0); color: rgb(255 0 0 / 100%); background: blue; }",
  },
  {
    label: "post-color-selector-merge",
    source:
      ".a { color: rgb(255 0 0 / 0%); } .b { color: rgb(255 0 0 / 50%); } .c { color: rgb(255, 0, 0, 0.5); }",
  },
  {
    label: "post-value-selector-merge",
    source:
      ".a { margin: 0px; } .b { margin: 0; } .c { width: calc(1px + 2px); } .d { width: 3px; }",
  },
  {
    label: "is-where-and-shorthand",
    source:
      ".a:is(.ready) { color: #FFFFFF; margin-top: 0px; margin-right: 0px; margin-bottom: 0px; margin-left: 0px; }",
  },
  {
    label: "box-shorthand-value-compression",
    source: ".a { margin: 1px 1px 1px 1px; padding: 1px 2px 3px 2px; }",
  },
  {
    label: "flex-shorthand-compression",
    source:
      ".a { flex: 0 1 auto; } .b { flex: 1 1 0%; } .c { flex: 1 2 0%; } .d { flex: 0 0 auto; } .e { flex-direction: row; flex-wrap: nowrap; } .f { flex-flow: row nowrap; } .g { flex-flow: column wrap; } .h{flex-direction:row;flex-wrap:nowrap}.i{flex-flow:row nowrap}.j{flex-grow:1;flex-shrink:1;flex-basis:0%}.k{flex-grow:1;flex-shrink:1;flex-basis:10px}.l{flex:1 1 0px}",
  },
  {
    label: "border-radius-shorthand-compression",
    source: ".a { border-radius: 1px 1px 1px 1px; }",
  },
  {
    label: "border-radius-longhand-compression",
    source:
      ".a { border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; }",
  },
  {
    label: "border-radius-ellipse-longhand-compression",
    source:
      ".a { border-top-left-radius: 1px 2px; border-top-right-radius: 3px 4px; border-bottom-right-radius: 1px 2px; border-bottom-left-radius: 3px 4px; }",
  },
  {
    label: "border-radius-slash-shorthand-compression",
    source:
      ".a { border-radius: 1px 1px 1px 1px / 2px 2px 2px 2px; } .b { border-radius: 1px 2px 1px 2px / 3px 4px 3px 4px; } .c { border-radius: 1px / 1px; }",
  },
  {
    label: "inset-shorthand-compression",
    source: ".a { inset: 1px 2px 1px 2px; }",
  },
  {
    label: "inset-longhand-compression",
    source: ".a { top: 1px; right: 2px; bottom: 1px; left: 2px; }",
  },
  {
    label: "list-style-shorthand-compression",
    source:
      ".a { list-style: disc outside none; } .b { list-style: none outside none; } .c { list-style: url(icon.svg) outside none; }",
  },
  {
    label: "list-style-longhand-compression",
    source: ".a { list-style-type: none; list-style-position: outside; list-style-image: none; }",
  },
  {
    label: "structural-rule-merge",
    source:
      ".dupe { display: block; } .dupe { display: block; } .sel-a { border: 0; } .sel-b { border: 0; } .merge { color: red; } .merge { background: #0000FF; }",
  },
  {
    label: "rule-merge-semicolonless",
    source: ".b{color:red}.b{background:blue}",
  },
  {
    label: "comment-empty-calc",
    source: "/* head */ .calc { width: calc(1px + 2px); } .empty { } /* tail */",
  },
  {
    label: "nested-comment-empty-rules",
    source:
      ".empty { } @supports (display: grid) { .nested { } .filled { color: red; } } .outer { .inner { } } .with-comment { /* remove after comment strip */ } .filled { color: red; }",
  },
  {
    label: "keyframes-empty-frame",
    source: "@keyframes fade { 0% {} to { opacity: 1 } } .empty{}",
  },
  {
    label: "keyframes-selector-aliases",
    source: "@keyframes fade { from { opacity: 0 } 100% { opacity: 1 } 50%, TO { opacity: .5 } }",
  },
  {
    label: "media-range-normalization",
    source: "@media screen and (min-width: 1px) and (max-width: 10px) { .a { color: red; } }",
  },
  {
    label: "media-range-calc-reduction",
    source:
      "@media (min-width: calc(1px + 1px)) and (max-height: clamp(1rem, 2rem, 3rem)) { .a { color: red; } }",
  },
  {
    label: "supports-group-color-compression",
    source:
      "@supports not (display: grid) { .a { color: red; } } @supports (display: grid) or (unknown: value) { .b { color: blue; } }",
  },
  {
    label: "linear-gradient-default-direction",
    source:
      ".a { background: linear-gradient(to bottom, red, blue); } .b { background-image: repeating-linear-gradient(180deg, white, black); } .c { list-style-image: linear-gradient(0.5turn, red, blue); } .d { mask-image: linear-gradient(200grad, red, blue); } .e { background: linear-gradient(0deg, red 10%, blue 90%); } .f { background-image: repeating-linear-gradient(to top, white, black); } .g { background: linear-gradient(to right, red, blue); } .h { background-image: repeating-linear-gradient(to left, white, black); }",
  },
  {
    label: "radial-conic-gradient-defaults",
    source:
      ".a { background: radial-gradient(circle at center, red, blue); } .b { background: radial-gradient(ellipse at center, red, blue); } .c { background: conic-gradient(from 0deg, red, blue); } .d { background: repeating-conic-gradient(from 0turn, red, blue); }",
  },
  {
    label: "calc-same-unit-nested",
    source: ".a { margin: calc(2rem + 3rem); padding: calc(10px - 4px); }",
  },
  {
    label: "calc-additive-chain",
    source:
      ".a { width: calc(2px + 3px + 4px); height: calc(.5rem + .25rem + .25rem); margin: calc(10px - 3px - 2px); }",
  },
  {
    label: "calc-parenthesized-multiplicative-chain",
    source:
      ".a { width: calc((1px + 2px)); height: calc(2px * 3 * 4); margin: calc(24px / 2 / 3); }",
  },
  {
    label: "nested-min-max-functions",
    source: ".a { width: min(10px, max(2px, 4px)); height: max(1px, min(4px, 2px)); }",
  },
  {
    label: "clamp-static-value",
    source: ".a { opacity: clamp(.1, .5, .9); }",
  },
  {
    label: "is-where-multi",
    source: ":is(.a) { color: #ffffff; } :where(.b) { color: #0000ff; }",
  },
  {
    label: "rule-selector-merge-with-named-color",
    source: ".a { color: red; } .b { color: red; } .a { background: blue; } .empty {}",
  },
  {
    label: "border-composite-named-color",
    source: ".a { border: 1px solid black; }",
  },
  {
    label: "extended-named-color-coverage",
    source:
      ".a { color: rebeccapurple; accent-color: #d2b48c; background: aliceblue; border-color: darkgray; outline-color: LightGoldenRodYellow; }",
  },
  {
    label: "current-color-keyword-case",
    source: ".a { color: currentcolor; border-color: CurrentColor; }",
  },
  {
    label: "single-keyword-property-case",
    source:
      ".a { cursor: POINTER; } .b { user-select: NONE; } .c { position: STICKY; } .d { text-align: MATCH-PARENT; } .e { visibility: HIDDEN; } .f { pointer-events: NONE; } .g { cursor: -WEBKIT-GRAB; }",
  },
  {
    label: "column-rule-currentcolor-shorthand",
    source: ".a { column-rule: medium none currentcolor; }",
  },
  {
    label: "columns-auto-and-column-rule",
    source: ".a { columns: auto auto; } .b { column-rule: medium none currentcolor; }",
  },
  {
    label: "column-rule-currentcolor-longhand",
    source:
      ".a { column-rule-width: medium; column-rule-style: none; column-rule-color: currentcolor; }",
  },
  {
    label: "border-outline-zero-shorthand-lengths",
    source:
      ".a { border: 0px solid #000000; } .b { border-top: 0px solid #000000; } .c { outline: 0px solid #000000; } .d { text-decoration: underline 0px #000000; }",
  },
  {
    label: "border-outline-default-none-shorthands",
    source:
      ".a { border: medium none currentcolor; } .b { border-top: currentcolor medium none; } .c { outline: medium none currentcolor; }",
  },
  {
    label: "display-multi-keyword-aliases",
    source:
      ".a { display: block flow; } .b { display: inline flow; } .c { display: block flow-root; } .d { display: inline flow-root; } .e { display: inline flex; } .f { display: block grid; } .g { display: list-item block flow; } .h { display: BLOCK; } .i { display: INLINE RUBY; } .j { display: list-item inline flow; } .k { display: block flow list-item; } .l { display: list-item flow-root; } .m { display: INITIAL; } .n { display: INLINE BLOCK; }",
  },
  {
    label: "url-zero-font-family",
    source: '.a { background: url("/icons/a.svg"); margin: 0 0 0 0%; font-family: "Inter"; }',
  },
  {
    label: "position-zero-percent-normalization",
    source:
      ".a { perspective-origin: 0% 0%; transform-origin: 0% 0%; opacity: 0%; background-position: 0% 0%; background-size: auto auto; mask-position: 0% 0%; }",
  },
  {
    label: "center-position-normalization",
    source:
      ".bg { background-position: center center; } .left { background-position: left center; } .origin { transform-origin: center top; } .mask { mask-position: bottom right; } .mask-axis { mask-position-x: center; mask-position-y: center; }",
  },
  {
    label: "background-size-position-normalization",
    source: ".a { background-position: center center; background-size: auto auto; }",
  },
  {
    label: "background-position-percent-center",
    source:
      ".a { transform-origin: 50% 0%; mask-position: 100% 50%; background-position: 0% 50%; -webkit-mask-position: 50% 50%; }",
  },
  {
    label: "opacity-percentage-normalization",
    source:
      ".a { opacity: 50%; } .b { opacity: 100%; } .c { fill-opacity: 100%; stroke-opacity: 50%; flood-opacity: 0%; stop-opacity: 5%; }",
  },
  {
    label: "aspect-ratio-spacing-normalization",
    source: ".a { aspect-ratio: 16 / 9; } .b { aspect-ratio: auto 4 / 3; }",
  },
  {
    label: "shadow-zero-length-normalization",
    source:
      ".a { box-shadow: 0px 0px 0px #000; } .b { box-shadow: inset 1px 2px 0px 0px #000; } .c { text-shadow: 1px 2px 0px #000; }",
  },
  {
    label: "time-unit-shortening",
    source: ".a { transition-duration: 100ms; transition-delay: .05s; animation-delay: 0ms; }",
  },
  {
    label: "motion-shorthand-defaults",
    source:
      ".a { transition: all 0s ease 0s; } .b { transition: opacity 0s linear .1s; } .c { transition: opacity 0s ease 0s, color .2s ease 0s; } .d { animation: none 0s ease 0s 1 normal none running; } .e { animation: 0s ease 0s 1 normal none running fade; }",
  },
  {
    label: "transition-longhand-compression",
    source:
      ".a { transition-property: all; transition-duration: 0s; transition-timing-function: ease; transition-delay: 0s; } .b { transition-property: opacity; transition-duration: .2s; transition-timing-function: ease; transition-delay: 0s; } .c { transition-property: all !important; transition-duration: 0s !important; transition-timing-function: ease !important; transition-delay: 0s !important; }",
  },
  {
    label: "transform-zero-unit-normalization",
    source: ".a { transform: rotate(0deg) translate(0px); }",
  },
  {
    label: "transform-scale-repeat-normalization",
    source: ".a { transform: scale(1, 1) scale(2, 2); }",
  },
  {
    label: "transform-zero-axis-normalization",
    source:
      ".a { transform: translateX(0px) translateY(0px) translateZ(0px) translate(0px, 0px) perspective(0px); }",
  },
  {
    label: "transform-skew-translate-tail-zero",
    source: ".a { transform: translate(1px, 0px) skew(0deg, 0deg) skewX(0deg) skewY(0turn); }",
  },
  {
    label: "transform-3d-axis-normalization",
    source:
      ".a { transform: scale(2, 1) scale3d(1, 1, 1) scale3d(2, 3, 1) scale3d(1, 1, 2) rotate3d(1, 0, 0, 0deg) rotate3d(0, 1, 0, 1turn) rotate3d(0, 0, 1, 10deg) translate3d(0px, 0px, 0px) translate3d(1px, 0px, 0px) translate3d(0px, 1px, 0px) translate3d(0px, 0px, 1px) translate3d(1px, 2px, 0px); }",
  },
  {
    label: "filter-default-functions",
    source:
      ".a { filter: opacity(100%) brightness(1) contrast(+1) saturate(0100%) blur(0px) hue-rotate(-0deg); } .b { backdrop-filter: opacity(.5) blur(1px); } .c { -webkit-filter: opacity(1.0); } .d { filter: drop-shadow(red 0px 0px 0px); } .e { filter: drop-shadow(1px 2px 0px #000); }",
  },
  {
    label: "individual-transform-properties",
    source:
      ".t0 { translate: 0px 0% 0px; } .t1 { translate: 1px 0px; } .t2 { translate: 0px 1px; } .t3 { translate: 1px 2px 0px; } .t4 { translate: 0px 0px 1px; } .s0 { scale: 1 1; } .s1 { scale: 2 2; } .s2 { scale: 1 2; } .s3 { scale: 2 3 1; } .s4 { scale: 1 1 2; } .s5 { scale: 1 1 1; } .s6 { scale: 50% 50%; } .r0 { rotate: z 0deg; } .r1 { rotate: 0 0 1 10deg; } .r2 { rotate: 1 0 0 .500turn; } .r3 { rotate: 0 1 0 10.0deg; } .r4 { rotate: 0rad; }",
  },
  {
    label: "font-family-list",
    source: '.fonts { font-family: "Arial", "Helvetica Neue", "system-ui", sans-serif; }',
  },
  {
    label: "font-longhand-keywords",
    source:
      ".fonts { font-weight: normal; font-stretch: normal; } .bold { font-weight: bold; font-stretch: condensed; }",
  },
  {
    label: "font-longhand-overrides",
    source:
      ".a { font-stretch: 100%; font-stretch: 75%; font-stretch: 50%; } .b { font-weight: normal; font-weight: 400; }",
  },
  {
    label: "font-longhand-shorthand-compression",
    source:
      '.a { font-style: normal; font-variant-caps: normal; font-weight: normal; font-stretch: normal; font-size: 16px; line-height: normal; font-family: Arial; } .b { font-style: normal; font-variant-caps: normal; font-weight: bold; font-stretch: condensed; font-size: 16px; line-height: 1.5; font-family: Arial, sans-serif; } .c { font-style: italic; font-variant-caps: small-caps; font-weight: bold; font-stretch: condensed; font-size: 1rem; line-height: 120%; font-family: "Open Sans", serif; } .d { font-style: normal !important; font-variant-caps: normal !important; font-weight: normal !important; font-stretch: normal !important; font-size: 16px !important; line-height: normal !important; font-family: Arial !important; }',
  },
  {
    label: "overflow-background-repeat-shorthand",
    source:
      ".a { background-repeat: repeat repeat; overflow-x: visible; overflow-y: visible; } .b { background-repeat: repeat no-repeat; overflow: hidden hidden; } .c { background-repeat: no-repeat repeat; overflow: visible visible; } .d { overflow-x: auto; overflow-y: hidden; } .e { overflow-y: scroll; overflow-x: clip; } .f { overflow: AUTO HIDDEN; }",
  },
  {
    label: "background-position-axis-shorthand",
    source:
      ".a { background-position-x: left; background-position-y: top; } .b { background-position-x: center; background-position-y: center; } .c { background-position-y: top; background-position-x: center; } .d { background-position-x: left !important; background-position-y: top !important; }",
  },
  {
    label: "case-insensitive-shorthand-keywords",
    source: ".a { background-repeat: Repeat Repeat; list-style: NONE OUTSIDE NONE; }",
  },
  {
    label: "place-axis-shorthands",
    source:
      ".a { align-items: stretch; justify-items: stretch; } .b { align-content: center; justify-content: center; } .c { justify-self: end; align-self: start; } .d { align-items: start !important; justify-items: end !important; } .e { place-content: normal normal; } .f { place-items: stretch stretch; } .g { place-self: auto auto; } .h { align-items: first baseline; justify-items: center; } .i { justify-items: legacy left; align-items: normal; } .j { align-self: safe center; justify-self: unsafe end; } .k { align-content: space-between; justify-content: first baseline; }",
  },
  {
    label: "gap-axis-shorthands",
    source:
      ".a { row-gap: 1px; column-gap: 1px; } .b { gap: 2px 2px; } .c { column-gap: 2px; row-gap: 1px; } .d { row-gap: 1px !important; column-gap: 2px !important; }",
  },
  {
    label: "scroll-box-shorthands",
    source:
      ".a { scroll-margin-top: 1px; scroll-margin-right: 2px; scroll-margin-bottom: 1px; scroll-margin-left: 2px; } .b { scroll-padding-top: 1px; scroll-padding-right: 1px; scroll-padding-bottom: 1px; scroll-padding-left: 1px; } .c { scroll-margin: 3px 3px; }",
  },
  {
    label: "text-decoration-shorthands",
    source:
      ".a { text-decoration-line: underline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .b { text-decoration: underline solid red auto; } .c { text-decoration-line: underline; text-decoration-style: wavy; text-decoration-color: red; text-decoration-thickness: 1px; } .d { text-decoration-line: underline !important; text-decoration-style: solid !important; text-decoration-color: currentcolor !important; text-decoration-thickness: auto !important; } .e { text-decoration-line: overline underline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .f { text-decoration-line: none underline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; }",
  },
  {
    label: "text-emphasis-shorthands",
    source:
      ".a { text-emphasis-style: none; text-emphasis-color: currentcolor; } .b { text-emphasis-style: filled dot; text-emphasis-color: red; } .c { text-emphasis-style: open sesame !important; text-emphasis-color: currentcolor !important; } .d { text-emphasis-position: over right; } .e { text-emphasis-position: left under; } .f { text-emphasis-position: over left; }",
  },
  {
    label: "logical-axis-shorthands",
    source:
      ".a { padding-block-start: 1px; padding-block-end: 1px; } .b { margin-inline-start: 1px; margin-inline-end: 2px; } .c { inset-block-end: 2px; inset-block-start: 1px; } .d { border-block-start-color: red; border-block-end-color: red; } .e { border-inline-start-width: 1px; border-inline-end-width: 2px; } .f { padding-block-start: 1px !important; padding-block-end: 2px !important; }",
  },
  {
    label: "logical-four-side-axis-shorthands",
    source:
      ".a { inset-block-start: 1px; inset-inline-end: 2px; inset-block-end: 1px; inset-inline-start: 2px; } .b { margin-block-start: 1px; margin-inline-end: 2px; margin-block-end: 3px; margin-inline-start: 4px; } .c { border-block-start-color: red; border-inline-end-color: blue; border-block-end-color: red; border-inline-start-color: blue; } .d { border-block-start-width: 1px; border-block-end-width: 1px; border-inline-start-width: 1px; border-inline-end-width: 1px; }",
  },
  {
    label: "scroll-logical-axis-shorthands",
    source:
      ".a { scroll-margin-block-start: 1px; scroll-margin-block-end: 1px; } .b { scroll-padding-inline-end: 2px; scroll-padding-inline-start: 1px; } .c { scroll-margin-inline-start: 1px !important; scroll-margin-inline-end: 2px !important; }",
  },
  {
    label: "line-style-shorthands",
    source:
      ".a { border-top-width: 1px; border-top-style: solid; border-top-color: red; } .b { border-width: medium; border-style: none; border-color: currentcolor; } .c { outline-width: medium; outline-style: solid; outline-color: currentcolor; } .d { outline-width: 1px; outline-style: none; outline-color: red; } .e { border-inline-width: medium !important; border-inline-style: none !important; border-inline-color: currentcolor !important; } .f { border-color: red; border-style: solid; border-width: 1px; } .g { border-width: 1px 1px 1px 1px; border-style: solid solid solid solid; border-color: red red red red; }",
  },
  {
    label: "logical-border-line-shorthands",
    source:
      ".a { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; } .b { border-block-start: 1px solid red; border-block-end: 1px solid red; } .c { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; border-block-end-width: 1px; border-block-end-style: solid; border-block-end-color: red; } .d { border-inline-end: 1px solid red; border-inline-start: 1px solid red; }",
  },
  {
    label: "border-side-shorthand-compression",
    source:
      ".a { border-top: 1px solid red; border-right: 1px solid red; border-bottom: 1px solid red; border-left: 1px solid red; } .b { border-top: 1px solid red !important; border-right: 1px solid red !important; border-bottom: 1px solid red !important; border-left: 1px solid red !important; }",
  },
  {
    label: "border-image-longhand-compression",
    source:
      ".a { border-image-source: url(a.png); border-image-slice: 10; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; } .b { border-image-source: linear-gradient(red, blue); border-image-slice: 10 20; border-image-width: auto; border-image-outset: 1; border-image-repeat: round; } .c { border-image-source: url(a.png); border-image-slice: 10 fill; border-image-width: 2; border-image-outset: 0; border-image-repeat: round space; }",
  },
  {
    label: "repeated-axis-shorthand-values",
    source:
      ".a { mask-repeat: repeat repeat; background-repeat: space round; -webkit-mask-repeat: no-repeat no-repeat; } .b { border-spacing: 1px 1px; } .c { scroll-margin-block: 1px 2px; scroll-padding-inline: 1px 1px; } .d { margin-block: 1px 2px; padding-inline: 2px 2px; } .e { border-block-color: red red; border-inline-width: 1px 1px; border-image-slice: 100% 100% 100% 100%; border-image-width: 1 1 1 1; border-image-outset: 0 0 0 0; } .f { mask-repeat: no-repeat repeat; background-repeat: repeat no-repeat; -webkit-mask-repeat: repeat no-repeat; }",
  },
  {
    label: "spacing-zero-units",
    source:
      ".a { border-spacing: 0px 0px; letter-spacing: 0px; word-spacing: 0px; outline-offset: 0px; text-indent: 0px; stroke-width: 0px; stroke-dasharray: 0px; stroke-dashoffset: 0px; tab-size: 0px; vertical-align: 0px; perspective: 0px; border-image-width: 0px; flex-basis: 0px; grid-template-columns: 0px 1fr; grid-auto-rows: 0px; font-size: 0px; }",
  },
  {
    label: "unit-normalized-duplicate-declarations",
    source: ".a { tab-size: 0px; tab-size: 0; opacity: 100%; opacity: 1; width: 0px; width: 0; }",
  },
  {
    label: "overridden-flex-longhands",
    source:
      ".a { flex-basis: 0%; flex: 1 1 0%; } .b { flex-grow: 1; flex-shrink: 1; flex: 2 1 0%; }",
  },
  {
    label: "alpha-hex-zero-line-height-calc",
    source:
      ".alpha { color: #ffffffff; border-color: #00000000; width: calc(2px * 3); height: calc(6px / 2); line-height: 0em; }",
  },
  {
    label: "opaque-rgba-hsla",
    source:
      ".opaque { color: rgba(255, 0, 0, 1); text-decoration-color: hsla(240, 100%, 50%, 100%); }",
  },
];

const reports = fixtures.map((fixture) => {
  const omena = runOmenaTransform(fixture);
  const lightning = runLightningTransform(fixture);

  assert.equal(omena.product, "omena-query.transform-execute", fixture.label);
  assert.equal(omena.execution.product, "omena-transform-passes.execution", fixture.label);
  assert.deepEqual(omena.unknownPassIds, [], fixture.label);
  assert.equal(omena.execution.passPlan.violatedDagEdgeCount, 0, fixture.label);
  assert.equal(omena.execution.passPlan.allRequestedRegistered, true, fixture.label);
  assert.equal(omena.execution.provenancePreserved, true, fixture.label);
  assert.deepEqual(
    omena.execution.outputCss,
    lightning,
    `${fixture.label} should match lightningcss minified output for the supported CSS subset`,
  );

  return {
    label: fixture.label,
    byteLength: omena.execution.outputCss.length,
    mutationCount: omena.execution.mutationCount,
    executedPassCount: omena.execution.executedPassIds.length,
  };
});

process.stdout.write(
  [
    "validated omena-query transform differential against lightningcss:",
    `fixtures=${reports.length}`,
    `bytes=${reports.reduce((sum, report) => sum + report.byteLength, 0)}`,
    `mutations=${reports.reduce((sum, report) => sum + report.mutationCount, 0)}`,
  ].join(" "),
);
process.stdout.write("\n");

function runOmenaTransform(fixture: DifferentialFixture): TransformExecuteSummaryV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      "transform-execute",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify({
        stylePath: `${fixture.label}.css`,
        styleSource: fixture.source,
        requestedPassIds: passIds,
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);

  return JSON.parse(result.stdout) as TransformExecuteSummaryV0;
}

function runLightningTransform(fixture: DifferentialFixture): string {
  const result = lightningTransform({
    filename: `${fixture.label}.css`,
    code: Buffer.from(fixture.source),
    minify: true,
  });

  return String(result.code);
}
