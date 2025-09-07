use base64::Engine;
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{CowStr, Event, Options, Parser as MarkdownParser};
use pulldown_latex::{mathml::push_mathml, Parser as LatexParser, Storage};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

use base64::engine::general_purpose::STANDARD;

// Define the output JSON structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    bg_color: String,
    text_color: String,
    title: String,
    content: String,
}

pub fn parse_markdown_with_frontmatter(
    content: &str,
    base_dir: &str,
) -> Result<Vec<Card>, Box<dyn Error>> {
    let matter = Matter::<YAML>::new();
    let sections = split_into_sections(content);
    let mut cards = Vec::new();

    for section in sections {
        // Extract frontmatter and content
        let result = matter.parse(&section);
        // Convert the YAML data (Pod) to a proper map
        let frontmatter = result.data.as_ref().unwrap();

        // Extract required fields from frontmatter with default values
        let bg_color = frontmatter["bgColor"]
            .as_string()
            .unwrap_or("bg-default".to_string());

        let text_color = frontmatter["textColor"]
            .as_string()
            .unwrap_or("text-default".to_string());

        let title = frontmatter["title"]
            .as_string()
            .unwrap_or("Untitled".to_string());

        // Process the content part with Markdown and LaTeX support
        let content = process_markdown_with_latex(&result.content, base_dir);

        cards.push(Card {
            bg_color,
            text_color,
            title,
            content,
        });
    }

    Ok(cards)
}

// Split content into multiple sections, each containing frontmatter and markdown content
fn split_into_sections(content: &str) -> Vec<String> {
    // First, normalize line endings to ensure consistent processing
    let mut content = content.replace("\r\n", "\n");

    // Add newlines at the beginning and end to ensure proper splitting
    content.insert(0, '\n');
    content.push('\n');

    // Split the content by "---" lines
    let parts: Vec<&str> = content.split("\n---\n").collect();
    let mut sections = Vec::new();
    let mut i = 0;

    while i < parts.len() {
        let yaml = parts[i].trim();

        // Skip non‑YAML leading part
        if i == 0 && (!yaml.is_empty() && !yaml.contains(':') || yaml.is_empty()) {
            i += 1;
            continue;
        }

        if i + 1 < parts.len() {
            let markdown = parts[i + 1].trim();
            let section = format!("---\n{}\n---\n{}", yaml, markdown);
            sections.push(section);
            i += 2;
        } else {
            break;
        }
    }

    // Fallback: whole file as a single section if none were extracted
    if sections.is_empty()
        && !content.trim().is_empty()
        && content.starts_with("---")
        && content.matches("---").count() >= 2
    {
        sections.push(content.to_string());
    }

    sections
}

// Process markdown content and handle KaTeX expressions
fn process_markdown_with_latex(content: &str, base_dir: &str) -> String {
    // Enable all desired Markdown extensions
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_MATH);

    // LaTeX processing setup
    let storage = Storage::new();
    let config = pulldown_latex::RenderConfig::default();
    let mut mathml_out = String::new();

    // Parse markdown and replace math events with MathML
    let parser = MarkdownParser::new_ext(content, options);
    let mapped = parser.map(|evt| match evt {
        Event::InlineMath(s) => {
            let latex_source = s.into_string();
            let latex_parser = LatexParser::new(&latex_source, &storage);
            mathml_out.clear();
            match push_mathml(&mut mathml_out, latex_parser, config) {
                Ok(_) => {
                    let wrapped = format!("<span class=\"math inline\">{}</span>", mathml_out);
                    Event::Html(CowStr::Boxed(wrapped.into_boxed_str()))
                }
                Err(e) => {
                    eprintln!("Error while rendering inline math: {}", e);
                    Event::Text(CowStr::Boxed(format!("Error: {}", e).into_boxed_str()))
                }
            }
        }
        Event::DisplayMath(s) => {
            let latex_source = s.into_string();
            let latex_parser = LatexParser::new(&latex_source, &storage);
            mathml_out.clear();
            match push_mathml(&mut mathml_out, latex_parser, config) {
                Ok(_) => {
                    let wrapped = format!("<div class=\"math display\">{}</div>", mathml_out);
                    Event::Html(CowStr::Boxed(wrapped.into_boxed_str()))
                }
                Err(e) => {
                    eprintln!("Error while rendering display math: {}", e);
                    Event::Text(CowStr::Boxed(format!("Error: {}", e).into_boxed_str()))
                }
            }
        }
        other => other,
    });

    // Convert the processed events to HTML
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, mapped);

    // Post‑process HTML to embed local assets as Base64 data URLs
    post_process_asset_paths(&html_output, base_dir)
}

// Convert a filesystem path to a data‑URL compatible path (used as a fallback)
fn resolve_asset_path(path: String) -> String {
    let normalized_path = path.replace('\\', "/");
    format!("asset://localhost/{}", normalized_path)
}

// Post‑process HTML to replace local image/video sources with Base64 data URLs
fn post_process_asset_paths(html: &str, base_dir: &str) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Helper closure to replace a source attribute
    let replace_src = |tag: &str, caps: &regex::Captures| -> String {
        let before_src = &caps[1];
        let src_path = &caps[2];
        let after_src = &caps[3];

        // Determine whether the src is a remote or absolute path
        let processed_src = if src_path.starts_with('/') || src_path.starts_with("http") {
            src_path.to_string()
        } else {
            // Build full filesystem path relative to the base_dir
            let full_path =
                format!("{}/{}", base_dir.trim_end_matches('/'), src_path).replace("//", "/");
            // Attempt to read and embed as Base64; fall back to asset:// protocol on error
            match read_file_as_base64(&full_path) {
                Ok(data_url) => data_url,
                Err(_) => resolve_asset_path(full_path),
            }
        };

        format!(
            "{}{}src=\"{}\" data-asset-path=\"{}\" data-base-dir=\"{}\"{}>",
            tag, before_src, processed_src, src_path, base_dir, after_src
        )
    };

    // img tags
    let img_regex = Regex::new(r#"<img([^>]+)src="([^"]+)"([^>]*)>"#).unwrap();
    result = img_regex
        .replace_all(&result, |caps: &regex::Captures| replace_src("<img", caps))
        .to_string();

    // video tags
    let video_regex = Regex::new(r#"<video([^>]+)src="([^"]+)"([^>]*)>"#).unwrap();
    result = video_regex
        .replace_all(&result, |caps: &regex::Captures| {
            replace_src("<video", caps)
        })
        .to_string();

    // source tags (e.g., within <video> or <audio>)
    let source_regex = Regex::new(r#"<source([^>]+)src="([^"]+)"([^>]*)>"#).unwrap();
    result = source_regex
        .replace_all(&result, |caps: &regex::Captures| {
            replace_src("<source", caps)
        })
        .to_string();

    result
}

// Read a file from the filesystem and convert it to a Base64 data URL
fn read_file_as_base64(file_path: &str) -> Result<String, Box<dyn Error>> {
    let bytes = fs::read(file_path)?;
    let mime_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    let encoded = STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime_type, encoded))
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use std::fs;

    #[test]
    fn test_parsing() {
        let input = r#"---
bgColor: bg-blue-500
textColor: text-white
title: Introduction to JavaScript
---
JavaScript is a versatile programming language used for web development.

---
bgColor: bg-green-500
textColor: text-white
title: Variables and Data Types
---
JavaScript supports various data types including strings, numbers, and objects.

The area of a circle is $A = \pi r^2$.

---
bgColor: bg-red-500
textColor: text-white
title: Functions
---
Functions are reusable blocks of code that perform a specific task.

$f(x) = \int_{-\infty}^{\infty} \hat{f}(\\xi) e^{2 \\pi i \\xi x} d\\xi$
"#;

        let result = parse_markdown_with_frontmatter(input, "/test/base").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Introduction to JavaScript");
        assert_eq!(result[1].bg_color, "bg-green-500");
        assert_eq!(result[2].text_color, "text-white");

        // Verify that LaTeX is processed
        assert!(result[2]
            .content
            .contains("<span class=\"math inline\"><math display=\"inline\">"));
    }
    #[test]
    fn test_image_base64_embedding() {
        // Path to the example image relative to the crate root
        let base_dir = env!("CARGO_MANIFEST_DIR");
        let img_rel_path = "../examples/images/sample-image.png";

        let markdown = format!(
            r#"---
bgColor: bg-gray-200
textColor: text-black
title: Image Test
---
![]({})"#,
            img_rel_path
        );

        let result = parse_markdown_with_frontmatter(&markdown, base_dir).unwrap();
        assert_eq!(result.len(), 1);
        let content = &result[0].content;

        // Build the expected data URL
        let img_path = format!("{}/{}", base_dir, img_rel_path);
        let img_bytes = fs::read(&img_path).expect("Unable to read test image");
        let expected_data_url = format!("data:image/png;base64,{}", STANDARD.encode(&img_bytes));

        // Verify the base64 data URL is present
        assert!(
            content.contains(&expected_data_url),
            "Image should be embedded as base64 data URL"
        );

        // Verify the <img> tag is properly formatted (e.g., <img src="data:..." />)
        let img_regex = regex::Regex::new(
            r#"<img\s+[^>]*src="data:image/png;base64,[A-Za-z0-9+/=]+"\s*([^>]*?)\/?>"#,
        )
        .unwrap();
        assert!(
            img_regex.is_match(content),
            "Image tag is not properly formatted"
        );
    }

    #[test]
    fn test_video_base64_embedding() {
        // Path to the example video relative to the crate root
        let base_dir = env!("CARGO_MANIFEST_DIR");
        let video_rel_path = "../examples/videos/sample-video.mp4";

        let markdown = format!(
            r#"---
bgColor: bg-gray-200
textColor: text-black
title: Video Test
---
<video src="{}" controls></video>"#,
            video_rel_path
        );

        let result = parse_markdown_with_frontmatter(&markdown, base_dir).unwrap();
        assert_eq!(result.len(), 1);
        let content = &result[0].content;

        // Build the expected data URL
        let video_path = format!("{}/{}", base_dir, video_rel_path);
        let video_bytes = fs::read(&video_path).expect("Unable to read test video");
        let expected_data_url = format!("data:video/mp4;base64,{}", STANDARD.encode(&video_bytes));

        // Verify the base64 data URL is present
        assert!(
            content.contains(&expected_data_url),
            "Video should be embedded as base64 data URL"
        );

        // Verify the <video> tag is properly formatted with a base64 src attribute
        let video_regex = regex::Regex::new(
            r#"<video\s+[^>]*src="data:video/mp4;base64,[A-Za-z0-9+/=]+"\s*([^>]*?)>(?s).*?</video>"#
        )
        .unwrap();
        assert!(
            video_regex.is_match(content),
            "Video tag is not properly formatted"
        );
    }
}
