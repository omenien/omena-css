#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleLanguage {
    Css,
    Scss,
    Less,
}

impl StyleLanguage {
    pub fn from_module_path(path: &str) -> Option<Self> {
        if path.ends_with(".module.css") || path.ends_with(".css") {
            Some(Self::Css)
        } else if path.ends_with(".module.scss") || path.ends_with(".scss") {
            Some(Self::Scss)
        } else if path.ends_with(".module.less") || path.ends_with(".less") {
            Some(Self::Less)
        } else {
            None
        }
    }
}
