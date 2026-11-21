//! PNG export: the PDF export, rasterized one page per slide.
//!
//! There is no PNG pipeline of its own. The deck is printed to a scratch PDF by
//! exactly the code `slideflare export pdf` runs, and that PDF is rasterized here
//! with hayro, a PDF renderer written in Rust. So the images show what the PDF
//! shows, and the only platform code involved is the print path CI already
//! gates.
//!
//! Why hayro rather than pdfium or poppler: it has no native library, so there
//! is nothing to bundle, sign or notarize per platform, and it renders the same
//! pixels on all three. Its output is not pixel-identical to pdfium's — glyph
//! edges and one-pixel rules are anti-aliased differently — but nothing is
//! missing or misplaced, which is what an agent checking its slides needs. Text
//! shaping (Persian joining, bidi, ligatures) is done by the webview before the
//! PDF is written, so the renderer only ever draws glyphs that are already
//! shaped and embedded.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};

/// Largest image width accepted, in pixels.
///
/// hayro sizes its pixmap in `u16`, and anything near that limit is far past
/// what a slide preview needs; 8192 is 6.4x the design resolution.
pub const MAX_WIDTH: u32 = 8192;

/// Rasterize every page of `pdf` into `dir` as `slide-NN.png`, `width` pixels wide.
///
/// Returns the files written, in page order. Every page is rendered and encoded
/// before anything on disk is touched, so a failure leaves the previous render
/// in `dir` intact rather than half replaced.
pub fn rasterize(pdf: &Path, dir: &Path, width: u32) -> Result<Vec<PathBuf>, String> {
    let bytes = fs::read(pdf).map_err(|e| format!("Could not read {}: {}", pdf.display(), e))?;
    let images = render_pages(bytes, width)?;
    write_pages(dir, &images)
}

/// Render and PNG-encode every page of a PDF, `width` pixels wide.
fn render_pages(bytes: Vec<u8>, width: u32) -> Result<Vec<Vec<u8>>, String> {
    if width == 0 || width > MAX_WIDTH {
        return Err(format!("width must be between 1 and {MAX_WIDTH} pixels"));
    }

    let pdf = Pdf::new(bytes).map_err(|e| format!("Could not read the exported PDF: {e:?}"))?;

    // hayro reports what it could not draw rather than failing the page. Those
    // are collected and surfaced once each: a slide rendered without one of its
    // fonts is exactly what a visual check exists to catch, so it must not pass
    // silently.
    let warnings = Arc::new(Mutex::new(BTreeSet::new()));
    let sink = Arc::clone(&warnings);
    let settings = InterpreterSettings {
        warning_sink: Arc::new(move |warning| {
            if let Ok(mut seen) = sink.lock() {
                seen.insert(format!("{warning:?}"));
            }
        }),
        ..Default::default()
    };

    let cache = RenderCache::new();
    let mut images = Vec::with_capacity(pdf.pages().len());

    for page in pdf.pages().iter() {
        let (page_width, page_height) = page.render_dimensions();
        // Height follows the page's own aspect ratio, so a 960x540pt slide at
        // the default 1280 comes out at exactly 1280x720.
        let height = (page_height / page_width * width as f32).round().max(1.0) as u32;
        let pixmap = PixmapSettings {
            x_scale: scale_for(page_width, width),
            y_scale: scale_for(page_height, height),
            bg_color: WHITE,
        };

        let image = render(page, &cache, &settings, &RenderSettings::default(), &pixmap);
        let png = image
            .into_png()
            .map_err(|e| format!("Could not encode slide {}: {e:?}", images.len() + 1))?;
        images.push(png);
    }

    if images.is_empty() {
        return Err("the exported PDF has no pages".to_string());
    }

    if let Ok(seen) = warnings.lock() {
        for warning in seen.iter() {
            eprintln!("slideflare: warning: the PNG renderer reported {warning}");
        }
    }

    Ok(images)
}

/// Scale that turns `points` into exactly `pixels`.
///
/// hayro sizes the pixmap by truncating `points * scale`, and the plain quotient
/// can land a hair under the integer in `f32` — 960 * (2000 / 960) is
/// 1999.9999 — which would make every image one pixel short and stretch it to
/// fit. Stepping up one representable value at a time fixes that without ever
/// overshooting into the next pixel.
fn scale_for(points: f32, pixels: u32) -> f32 {
    let mut scale = pixels as f32 / points;
    while ((points * scale) as u32) < pixels {
        scale = scale.next_up();
    }
    scale
}

/// Replace the previous render in `dir` with `images`.
///
/// Stale `slide-NN.png` files are removed first. Rendering into the same
/// directory again is the normal loop — edit, render, look — and a deck that
/// lost a slide would otherwise leave its old last slide behind, looking current.
/// Only files matching that exact pattern are touched.
fn write_pages(dir: &Path, images: &[Vec<u8>]) -> Result<Vec<PathBuf>, String> {
    if dir.is_file() {
        return Err(format!(
            "{}: is a file; PNG export writes a directory of images",
            dir.display()
        ));
    }

    fs::create_dir_all(dir).map_err(|e| format!("Could not create {}: {}", dir.display(), e))?;

    let entries =
        fs::read_dir(dir).map_err(|e| format!("Could not read {}: {}", dir.display(), e))?;
    for entry in entries.flatten() {
        if is_slide_image(&entry.file_name().to_string_lossy()) {
            let path = entry.path();
            fs::remove_file(&path)
                .map_err(|e| format!("Could not remove {}: {}", path.display(), e))?;
        }
    }

    // At least two digits, so a listing sorts in slide order up to 99 slides
    // and every name in one render has the same width beyond that.
    let digits = images.len().to_string().len().max(2);
    let mut written = Vec::with_capacity(images.len());

    for (index, png) in images.iter().enumerate() {
        let path = dir.join(format!("slide-{:0digits$}.png", index + 1));
        fs::write(&path, png).map_err(|e| format!("Could not write {}: {}", path.display(), e))?;
        written.push(path);
    }

    Ok(written)
}

/// Whether `name` is one of ours: `slide-` + digits + `.png`.
fn is_slide_image(name: &str) -> bool {
    name.strip_prefix("slide-")
        .and_then(|rest| rest.strip_suffix(".png"))
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, well-formed PDF with one 960x540pt page per entry in `fills`,
    /// each filled edge to edge with that grey level (0-1).
    ///
    /// Built by hand, xref offsets and all, so the tests need no fixture file and
    /// no print pipeline.
    fn pdf_with_pages(fills: &[f32]) -> Vec<u8> {
        let pages = fills.len();
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {pages} >>",
                (0..pages)
                    .map(|i| format!("{} 0 R", 3 + 2 * i))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ];
        for (i, grey) in fills.iter().enumerate() {
            let content = format!("{grey} g 0 0 960 540 re f");
            objects.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 960 540] /Contents {} 0 R >>",
                4 + 2 * i
            ));
            objects.push(format!(
                "<< /Length {} >>\nstream\n{content}\nendstream",
                content.len()
            ));
        }

        let mut out = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend(format!("{} 0 obj\n{object}\nendobj\n", i + 1).bytes());
        }
        let xref = out.len();
        out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
        for offset in offsets {
            out.extend(format!("{offset:010} 00000 n \n").bytes());
        }
        out.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .bytes(),
        );
        out
    }

    /// Width and height from a PNG's IHDR chunk.
    fn png_size(png: &[u8]) -> (u32, u32) {
        let be = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        (be(16), be(20))
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("slideflare-png-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn renders_one_image_per_page_at_the_requested_size() {
        let images = render_pages(pdf_with_pages(&[0.0, 0.5, 1.0]), 1280).unwrap();
        assert_eq!(images.len(), 3);
        for png in &images {
            assert_eq!(png_size(png), (1280, 720));
        }
    }

    #[test]
    fn sizes_that_truncate_badly_in_f32_still_come_out_exact() {
        // 960pt at 2000px is 1999.9999 in plain f32 arithmetic.
        for width in [2000, 1999, 1920, 640, 333, 1] {
            let png = &render_pages(pdf_with_pages(&[0.0]), width).unwrap()[0];
            let height = (540.0f32 / 960.0 * width as f32).round().max(1.0) as u32;
            assert_eq!(png_size(png), (width, height), "width {width}");
        }
    }

    #[test]
    fn rejects_a_width_out_of_range() {
        assert!(render_pages(pdf_with_pages(&[0.0]), 0).is_err());
        assert!(render_pages(pdf_with_pages(&[0.0]), MAX_WIDTH + 1).is_err());
    }

    #[test]
    fn rejects_something_that_is_not_a_pdf() {
        assert!(render_pages(b"not a pdf".to_vec(), 1280).is_err());
    }

    #[test]
    fn replaces_a_previous_render_without_touching_other_files() {
        let dir = scratch_dir("replace");
        fs::create_dir_all(&dir).unwrap();
        // A longer previous render, plus files that merely look similar.
        for name in [
            "slide-01.png",
            "slide-02.png",
            "slide-03.png",
            "notes.png",
            "slide-x.png",
        ] {
            fs::write(dir.join(name), b"old").unwrap();
        }

        let images = render_pages(pdf_with_pages(&[0.0, 1.0]), 64).unwrap();
        let written = write_pages(&dir, &images).unwrap();

        let mut names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            ["notes.png", "slide-01.png", "slide-02.png", "slide-x.png"]
        );
        assert_eq!(
            written,
            [dir.join("slide-01.png"), dir.join("slide-02.png")]
        );
        assert_ne!(fs::read(dir.join("slide-01.png")).unwrap(), b"old");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_to_write_into_a_file() {
        let dir = scratch_dir("file");
        fs::write(&dir, b"").unwrap();
        assert!(write_pages(&dir, &[vec![0]]).is_err());
        fs::remove_file(&dir).unwrap();
    }

    #[test]
    fn pads_names_to_the_page_count() {
        let dir = scratch_dir("pad");
        let written = write_pages(&dir, &vec![vec![0u8]; 100]).unwrap();
        assert!(written[0].ends_with("slide-001.png"));
        assert!(written[99].ends_with("slide-100.png"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn recognises_only_its_own_file_names() {
        assert!(is_slide_image("slide-01.png"));
        assert!(is_slide_image("slide-100.png"));
        assert!(!is_slide_image("slide-.png"));
        assert!(!is_slide_image("slide-01.jpg"));
        assert!(!is_slide_image("slide-1a.png"));
        assert!(!is_slide_image("my-slide-01.png"));
    }
}
