use std::cell::Cell;

#[derive(Debug)]
pub struct PanelState {
    pub cursor: usize,
    pub scroll: Cell<usize>,
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            scroll: Cell::new(0),
        }
    }

    pub fn navigate_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn navigate_down(&mut self, max: usize) {
        if self.cursor + 1 < max {
            self.cursor += 1;
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.scroll.set(0);
    }

    pub fn ensure_visible(&self, view_height: usize, total: usize) {
        if total == 0 {
            return;
        }
        let view_height = view_height.max(1);
        let current = self.scroll.get();
        let new = if self.cursor < current {
            self.cursor
        } else if self.cursor >= current + view_height {
            self.cursor.saturating_sub(view_height).saturating_add(1)
        } else {
            current
        };
        self.scroll.set(new);
    }

    pub fn visible_range(&self, view_height: usize, total: usize) -> std::ops::Range<usize> {
        if total == 0 || view_height == 0 {
            return 0..0;
        }
        let start = self.scroll.get().min(total.saturating_sub(1));
        let end = (start + view_height).min(total);
        start..end
    }
}
