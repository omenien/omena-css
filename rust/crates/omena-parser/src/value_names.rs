pub(crate) const VALUES_L4_MATH_FUNCTION_NAMES: &[&str] = &[
    "min", "max", "clamp", "round", "mod", "rem", "sin", "cos", "tan", "asin", "acos", "atan",
    "atan2", "pow", "sqrt", "hypot", "log", "exp", "abs", "sign",
];

pub(crate) const CSS_COLOR_FUNCTION_NAMES: &[&str] = &[
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "color",
    "color-mix",
    "device-cmyk",
    "light-dark",
    "contrast-color",
];

pub(crate) const CSS_GRADIENT_FUNCTION_NAMES: &[&str] = &[
    "linear-gradient",
    "radial-gradient",
    "conic-gradient",
    "repeating-linear-gradient",
    "repeating-radial-gradient",
    "repeating-conic-gradient",
];

pub(crate) const CSS_TRANSFORM_FUNCTION_NAMES: &[&str] = &[
    "matrix",
    "matrix3d",
    "translate",
    "translate3d",
    "translateX",
    "translateY",
    "translateZ",
    "scale",
    "scale3d",
    "scaleX",
    "scaleY",
    "scaleZ",
    "rotate",
    "rotate3d",
    "rotateX",
    "rotateY",
    "rotateZ",
    "skew",
    "skewX",
    "skewY",
    "perspective",
];

pub(crate) const CSS_FILTER_FUNCTION_NAMES: &[&str] = &[
    "blur",
    "brightness",
    "contrast",
    "drop-shadow",
    "grayscale",
    "hue-rotate",
    "invert",
    "opacity",
    "saturate",
    "sepia",
];

pub(crate) const CSS_IMAGE_FUNCTION_NAMES: &[&str] =
    &["image", "image-set", "cross-fade", "element", "paint"];

pub(crate) const CSS_SHAPE_FUNCTION_NAMES: &[&str] = &[
    "path", "shape", "ray", "inset", "circle", "ellipse", "polygon",
];
