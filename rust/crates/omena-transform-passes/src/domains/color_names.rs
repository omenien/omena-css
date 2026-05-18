use super::SrgbColor;

pub(super) fn parse_basic_named_srgb_color(text: &str) -> Option<SrgbColor> {
    match text.to_ascii_lowercase().as_str() {
        "aliceblue" => Some(SrgbColor {
            red: 240,
            green: 248,
            blue: 255,
        }),
        "antiquewhite" => Some(SrgbColor {
            red: 250,
            green: 235,
            blue: 215,
        }),
        "aqua" | "cyan" => Some(SrgbColor {
            red: 0,
            green: 255,
            blue: 255,
        }),
        "aquamarine" => Some(SrgbColor {
            red: 127,
            green: 255,
            blue: 212,
        }),
        "azure" => Some(SrgbColor {
            red: 240,
            green: 255,
            blue: 255,
        }),
        "beige" => Some(SrgbColor {
            red: 245,
            green: 245,
            blue: 220,
        }),
        "bisque" => Some(SrgbColor {
            red: 255,
            green: 228,
            blue: 196,
        }),
        "black" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 0,
        }),
        "blanchedalmond" => Some(SrgbColor {
            red: 255,
            green: 235,
            blue: 205,
        }),
        "blue" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 255,
        }),
        "blueviolet" => Some(SrgbColor {
            red: 138,
            green: 43,
            blue: 226,
        }),
        "brown" => Some(SrgbColor {
            red: 165,
            green: 42,
            blue: 42,
        }),
        "burlywood" => Some(SrgbColor {
            red: 222,
            green: 184,
            blue: 135,
        }),
        "cadetblue" => Some(SrgbColor {
            red: 95,
            green: 158,
            blue: 160,
        }),
        "chartreuse" => Some(SrgbColor {
            red: 127,
            green: 255,
            blue: 0,
        }),
        "chocolate" => Some(SrgbColor {
            red: 210,
            green: 105,
            blue: 30,
        }),
        "coral" => Some(SrgbColor {
            red: 255,
            green: 127,
            blue: 80,
        }),
        "cornflowerblue" => Some(SrgbColor {
            red: 100,
            green: 149,
            blue: 237,
        }),
        "cornsilk" => Some(SrgbColor {
            red: 255,
            green: 248,
            blue: 220,
        }),
        "crimson" => Some(SrgbColor {
            red: 220,
            green: 20,
            blue: 60,
        }),
        "darkblue" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 139,
        }),
        "darkcyan" => Some(SrgbColor {
            red: 0,
            green: 139,
            blue: 139,
        }),
        "darkgoldenrod" => Some(SrgbColor {
            red: 184,
            green: 134,
            blue: 11,
        }),
        "darkgray" | "darkgrey" => Some(SrgbColor {
            red: 169,
            green: 169,
            blue: 169,
        }),
        "darkgreen" => Some(SrgbColor {
            red: 0,
            green: 100,
            blue: 0,
        }),
        "darkkhaki" => Some(SrgbColor {
            red: 189,
            green: 183,
            blue: 107,
        }),
        "darkmagenta" => Some(SrgbColor {
            red: 139,
            green: 0,
            blue: 139,
        }),
        "darkolivegreen" => Some(SrgbColor {
            red: 85,
            green: 107,
            blue: 47,
        }),
        "darkorange" => Some(SrgbColor {
            red: 255,
            green: 140,
            blue: 0,
        }),
        "darkorchid" => Some(SrgbColor {
            red: 153,
            green: 50,
            blue: 204,
        }),
        "darkred" => Some(SrgbColor {
            red: 139,
            green: 0,
            blue: 0,
        }),
        "darksalmon" => Some(SrgbColor {
            red: 233,
            green: 150,
            blue: 122,
        }),
        "darkseagreen" => Some(SrgbColor {
            red: 143,
            green: 188,
            blue: 143,
        }),
        "darkslateblue" => Some(SrgbColor {
            red: 72,
            green: 61,
            blue: 139,
        }),
        "darkslategray" | "darkslategrey" => Some(SrgbColor {
            red: 47,
            green: 79,
            blue: 79,
        }),
        "darkturquoise" => Some(SrgbColor {
            red: 0,
            green: 206,
            blue: 209,
        }),
        "darkviolet" => Some(SrgbColor {
            red: 148,
            green: 0,
            blue: 211,
        }),
        "deeppink" => Some(SrgbColor {
            red: 255,
            green: 20,
            blue: 147,
        }),
        "deepskyblue" => Some(SrgbColor {
            red: 0,
            green: 191,
            blue: 255,
        }),
        "dimgray" | "dimgrey" => Some(SrgbColor {
            red: 105,
            green: 105,
            blue: 105,
        }),
        "dodgerblue" => Some(SrgbColor {
            red: 30,
            green: 144,
            blue: 255,
        }),
        "firebrick" => Some(SrgbColor {
            red: 178,
            green: 34,
            blue: 34,
        }),
        "floralwhite" => Some(SrgbColor {
            red: 255,
            green: 250,
            blue: 240,
        }),
        "forestgreen" => Some(SrgbColor {
            red: 34,
            green: 139,
            blue: 34,
        }),
        "fuchsia" | "magenta" => Some(SrgbColor {
            red: 255,
            green: 0,
            blue: 255,
        }),
        "gainsboro" => Some(SrgbColor {
            red: 220,
            green: 220,
            blue: 220,
        }),
        "ghostwhite" => Some(SrgbColor {
            red: 248,
            green: 248,
            blue: 255,
        }),
        "gold" => Some(SrgbColor {
            red: 255,
            green: 215,
            blue: 0,
        }),
        "goldenrod" => Some(SrgbColor {
            red: 218,
            green: 165,
            blue: 32,
        }),
        "gray" | "grey" => Some(SrgbColor {
            red: 128,
            green: 128,
            blue: 128,
        }),
        "green" => Some(SrgbColor {
            red: 0,
            green: 128,
            blue: 0,
        }),
        "greenyellow" => Some(SrgbColor {
            red: 173,
            green: 255,
            blue: 47,
        }),
        "honeydew" => Some(SrgbColor {
            red: 240,
            green: 255,
            blue: 240,
        }),
        "hotpink" => Some(SrgbColor {
            red: 255,
            green: 105,
            blue: 180,
        }),
        "indianred" => Some(SrgbColor {
            red: 205,
            green: 92,
            blue: 92,
        }),
        "indigo" => Some(SrgbColor {
            red: 75,
            green: 0,
            blue: 130,
        }),
        "ivory" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 240,
        }),
        "khaki" => Some(SrgbColor {
            red: 240,
            green: 230,
            blue: 140,
        }),
        "lavender" => Some(SrgbColor {
            red: 230,
            green: 230,
            blue: 250,
        }),
        "lavenderblush" => Some(SrgbColor {
            red: 255,
            green: 240,
            blue: 245,
        }),
        "lawngreen" => Some(SrgbColor {
            red: 124,
            green: 252,
            blue: 0,
        }),
        "lemonchiffon" => Some(SrgbColor {
            red: 255,
            green: 250,
            blue: 205,
        }),
        "lightblue" => Some(SrgbColor {
            red: 173,
            green: 216,
            blue: 230,
        }),
        "lightcoral" => Some(SrgbColor {
            red: 240,
            green: 128,
            blue: 128,
        }),
        "lightcyan" => Some(SrgbColor {
            red: 224,
            green: 255,
            blue: 255,
        }),
        "lightgoldenrodyellow" => Some(SrgbColor {
            red: 250,
            green: 250,
            blue: 210,
        }),
        "lightgray" | "lightgrey" => Some(SrgbColor {
            red: 211,
            green: 211,
            blue: 211,
        }),
        "lightgreen" => Some(SrgbColor {
            red: 144,
            green: 238,
            blue: 144,
        }),
        "lightpink" => Some(SrgbColor {
            red: 255,
            green: 182,
            blue: 193,
        }),
        "lightsalmon" => Some(SrgbColor {
            red: 255,
            green: 160,
            blue: 122,
        }),
        "lightseagreen" => Some(SrgbColor {
            red: 32,
            green: 178,
            blue: 170,
        }),
        "lightskyblue" => Some(SrgbColor {
            red: 135,
            green: 206,
            blue: 250,
        }),
        "lightslategray" | "lightslategrey" => Some(SrgbColor {
            red: 119,
            green: 136,
            blue: 153,
        }),
        "lightsteelblue" => Some(SrgbColor {
            red: 176,
            green: 196,
            blue: 222,
        }),
        "lightyellow" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 224,
        }),
        "lime" => Some(SrgbColor {
            red: 0,
            green: 255,
            blue: 0,
        }),
        "limegreen" => Some(SrgbColor {
            red: 50,
            green: 205,
            blue: 50,
        }),
        "linen" => Some(SrgbColor {
            red: 250,
            green: 240,
            blue: 230,
        }),
        "maroon" => Some(SrgbColor {
            red: 128,
            green: 0,
            blue: 0,
        }),
        "mediumaquamarine" => Some(SrgbColor {
            red: 102,
            green: 205,
            blue: 170,
        }),
        "mediumblue" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 205,
        }),
        "mediumorchid" => Some(SrgbColor {
            red: 186,
            green: 85,
            blue: 211,
        }),
        "mediumpurple" => Some(SrgbColor {
            red: 147,
            green: 112,
            blue: 219,
        }),
        "mediumseagreen" => Some(SrgbColor {
            red: 60,
            green: 179,
            blue: 113,
        }),
        "mediumslateblue" => Some(SrgbColor {
            red: 123,
            green: 104,
            blue: 238,
        }),
        "mediumspringgreen" => Some(SrgbColor {
            red: 0,
            green: 250,
            blue: 154,
        }),
        "mediumturquoise" => Some(SrgbColor {
            red: 72,
            green: 209,
            blue: 204,
        }),
        "mediumvioletred" => Some(SrgbColor {
            red: 199,
            green: 21,
            blue: 133,
        }),
        "midnightblue" => Some(SrgbColor {
            red: 25,
            green: 25,
            blue: 112,
        }),
        "mintcream" => Some(SrgbColor {
            red: 245,
            green: 255,
            blue: 250,
        }),
        "mistyrose" => Some(SrgbColor {
            red: 255,
            green: 228,
            blue: 225,
        }),
        "moccasin" => Some(SrgbColor {
            red: 255,
            green: 228,
            blue: 181,
        }),
        "navajowhite" => Some(SrgbColor {
            red: 255,
            green: 222,
            blue: 173,
        }),
        "navy" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 128,
        }),
        "oldlace" => Some(SrgbColor {
            red: 253,
            green: 245,
            blue: 230,
        }),
        "olive" => Some(SrgbColor {
            red: 128,
            green: 128,
            blue: 0,
        }),
        "olivedrab" => Some(SrgbColor {
            red: 107,
            green: 142,
            blue: 35,
        }),
        "orange" => Some(SrgbColor {
            red: 255,
            green: 165,
            blue: 0,
        }),
        "orangered" => Some(SrgbColor {
            red: 255,
            green: 69,
            blue: 0,
        }),
        "orchid" => Some(SrgbColor {
            red: 218,
            green: 112,
            blue: 214,
        }),
        "palegoldenrod" => Some(SrgbColor {
            red: 238,
            green: 232,
            blue: 170,
        }),
        "palegreen" => Some(SrgbColor {
            red: 152,
            green: 251,
            blue: 152,
        }),
        "paleturquoise" => Some(SrgbColor {
            red: 175,
            green: 238,
            blue: 238,
        }),
        "palevioletred" => Some(SrgbColor {
            red: 219,
            green: 112,
            blue: 147,
        }),
        "papayawhip" => Some(SrgbColor {
            red: 255,
            green: 239,
            blue: 213,
        }),
        "peachpuff" => Some(SrgbColor {
            red: 255,
            green: 218,
            blue: 185,
        }),
        "peru" => Some(SrgbColor {
            red: 205,
            green: 133,
            blue: 63,
        }),
        "pink" => Some(SrgbColor {
            red: 255,
            green: 192,
            blue: 203,
        }),
        "plum" => Some(SrgbColor {
            red: 221,
            green: 160,
            blue: 221,
        }),
        "powderblue" => Some(SrgbColor {
            red: 176,
            green: 224,
            blue: 230,
        }),
        "purple" => Some(SrgbColor {
            red: 128,
            green: 0,
            blue: 128,
        }),
        "rebeccapurple" => Some(SrgbColor {
            red: 102,
            green: 51,
            blue: 153,
        }),
        "red" => Some(SrgbColor {
            red: 255,
            green: 0,
            blue: 0,
        }),
        "rosybrown" => Some(SrgbColor {
            red: 188,
            green: 143,
            blue: 143,
        }),
        "royalblue" => Some(SrgbColor {
            red: 65,
            green: 105,
            blue: 225,
        }),
        "saddlebrown" => Some(SrgbColor {
            red: 139,
            green: 69,
            blue: 19,
        }),
        "salmon" => Some(SrgbColor {
            red: 250,
            green: 128,
            blue: 114,
        }),
        "sandybrown" => Some(SrgbColor {
            red: 244,
            green: 164,
            blue: 96,
        }),
        "seagreen" => Some(SrgbColor {
            red: 46,
            green: 139,
            blue: 87,
        }),
        "seashell" => Some(SrgbColor {
            red: 255,
            green: 245,
            blue: 238,
        }),
        "sienna" => Some(SrgbColor {
            red: 160,
            green: 82,
            blue: 45,
        }),
        "silver" => Some(SrgbColor {
            red: 192,
            green: 192,
            blue: 192,
        }),
        "skyblue" => Some(SrgbColor {
            red: 135,
            green: 206,
            blue: 235,
        }),
        "slateblue" => Some(SrgbColor {
            red: 106,
            green: 90,
            blue: 205,
        }),
        "slategray" | "slategrey" => Some(SrgbColor {
            red: 112,
            green: 128,
            blue: 144,
        }),
        "snow" => Some(SrgbColor {
            red: 255,
            green: 250,
            blue: 250,
        }),
        "springgreen" => Some(SrgbColor {
            red: 0,
            green: 255,
            blue: 127,
        }),
        "steelblue" => Some(SrgbColor {
            red: 70,
            green: 130,
            blue: 180,
        }),
        "tan" => Some(SrgbColor {
            red: 210,
            green: 180,
            blue: 140,
        }),
        "teal" => Some(SrgbColor {
            red: 0,
            green: 128,
            blue: 128,
        }),
        "thistle" => Some(SrgbColor {
            red: 216,
            green: 191,
            blue: 216,
        }),
        "tomato" => Some(SrgbColor {
            red: 255,
            green: 99,
            blue: 71,
        }),
        "turquoise" => Some(SrgbColor {
            red: 64,
            green: 224,
            blue: 208,
        }),
        "violet" => Some(SrgbColor {
            red: 238,
            green: 130,
            blue: 238,
        }),
        "wheat" => Some(SrgbColor {
            red: 245,
            green: 222,
            blue: 179,
        }),
        "white" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 255,
        }),
        "whitesmoke" => Some(SrgbColor {
            red: 245,
            green: 245,
            blue: 245,
        }),
        "yellow" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 0,
        }),
        "yellowgreen" => Some(SrgbColor {
            red: 154,
            green: 205,
            blue: 50,
        }),
        _ => None,
    }
}

pub(super) fn shortest_named_srgb_color(color: SrgbColor) -> Option<&'static str> {
    match (color.red, color.green, color.blue) {
        (0, 0, 128) => Some("navy"),
        (0, 128, 0) => Some("green"),
        (0, 128, 128) => Some("teal"),
        (75, 0, 130) => Some("indigo"),
        (128, 0, 0) => Some("maroon"),
        (128, 0, 128) => Some("purple"),
        (128, 128, 0) => Some("olive"),
        (128, 128, 128) => Some("gray"),
        (160, 82, 45) => Some("sienna"),
        (165, 42, 42) => Some("brown"),
        (192, 192, 192) => Some("silver"),
        (205, 133, 63) => Some("peru"),
        (210, 180, 140) => Some("tan"),
        (218, 112, 214) => Some("orchid"),
        (221, 160, 221) => Some("plum"),
        (238, 130, 238) => Some("violet"),
        (240, 230, 140) => Some("khaki"),
        (240, 255, 255) => Some("azure"),
        (245, 222, 179) => Some("wheat"),
        (245, 245, 220) => Some("beige"),
        (250, 128, 114) => Some("salmon"),
        (250, 240, 230) => Some("linen"),
        (255, 0, 0) => Some("red"),
        (255, 99, 71) => Some("tomato"),
        (255, 127, 80) => Some("coral"),
        (255, 165, 0) => Some("orange"),
        (255, 192, 203) => Some("pink"),
        (255, 215, 0) => Some("gold"),
        (255, 228, 196) => Some("bisque"),
        (255, 250, 250) => Some("snow"),
        (255, 255, 240) => Some("ivory"),
        _ => None,
    }
}
