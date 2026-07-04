use crate::fs::node::Node;

#[derive(Clone, Debug, PartialEq)]
pub enum SearchKind {
    Fuzzy,
    Regex,
}

impl SearchKind {
    pub fn next(&self) -> Self {
        match self {
            Self::Fuzzy => Self::Regex,
            Self::Regex => Self::Fuzzy,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Fuzzy => "fuzzy",
            Self::Regex => "regex",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchState {
    pub query: String,
    pub kind: SearchKind,
    pub results: Vec<usize>,
    pub cursor: usize,
    pub scroll: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            kind: SearchKind::Fuzzy,
            results: Vec::new(),
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn execute(&mut self, nodes: &[Node]) {
        if self.query.is_empty() {
            self.results = (0..nodes.len()).collect();
        } else {
            let lower = self.query.to_lowercase();
            self.results = match self.kind {
                SearchKind::Fuzzy => {
                    let mut scored: Vec<(usize, u32)> = nodes
                        .iter()
                        .enumerate()
                        .filter_map(|(i, n)| fuzzy_score(&lower, &n.name.to_lowercase()).map(|s| (i, s)))
                        .collect();
                    scored.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| {
                        nodes[a.0].name.to_lowercase().cmp(&nodes[b.0].name.to_lowercase())
                    }));
                    scored.into_iter().map(|(i, _)| i).collect()
                }
                SearchKind::Regex => {
                    let re = match regex::Regex::new(&self.query) {
                        Ok(r) => r,
                        Err(_) => return,
                    };
                    nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| re.is_match(&n.name))
                        .map(|(i, _)| i)
                        .collect()
                }
            };
        }
        self.cursor = self.cursor.min(self.results.len().saturating_sub(1));
    }

    pub fn navigate_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        let max = self.results.len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn ensure_visible(&mut self, view_height: usize) {
        if self.results.is_empty() || view_height == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + view_height {
            self.scroll = self.cursor.saturating_add(1).saturating_sub(view_height);
        }
    }

}

fn fuzzy_score(query: &str, text: &str) -> Option<u32> {
    let mut score: u32 = 0;
    let mut qi = 0usize;
    let mut ti = 0usize;
    let qb = query.as_bytes();
    let tb = text.as_bytes();

    while qi < qb.len() {
        let (qc, qs) = decode_char(qb, qi);
        // scan forward in text for matching char
        loop {
            if ti >= tb.len() {
                return None;
            }
            let (tc, ts) = decode_char(tb, ti);
            if tc == qc {
                // separator bonus: if previous char is word separator, less penalty
                let mut pi = ti.saturating_sub(1);
                while pi > 0 && !is_char_boundary(tb, pi) {
                    pi -= 1;
                }
                if pi > 0 {
                    let (pc, _) = decode_char(tb, pi);
                    if pc != ' ' && pc != '_' && pc != '-' {
                        score += 5;
                    }
                }
                qi += qs;
                ti += ts;
                break;
            }
            ti += ts;
            score += 10;
        }
    }
    Some(score)
}

fn decode_char(bytes: &[u8], i: usize) -> (char, usize) {
    let s = std::str::from_utf8(&bytes[i..]).unwrap_or("");
    let c = s.chars().next().unwrap_or('\0');
    (c, c.len_utf8())
}

#[cfg(test)]
mod bench {
    use std::time::Instant;
    use super::fuzzy_score;

    #[test]
    fn bench_fuzzy_score() {
        let files: Vec<String> = (0..10_000)
            .map(|i| format!("file_{}_name_{}.rs", i % 100, i))
            .collect();
        let query = "file_name";

        let start = Instant::now();
        let mut count = 0u32;
        for f in &files {
            if let Some(s) = fuzzy_score(query, f) {
                count += s;
            }
        }
        let elapsed = start.elapsed();
        println!("[bench] fuzzy_score 10k files: {:?} (checksum: {})", elapsed, count);
        assert!(elapsed.as_millis() < 200, "fuzzy_score too slow: {:?}", elapsed);
    }
}

fn is_char_boundary(bytes: &[u8], i: usize) -> bool {
    bytes[i] & 0xC0 != 0x80
}
