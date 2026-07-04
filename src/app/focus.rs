#[derive(Clone, Debug, PartialEq)]
pub enum Focus {
    Tree,
    Directory,
    Preview,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Self::Tree => Self::Directory,
            Self::Directory => Self::Preview,
            Self::Preview => Self::Tree,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Tree => Self::Preview,
            Self::Directory => Self::Tree,
            Self::Preview => Self::Directory,
        }
    }
}
