use imara_diff::{Algorithm, Diff, InternedInput, TokenSource};
use serde::{Deserialize, Serialize};
use std::error::Error;
use twox_hash::XxHash32;

use super::parser::{parse_individual_slide, Slide};

/// Metadata for tracking slide changes
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SlideMetadata {
    pub hash: u32,
    pub index: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VecSlideHashes {
    data: Vec<u32>,
}

impl VecSlideHashes {
    pub fn new(data: Vec<u32>) -> Self {
        VecSlideHashes { data }
    }
}

impl TokenSource for &VecSlideHashes {
    type Token = u32; // Each element in the vector is a token
    type Tokenizer = std::vec::IntoIter<Self::Token>;

    fn tokenize(&self) -> Self::Tokenizer {
        self.data.clone().into_iter()
    }

    fn estimate_tokens(&self) -> u32 {
        self.data.len() as u32
    }
}

/// Types of slide changes
#[derive(Serialize, Deserialize, Debug)]
pub enum SlideChangeType {
    Added { index: usize, slide: Slide },
    Removed { index: usize, old_hash: u32 },
}

/// Event representing slide-level changes
#[derive(Serialize, Deserialize, Debug)]
pub struct SlideChangeEvent {
    pub changes: Vec<SlideChangeType>,
    pub file_hash: u64,
}

/// Compute hash for a single slide's raw content
/// Uses the same XxHash32 algorithm and seed as file hashing
pub fn compute_slide_hash(slide_content: &str) -> u32 {
    XxHash32::oneshot(42, slide_content.as_bytes())
}

/// Split content into sections with indices
/// Returns vector of (index, section_content) tuples
pub fn split_into_sections_with_indices(content: &str) -> Vec<(usize, String)> {
    let sections = super::parser::split_into_sections(content);
    sections.into_iter().enumerate().collect()
}

// /// Compute metadata (hash + index) for all slides in content
// pub fn compute_slide_metadata(content: &str) -> Result<Vec<SlideMetadata>, Box<dyn Error>> {
//     let sections = split_into_sections_with_indices(content);
//     let mut metadata = Vec::new();

//     for (index, section) in sections {
//         let hash = compute_slide_hash(&section);
//         metadata.push(SlideMetadata { hash, index });
//     }

//     Ok(metadata)
// }

/// Compute hash for all slides in content
pub fn compute_slide_hashes(content: &str) -> Result<VecSlideHashes, Box<dyn Error>> {
    let sections = super::parser::split_into_sections(content);
    let mut hashes = Vec::new();

    for section in sections {
        let hash = compute_slide_hash(&section);
        hashes.push(hash);
    }

    Ok(VecSlideHashes::new(hashes))
}

pub fn detect_slide_changes(old_hashes: &VecSlideHashes, new_hashes: &VecSlideHashes) -> Diff {
    let input = InternedInput::new(old_hashes, new_hashes);
    Diff::compute(Algorithm::Histogram, &input)
}

/// Parse only the slides that changed and create change events
pub fn create_slide_change_events(
    old_content: &str,
    new_content: &str,
    changes: Diff,
    base_dir: &str,
) -> Result<Vec<SlideChangeType>, Box<dyn Error>> {
    let mut change_events = Vec::new();

    let old_sections = split_into_sections_with_indices(old_content);
    let new_sections = split_into_sections_with_indices(new_content);

    let mut index_adjustment: i32 = 0;
    for change in changes.hunks() {
        let before_len = change.before.len() as i32;
        for index in change.before {
            if let Some((idx, section)) = old_sections.get(index as usize) {
                let old_hash = compute_slide_hash(section);
                change_events.push(SlideChangeType::Removed {
                    index: (*idx as i32 + index_adjustment) as usize,
                    old_hash,
                });
            }
        }
        index_adjustment += change.after.len() as i32 - before_len;
        for index in change.after {
            let index = index as usize;
            if let Some((_, section)) = new_sections.get(index) {
                let slide = parse_individual_slide(section, base_dir)?;
                change_events.push(SlideChangeType::Added { index, slide });
            }
        }
    }

    Ok(change_events)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_slide_hash_stability() {
//         let content = r#"---
// bg_color: bg-blue-500
// text_color: text-white
// title: Test Slide
// ---
// This is test content"#;

//         let hash1 = compute_slide_hash(content);
//         let hash2 = compute_slide_hash(content);

//         assert_eq!(hash1, hash2, "Same content should produce same hash");
//     }

//     #[test]
//     fn test_slide_hash_uniqueness() {
//         let content1 = r#"---
// bg_color: bg-blue-500
// ---
// Content 1"#;

//         let content2 = r#"---
// bg_color: bg-red-500
// ---
// Content 2"#;

//         let hash1 = compute_slide_hash(content1);
//         let hash2 = compute_slide_hash(content2);

//         assert_ne!(
//             hash1, hash2,
//             "Different content should produce different hashes"
//         );
//     }

//     #[test]
//     fn test_compute_slide_metadata() {
//         let content = r#"---
// title: Slide 1
// ---
// Content 1

// ---
// title: Slide 2
// ---
// Content 2"#;

//         let metadata = compute_slide_metadata(content).unwrap();
//         assert_eq!(metadata.len(), 2);
//         assert_eq!(metadata[0].index, 0);
//         assert_eq!(metadata[1].index, 1);
//         assert_ne!(metadata[0].hash, metadata[1].hash);
//     }

//     #[test]
//     fn test_detect_slide_changes_added() {
//         let old_hashes = VecSlideHashes::new(vec![11, 22]);
//         let new_hashes = VecSlideHashes::new(vec![22, 11]);

//         let diff = detect_slide_changes(&old_hashes, &new_hashes);
//         let changes = diff.hunks();

//         for change in changes {
//             println!("{:?}", change);
//             println!("pure insertion: {:?}", change.is_pure_insertion());
//             println!("pure removal: {:?}", change.is_pure_removal());
//         }

//         // assert_eq!(changes.collect::<Vec<Hunk>>().len(), 1);
//         // match &changes.next() {
//         //     SlideChange::Added(index) => assert_eq!(*index, 2),
//         //     _ => panic!("Expected Added change"),
//         // }
//     }

//     #[test]
//     fn test_detect_slide_changes_removed() {
//         let old_hashes = vec![1, 2, 3];
//         let new_hashes = vec![1, 3];

//         let changes = detect_slide_changes(&old_hashes, &new_hashes);

//         assert_eq!(changes.len(), 1);
//         match &changes[0] {
//             SlideChange::Removed(index) => assert_eq!(*index, 1),
//             _ => panic!("Expected Removed change"),
//         }
//     }

//     #[test]
//     fn test_detect_slide_changes_complex() {
//         let old_hashes = vec![1, 2, 3];
//         let new_hashes = vec![2, 4, 3, 5];

//         let changes = detect_slide_changes(&old_hashes, &new_hashes);

//         // Debug: print what we actually get
//         println!("Changes detected: {:?}", changes);

//         // With our simple algorithm:
//         // - slide 0 (hash 1) is removed
//         // - slide 1 becomes slide 0 (hash 2) - same hash, but moved
//         // - slide 2 (hash 3) stays slide 2 - same hash
//         // - slide with hash 4 is added at position 1
//         // - slide with hash 5 is added at position 3

//         // Our current logic:
//         // Common prefix: empty (since 1 != 2)
//         // Common suffix: slide with hash 3 at end
//         // So we get: removed slide 0, added slide 1, added slide 3

//         assert_eq!(changes.len(), 3);

//         // Check that slide 0 was removed
//         assert!(changes
//             .iter()
//             .any(|c| matches!(c, SlideChange::Removed(idx) if *idx == 0)));

//         // Check that we have additions
//         assert!(changes.iter().any(|c| matches!(c, SlideChange::Added(_))));
//     }

//     #[test]
//     fn test_incremental_pipeline_end_to_end() {
//         // Simulate complete incremental processing pipeline
//         let old_content = r#"---
// title: Slide 1
// ---
// Content 1

// ---
// title: Slide 2
// ---
// Content 2"#;

//         let new_content = r#"---
// title: Slide 1
// ---
// Content 1

// ---
// title: Modified Slide 2
// ---
// Modified Content 2

// ---
// title: New Slide 3
// ---
// Content 3"#;

//         // 1. Compute old and new metadata
//         let old_metadata = compute_slide_metadata(old_content).unwrap();
//         let new_metadata = compute_slide_metadata(new_content).unwrap();

//         let old_hashes: Vec<u32> = old_metadata.iter().map(|m| m.hash).collect();
//         let new_hashes: Vec<u32> = new_metadata.iter().map(|m| m.hash).collect();

//         // 2. Detect changes
//         let changes = detect_slide_changes(&old_hashes, &new_hashes);

//         // Should detect modifications and additions
//         assert!(!changes.is_empty());

//         // 3. Create change events
//         let change_events =
//             create_slide_change_events(old_content, new_content, &changes, "/test/base").unwrap();

//         // Should have change events
//         assert!(!change_events.is_empty());

//         // Verify events contain expected change types
//         assert!(change_events
//             .iter()
//             .any(|e| matches!(e, SlideChangeType::Added { .. })
//                 || matches!(e, SlideChangeType::Modified { .. })));
//     }

//     #[test]
//     fn test_incremental_state_consistency() {
//         // Test that applying changes produces same result as full reload
//         let initial_content = r#"---
// title: Slide 1
// ---
// Content 1"#;

//         let updated_content = r#"---
// title: Slide 1
// ---
// Content 1

// ---
// title: Slide 2
// ---
// Content 2"#;

//         // 1. Get initial metadata
//         let initial_metadata = compute_slide_metadata(initial_content).unwrap();
//         let initial_hashes: Vec<u32> = initial_metadata.iter().map(|m| m.hash).collect();

//         // 2. Get updated metadata
//         let updated_metadata = compute_slide_metadata(updated_content).unwrap();
//         let updated_hashes: Vec<u32> = updated_metadata.iter().map(|m| m.hash).collect();

//         // 3. Detect changes
//         let _changes = detect_slide_changes(&initial_hashes, &updated_hashes);

//         // 4. Verify slide stability
//         let min_len = initial_hashes.len().min(updated_hashes.len());
//         let mut common_prefix_len = 0;
//         for i in 0..min_len {
//             if initial_hashes[i] == updated_hashes[i] {
//                 common_prefix_len += 1;
//             } else {
//                 break;
//             }
//         }

//         assert_eq!(common_prefix_len, 1, "First slide should be unchanged");
//     }
// }
