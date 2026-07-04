pub mod normal;
pub mod retro;

use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use image::imageops::FilterType;
use image::RgbaImage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RendererKind {
    Normal,
    Retro,
}

impl RendererKind {
    pub fn render(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
        panel_w: u16,
        panel_h: u16,
    ) -> Vec<Line<'static>> {
        match self {
            Self::Normal => normal::render_frame(rgba, width, height, panel_w, panel_h),
            Self::Retro => retro::render_frame(rgba, width, height, panel_w, panel_h),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Normal => Self::Retro,
            Self::Retro => Self::Normal,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Normal => "Normal",
            Self::Retro => "Retro",
        }
    }
}

pub(crate) fn fit_contain(img_w: u32, img_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    let ratio_w = max_w as f64 / img_w as f64;
    let ratio_h = max_h as f64 / img_h as f64;
    let scale = ratio_w.min(ratio_h).min(1.0);
    (
        (img_w as f64 * scale).round() as u32,
        (img_h as f64 * scale).round() as u32,
    )
}

pub(crate) fn scale_rgba(
    rgba: &[u8],
    w: u32,
    h: u32,
    new_w: u32,
    new_h: u32,
) -> (Vec<u8>, u32, u32) {
    let img = RgbaImage::from_raw(w, h, rgba.to_vec()).unwrap_or_else(|| RgbaImage::new(1, 1));
    let resized =
        image::imageops::resize(&img, new_w.max(1), new_h.max(1), FilterType::CatmullRom);
    (resized.into_raw(), new_w.max(1), new_h.max(1))
}

pub(crate) fn pixel_color(data: &[u8], stride: u32, y: u32, x: u32) -> Color {
    let idx = ((y * stride + x) * 4) as usize;
    if idx + 2 < data.len() {
        Color::Rgb(data[idx], data[idx + 1], data[idx + 2])
    } else {
        Color::Black
    }
}

pub(crate) fn build_half_block_lines(
    rgba: &[u8],
    sw: u32,
    sh: u32,
    cell_w: u32,
    panel_h: u16,
) -> Vec<Line<'static>> {
    let pad_x = (cell_w.saturating_sub(sw)) / 2;
    let pad_y = (panel_h as u32).saturating_sub(sh / 2) / 2;

    let mut lines: Vec<Line<'static>> = Vec::new();

    for _ in 0..pad_y {
        lines.push(Line::from(vec![Span::raw("")]));
    }

    for y in (0..sh).step_by(2) {
        let mut spans = Vec::with_capacity((sw + pad_x * 2) as usize);

        for _ in 0..pad_x {
            spans.push(Span::raw(" "));
        }

        for x in 0..sw {
            let top = pixel_color(rgba, sw, y, x);
            let bottom = if y + 1 < sh {
                pixel_color(rgba, sw, y + 1, x)
            } else {
                Color::Black
            };
            spans.push(Span::styled("▄", Style::default().fg(bottom).bg(top)));
        }

        lines.push(Line::from(spans));
    }

    lines
}
