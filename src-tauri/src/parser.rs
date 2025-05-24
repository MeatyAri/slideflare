use serde::{Deserialize, Serialize};
use serde_json::json;
use pulldown_cmark::{Parser, Options, Event};
use gray_matter::{Matter, engine::YAML};
use std::fs;
use std::error::Error;
use regex::Regex;

// Define the output JSON structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    bgColor: String,
    textColor: String,
    title: String,
    content: String,
}

pub fn parse_markdown_with_frontmatter(content: &str) -> Result<Vec<Card>, Box<dyn Error>> {
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
        
        // Process the content part with Markdown and KaTeX support
        let content = process_markdown_with_katex(&result.content);
        
        cards.push(Card {
            bgColor: bg_color,
            textColor: text_color,
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
    // This is to handle cases where the content might start or end with "---"
    content.insert_str(0, "\n");
    content.insert_str(content.len(), "\n");
    
    // The pattern for a markdown document with frontmatter is:
    // 1. It starts with "---"
    // 2. Contains YAML until the next "---"
    // 3. After that, it's markdown content until the next "---" or EOF
    
    // Split the content by "---" lines
    let parts: Vec<&str> = content.split("\n---\n").collect();
    let mut sections = Vec::new();
    
    // Process each pair of parts as a section (YAML + content)
    let mut i = 0;
    while i < parts.len() {
        // First part should be YAML frontmatter (may be empty if starting with ---)
        let yaml = parts[i].trim();
        
        // If we're at the beginning and the first part isn't YAML content, skip it
        if i == 0 && !yaml.is_empty() && !yaml.contains(":") || yaml.is_empty() {
            i += 1;
            continue;
        }
        
        // Second part is markdown content
        if i + 1 < parts.len() {
            let markdown = parts[i + 1].trim();
            
            // Create a proper document with frontmatter for gray_matter to parse
            let section = format!("---\n{}\n---\n{}", yaml, markdown);
            sections.push(section);
            
            // Move to the next pair
            i += 2;
        } else {
            // If we have an odd number of parts, the last part might be markdown without frontmatter
            // We'll skip it as we're looking for sections with frontmatter
            break;
        }
    }
    
    // If we couldn't extract any valid sections, try to parse the whole file
    if sections.is_empty() && !content.trim().is_empty() {
        // Check if the content has a proper frontmatter structure
        if content.starts_with("---") && content.matches("---").count() >= 2 {
            sections.push(content.to_string());
        }
    }
    
    sections
}

// Process markdown content and handle KaTeX expressions
fn process_markdown_with_katex(content: &str) -> String {
    // Set up Markdown parser with all extensions enabled
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    
    // Parse markdown
    let parser = Parser::new_ext(content, options);
    
    // Convert to HTML first
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    
    // Process KaTeX expressions
    process_katex(&html_output)
}

// Simple KaTeX expression processor
// Note: For a production system, you'd want to use a dedicated KaTeX renderer
fn process_katex(content: &str) -> String {
    // Pattern for inline math: $...$
    let inline_re = Regex::new(r"\$([^\$]+)\$").unwrap();
    let content = inline_re.replace_all(content, "<span class=\"katex\">$1</span>");
    
    // Pattern for block math: $$...$$
    let block_re = Regex::new(r"\$\$([^\$]+)\$\$").unwrap();
    let content = block_re.replace_all(&content, "<div class=\"katex-block\">$1</div>");
    
    content.to_string()
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    
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

$f(x) = \int_{-\infty}^{\infty} \hat{f}(\xi) e^{2 \pi i \xi x} d\xi$
"#;

        let result = parse_markdown_with_frontmatter(input).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Introduction to JavaScript");
        assert_eq!(result[1].bgColor, "bg-green-500");
        assert_eq!(result[2].textColor, "text-white");
    }
}