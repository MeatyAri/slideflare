use base64::Engine;
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{CowStr, Event, Options, Parser as MarkdownParser};
use pulldown_latex::{mathml::push_mathml, Parser as LatexParser, Storage};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Debug, Serialize, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: Option<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {}: {}", line, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl Error for ParseError {}

use base64::engine::general_purpose::STANDARD;

// Define the output JSON structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Slide {
    bg_color: String,
    text_color: String,
    title: String,
    content: String,
}

#[derive(Deserialize, Debug)]
pub struct Frontmatter {
    bg_color: Option<String>,
    text_color: Option<String>,
    title: Option<String>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            bg_color: Some("".to_string()),
            text_color: Some("".to_string()),
            title: Some("Untitled".to_string()),
        }
    }
}

pub fn parse_markdown_with_frontmatter(
    content: &str,
    base_dir: &str,
) -> Result<Vec<Slide>, Box<dyn Error>> {
    validate_slide_divider_syntax(content)?;

    let matter = Matter::<YAML>::new();
    let sections = split_into_sections(content)?;
    let mut cards = Vec::new();

    for section in sections {
        // Extract frontmatter and content
        let result = matter.parse::<Frontmatter>(&section)?;

        // Convert the YAML data (Pod) to a proper map
        // let frontmatter = result.data.as_ref().unwrap_or(&Frontmatter::default());
        let frontmatter = match result.data.as_ref() {
            Some(data) => data,
            None => {
                // If there's no frontmatter, use default values
                &Frontmatter::default()
            }
        };

        // Process the content part with Markdown and LaTeX support
        let content = process_markdown_with_latex(&result.content, base_dir);

        let slide = Slide {
            content,
            bg_color: frontmatter.bg_color.clone().unwrap_or_default(),
            text_color: frontmatter.text_color.clone().unwrap_or_default(),
            title: frontmatter.title.clone().unwrap_or_default(),
        };
        // println!("Slide created: {:?}", slide);

        cards.push(slide);
    }

    Ok(cards)
}

/// Parse a single slide section (for incremental processing)
pub fn parse_individual_slide(section: &str, base_dir: &str) -> Result<Slide, Box<dyn Error>> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse::<Frontmatter>(section)?;

    // Convert the YAML data (Pod) to a proper map
    let frontmatter = match result.data.as_ref() {
        Some(data) => data,
        None => &Frontmatter::default(),
    };

    // Process the content part with Markdown and LaTeX support
    let content = process_markdown_with_latex(&result.content, base_dir);

    let slide = Slide {
        content,
        bg_color: frontmatter.bg_color.clone().unwrap_or_default(),
        text_color: frontmatter.text_color.clone().unwrap_or_default(),
        title: frontmatter.title.clone().unwrap_or_default(),
    };

    Ok(slide)
}

pub fn validate_slide_divider_syntax(content: &str) -> Result<(), ParseError> {
    let normalized = content.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let mut in_frontmatter = false;
    let mut open_divider_line: Option<usize> = None;
    let mut divider_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        if trimmed == "---" {
            divider_count += 1;

            if let Some(last_line) = open_divider_line {
                if i == last_line + 1 {
                    return Err(ParseError {
                        message: "Consecutive dividers '---' found. Each divider must separate frontmatter from content.".to_string(),
                        line: Some(line_num),
                    });
                }
            }

            open_divider_line = Some(i);

            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                in_frontmatter = false;
            }
        } else if in_frontmatter && !trimmed.is_empty() {
            if trimmed.contains("---") {
                return Err(ParseError {
                    message: "Invalid divider '---' found within frontmatter. Dividers must be on their own line.".to_string(),
                    line: Some(line_num),
                });
            }
        }
    }

    if divider_count % 2 != 0 {
        return Err(ParseError {
            message: "Unmatched divider '---'. Each opening '---' must have a closing '---'."
                .to_string(),
            line: None,
        });
    }

    if divider_count == 0 {
        return Err(ParseError {
            message: "No slide dividers '---' found. Slides must be separated with '---' dividers."
                .to_string(),
            line: None,
        });
    }

    if content.trim().contains("---") {
        let has_proper_format = divider_count >= 2;
        if !has_proper_format {
            return Err(ParseError {
                message: "Invalid divider '---' found without proper frontmatter format."
                    .to_string(),
                line: None,
            });
        }
    }

    Ok(())
}

// Split content into multiple sections, each containing frontmatter and markdown content
pub fn split_into_sections(content: &str) -> Result<Vec<String>, ParseError> {
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

    Ok(sections)
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
        let full_path = if src_path.starts_with('/') || src_path.starts_with("http") {
            src_path.to_string()
        } else {
            // Build full filesystem path relative to the base_dir
            format!("{}/{}", base_dir.trim_end_matches('/'), src_path).replace("//", "/")
        };

        // Attempt to read and embed as Base64; fall back to asset:// protocol on error
        let processed_src = match read_file_as_base64(&full_path) {
            Ok(data_url) => data_url,
            Err(_) => resolve_asset_path(full_path),
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
bg_color: bg-blue-500
text_color: text-white
title: Introduction to JavaScript
---
JavaScript is a versatile programming language used for web development.

---
bg_color: bg-green-500
text_color: text-white
title: Variables and Data Types
---
JavaScript supports various data types including strings, numbers, and objects.

The area of a circle is $A = \pi r^2$.

---
bg_color: bg-red-500
text_color: text-white
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
bg_color: bg-gray-200
text_color: text-black
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
bg_color: bg-gray-200
text_color: text-black
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

    #[test]
    fn test_validate_valid_slides() {
        let valid_slides = r#"---
title: Slide 1
---
Content 1

---
title: Slide 2
---
Content 2"#;
        assert!(validate_slide_divider_syntax(valid_slides).is_ok());
    }

    #[test]
    fn test_validate_no_dividers() {
        let no_dividers = r#"# My Slide

Some content without any dividers."#;
        let result = validate_slide_divider_syntax(no_dividers);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("No slide dividers"));
    }

    #[test]
    fn test_error_then_fix_recovery() {
        let broken = "No dividers here";
        let fixed = r#"---
title: Fixed
---
Content"#;

        let result_broken = validate_slide_divider_syntax(broken);
        assert!(result_broken.is_err());

        let result_fixed = validate_slide_divider_syntax(fixed);
        assert!(result_fixed.is_ok());
        let sections = split_into_sections(fixed).unwrap();
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn test_validate_single_slide() {
        let single_slide = r#"---
title: My Slide
---
Some content here."#;
        assert!(validate_slide_divider_syntax(single_slide).is_ok());
    }

    #[test]
    fn test_validate_consecutive_dividers() {
        let consecutive = "---\ntitle: Slide 1\n---\n---\ntitle: Slide 2";
        let result = validate_slide_divider_syntax(consecutive);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Consecutive dividers"));
    }

    #[test]
    fn test_validate_unmatched_divider() {
        let no_closing = r#"---
title: Slide 1
---
Content
---
title: Slide 2"#;
        let result = validate_slide_divider_syntax(no_closing);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unmatched"));
    }

    #[test]
    fn test_validate_trailing_divider() {
        let trailing = r#"---
title: Slide 1
---
Content 1

---"#;
        let result = validate_slide_divider_syntax(trailing);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unmatched") || err.message.contains("Trailing"));
    }

    #[test]
    fn test_validate_divider_in_content() {
        let divider_in_content = r#"---
title: Slide
---
Use --- to separate items"#;
        let result = validate_slide_divider_syntax(divider_in_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_divider_in_frontmatter() {
        let in_frontmatter = r#"---
title: Slide ---
key: value
---
Content"#;
        let result = validate_slide_divider_syntax(in_frontmatter);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("within frontmatter"));
    }

    #[test]
    fn test_split_into_sections_returns_result() {
        let content = r#"---
title: Slide 1
---
Content 1"#;
        let result = split_into_sections(content);
        assert!(result.is_ok());
        let sections = result.unwrap();
        assert_eq!(sections.len(), 1);
    }
}
