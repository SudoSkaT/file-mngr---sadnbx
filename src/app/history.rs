use std::path::PathBuf;

pub struct History {
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
}

impl History {
    pub fn new() -> Self {
        Self {
            back: Vec::new(),
            forward: Vec::new(),
        }
    }

    pub fn visit(&mut self, dir: PathBuf) {
        self.back.push(dir);
        self.forward.clear();
    }

    pub fn go_back(&mut self, current: PathBuf) -> Option<PathBuf> {
        let target = self.back.pop()?;
        self.forward.push(current);
        Some(target)
    }

    pub fn go_forward(&mut self, current: PathBuf) -> Option<PathBuf> {
        let target = self.forward.pop()?;
        self.back.push(current);
        Some(target)
    }

    #[allow(dead_code)]
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    #[allow(dead_code)]
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}
