use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use color_eyre::Result;
use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::AnimationDecoder;
use image::GenericImageView;
use image::ImageFormat;

const MAX_DECODE_DIM: u32 = 2000;

#[derive(Clone)]
pub struct FrameData {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

#[derive(Clone)]
pub struct DecodedImage {
    pub frames: Vec<FrameData>,
    pub is_animated: bool,
}

pub fn decode(path: &Path) -> Result<DecodedImage> {
    if is_video_extension(path) {
        return extract_video_frames(path);
    }
    if detect_format(path) == Some(ImageFormat::Gif) {
        return decode_gif(path);
    }
    decode_static(path)
}

fn decode_static(path: &Path) -> Result<DecodedImage> {
    let data = std::fs::read(path)?;
    let img = image::load_from_memory(&data)?;
    let (w, h) = img.dimensions();
    let rgba_img = if w > MAX_DECODE_DIM || h > MAX_DECODE_DIM {
        let scale = MAX_DECODE_DIM as f64 / w.max(h) as f64;
        let nw = (w as f64 * scale).round() as u32;
        let nh = (h as f64 * scale).round() as u32;
        image::imageops::resize(&img, nw.max(1), nh.max(1), FilterType::Lanczos3)
    } else {
        img.to_rgba8()
    };
    let (w, h) = rgba_img.dimensions();
    let rgba = rgba_img.into_raw();
    Ok(DecodedImage {
        frames: vec![FrameData { rgba, width: w, height: h, delay_ms: 0 }],
        is_animated: false,
    })
}

fn decode_gif(path: &Path) -> Result<DecodedImage> {
    use std::io::BufReader;
    let file = std::fs::File::open(path)?;
    let decoder = GifDecoder::new(BufReader::new(file))?;
    let frames: Vec<_> = decoder.into_frames().collect_frames()?;

    let is_animated = frames.len() > 1;
    let frame_data: Vec<FrameData> = frames
        .into_iter()
        .map(|f| {
            let (w, h) = f.buffer().dimensions();
            let rgba = f.buffer().clone().into_raw();
            let delay = f.delay().numer_denom_ms().0;
            FrameData {
                rgba,
                width: w,
                height: h,
                delay_ms: delay.max(50),
            }
        })
        .collect();

    Ok(DecodedImage {
        frames: frame_data,
        is_animated,
    })
}

fn extract_video_frames(path: &Path) -> Result<DecodedImage> {
    let tmp_dir = std::env::temp_dir().join(format!(
        "fm-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp_dir)?;

    let pattern = tmp_dir.join("frame_%03d.png");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &path.to_string_lossy(),
            "-vf",
            "fps=2,scale=320:-1:flags=lanczos",
            "-frames:v",
            "10",
            "-qscale:v",
            "2",
            &pattern.to_string_lossy(),
        ])
        .output();

    let frame_paths = match status {
        Ok(output) if output.status.success() => {
            let mut paths: Vec<_> = (1..=10)
                .map(|i| tmp_dir.join(format!("frame_{:03}.png", i)))
                .filter(|p| p.exists())
                .collect();
            paths.sort();
            paths
        }
        _ => Vec::new(),
    };

    if frame_paths.is_empty() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return decode_static(path);
    }

    let mut frames = Vec::new();
    for fp in &frame_paths {
        if let Ok(data) = std::fs::read(fp)
            && let Ok(img) = image::load_from_memory(&data)
        {
            let (w, h) = img.dimensions();
            let rgba = img.to_rgba8().into_raw();
            frames.push(FrameData {
                rgba,
                width: w,
                height: h,
                delay_ms: 500,
            });
        }
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);

    if frames.is_empty() {
        decode_static(path)
    } else {
        Ok(DecodedImage {
            is_animated: frames.len() > 1,
            frames,
        })
    }
}

fn detect_format(path: &Path) -> Option<ImageFormat> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => Some(ImageFormat::Png),
        Some("jpg") | Some("jpeg") => Some(ImageFormat::Jpeg),
        Some("gif") => Some(ImageFormat::Gif),
        Some("webp") => Some(ImageFormat::WebP),
        _ => None,
    }
}

fn is_video_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            matches!(
                e.to_lowercase().as_str(),
                "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_data(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name)
    }

    #[test]
    fn decode_webp_lossy() {
        let path = test_data("img.webp");
        let result = decode(&path);
        assert!(result.is_ok(), "lossy WebP should decode: {:?}", result.err());
        let img = result.unwrap();
        assert!(!img.frames.is_empty());
        assert_eq!(img.frames[0].width, 500);
        assert_eq!(img.frames[0].height, 333);
    }

    #[test]
    fn decode_non_image_returns_error() {
        let path = test_data("Cargo.toml");
        let result = decode(&path);
        assert!(result.is_err(), "non-image file should return error");
    }

    #[test]
    fn decode_nonexistent_returns_error() {
        let path = PathBuf::from("/nonexistent/file.webp");
        let result = decode(&path);
        assert!(result.is_err());
    }

    #[test]
    fn decode_roundtrip_webp_with_alpha() {
        let mut rgba = vec![0u8; 64 * 64 * 4];
        for y in 0..64 {
            for x in 0..64 {
                let i = (y * 64 + x) * 4;
                rgba[i] = (x * 4) as u8;
                rgba[i + 1] = (y * 4) as u8;
                rgba[i + 2] = 128;
                rgba[i + 3] = 255;
            }
        }
        let img = image::RgbaImage::from_raw(64, 64, rgba).unwrap();
        let mut bytes = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut bytes);
        img.write_with_encoder(encoder).unwrap();

        let decoded = decode_static_from_bytes(&bytes).unwrap();
        assert!(!decoded.frames.is_empty());
        assert_eq!(decoded.frames[0].width, 64);
        assert_eq!(decoded.frames[0].height, 64);
    }

    fn decode_static_from_bytes(data: &[u8]) -> Result<DecodedImage> {
        let img = image::load_from_memory(data)?;
        let (w, h) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();
        Ok(DecodedImage {
            frames: vec![FrameData { rgba, width: w, height: h, delay_ms: 0 }],
            is_animated: false,
        })
    }
}
