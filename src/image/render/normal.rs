use ratatui::text::Line;

use super::build_half_block_lines;
use super::fit_contain;
use super::scale_rgba;

pub fn render_frame(
    rgba: &[u8],
    width: u32,
    height: u32,
    panel_w: u16,
    panel_h: u16,
) -> Vec<Line<'static>> {
    if width == 0 || height == 0 || panel_w == 0 || panel_h == 0 {
        return vec![];
    }

    let cell_w = panel_w as u32;
    let cell_h = (panel_h as u32).saturating_mul(2);

    let (fit_w, fit_h) = fit_contain(width, height, cell_w, cell_h);
    let fit_h = fit_h - (fit_h % 2);

    let (scaled, sw, sh) = if fit_w != width || fit_h != height {
        scale_rgba(rgba, width, height, fit_w.max(1), fit_h.max(1))
    } else {
        (rgba.to_vec(), fit_w.max(1), fit_h.max(1))
    };

    build_half_block_lines(&scaled, sw, sh, cell_w, panel_h)
}
