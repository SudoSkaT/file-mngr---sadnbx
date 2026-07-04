const MAX_ERRORS: usize = 5;

#[derive(Clone, Debug)]
pub struct Errors {
    messages: Vec<String>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: String) {
        if self.messages.len() >= MAX_ERRORS {
            self.messages.remove(0);
        }
        self.messages.push(msg);
    }

    pub fn latest(&self) -> Option<&str> {
        self.messages.last().map(|s| s.as_str())
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
