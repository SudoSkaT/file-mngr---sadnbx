use ratatui::text::Line;

use super::build_half_block_lines;
use super::fit_contain;
use super::scale_rgba;

const CGA_PALETTE: &[(u8, u8, u8)] = &[
    (0x00, 0x00, 0x00), // black
    (0x00, 0x00, 0xAA), // blue
    (0x00, 0xAA, 0x00), // green
    (0x00, 0xAA, 0xAA), // cyan
    (0xAA, 0x00, 0x00), // red
    (0xAA, 0x00, 0xAA), // magenta
    (0xAA, 0x55, 0x00), // brown
    (0xAA, 0xAA, 0xAA), // light gray
    (0x55, 0x55, 0x55), // dark gray
    (0x55, 0x55, 0xFF), // light blue
    (0x55, 0xFF, 0x55), // light green
    (0x55, 0xFF, 0xFF), // light cyan
    (0xFF, 0x55, 0x55), // light red
    (0xFF, 0x55, 0xFF), // light magenta
    (0xFF, 0xFF, 0x55), // yellow
    (0xFF, 0xFF, 0xFF), // white
];

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

    let dithered = floyd_steinberg(&scaled, sw, sh);

    build_half_block_lines(&dithered, sw, sh, cell_w, panel_h)
}

fn floyd_steinberg(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let total = (w * h) as usize;
    let mut pixels: Vec<[f32; 3]> = Vec::with_capacity(total);

    for i in 0..total {
        let idx = i * 4;
        pixels.push([
            rgba[idx] as f32,
            rgba[idx + 1] as f32,
            rgba[idx + 2] as f32,
        ]);
    }

    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let r = pixels[i][0].round() as u8;
            let g = pixels[i][1].round() as u8;
            let b = pixels[i][2].round() as u8;

            let (nq_r, nq_g, nq_b) = nearest_cga(r, g, b);

            let er = pixels[i][0] - nq_r as f32;
            let eg = pixels[i][1] - nq_g as f32;
            let eb = pixels[i][2] - nq_b as f32;

            pixels[i] = [nq_r as f32, nq_g as f32, nq_b as f32];

            if x + 1 < w {
                let ri = (y * w + x + 1) as usize;
                pixels[ri][0] += er * 7.0 / 16.0;
                pixels[ri][1] += eg * 7.0 / 16.0;
                pixels[ri][2] += eb * 7.0 / 16.0;
            }
            if y + 1 < h {
                if x > 0 {
                    let ri = ((y + 1) * w + x - 1) as usize;
                    pixels[ri][0] += er * 3.0 / 16.0;
                    pixels[ri][1] += eg * 3.0 / 16.0;
                    pixels[ri][2] += eb * 3.0 / 16.0;
                }
                let ri = ((y + 1) * w + x) as usize;
                pixels[ri][0] += er * 5.0 / 16.0;
                pixels[ri][1] += eg * 5.0 / 16.0;
                pixels[ri][2] += eb * 5.0 / 16.0;
                if x + 1 < w {
                    let ri = ((y + 1) * w + x + 1) as usize;
                    pixels[ri][0] += er * 1.0 / 16.0;
                    pixels[ri][1] += eg * 1.0 / 16.0;
                    pixels[ri][2] += eb * 1.0 / 16.0;
                }
            }
        }
    }

    let mut result = Vec::with_capacity(total * 4);
    for p in &pixels {
        result.push(p[0].round() as u8);
        result.push(p[1].round() as u8);
        result.push(p[2].round() as u8);
        result.push(255);
    }
    result
}

fn nearest_cga(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;

    let mut best = CGA_PALETTE[0];
    let mut best_dist = f32::MAX;

    for &(pr, pg, pb) in CGA_PALETTE {
        let dr = rf - pr as f32;
        let dg = gf - pg as f32;
        let db = bf - pb as f32;
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best = (pr, pg, pb);
        }
    }

    best
}
