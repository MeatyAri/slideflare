use imara_diff::{Algorithm, Diff, InternedInput, TokenSource};
use serde::{Deserialize, Serialize};
use std::error::Error;
use twox_hash::XxHash32;

use super::parser::{parse_individual_slide, validate_slide_divider_syntax, ParseError, Slide};

// /// Metadata for tracking slide changes
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub struct SlideMetadata {
//     pub hash: u32,
//     pub index: usize,
// }

#[derive(Debug, PartialEq, Eq, Default)]
pub struct VecSlideHashes {
    pub data: Vec<u32>,
}

impl VecSlideHashes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_from(data: Vec<u32>) -> Self {
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
}

/// Compute hash for a single slide's raw content
/// Uses the same XxHash32 algorithm and seed as file hashing
pub fn compute_slide_hash(slide_content: &str) -> u32 {
    XxHash32::oneshot(42, slide_content.as_bytes())
}

/// Split content into sections with indices
/// Returns vector of (index, section_content) tuples
pub fn split_into_sections_with_indices(content: &str) -> Result<Vec<(usize, String)>, ParseError> {
    let sections = super::parser::split_into_sections(content)?;
    Ok(sections.into_iter().enumerate().collect())
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
    validate_slide_divider_syntax(content).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let sections = super::parser::split_into_sections(content)?;
    let mut hashes = Vec::new();

    for section in sections {
        let hash = compute_slide_hash(&section);
        hashes.push(hash);
    }

    Ok(VecSlideHashes::create_from(hashes))
}

pub fn detect_slide_changes(old_hashes: &VecSlideHashes, new_hashes: &VecSlideHashes) -> Diff {
    let input = InternedInput::new(old_hashes, new_hashes);
    Diff::compute(Algorithm::Histogram, &input)
}

/// Parse only the slides that changed and create change events
pub fn create_slide_change_events(
    last_slide_hashes: &VecSlideHashes,
    new_content: &str,
    changes: Diff,
    base_dir: &str,
) -> Result<Vec<SlideChangeType>, Box<dyn Error>> {
    validate_slide_divider_syntax(new_content).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let mut change_events = Vec::new();

    let new_sections = split_into_sections_with_indices(new_content)?;

    let mut index_adjustment: i32 = 0;
    for change in changes.hunks() {
        let before_len = change.before.len() as i32;
        for index in change.before {
            if let Some(old_hash) = last_slide_hashes.data.get(index as usize) {
                change_events.push(SlideChangeType::Removed {
                    index: (index as i32 + index_adjustment) as usize,
                    old_hash: *old_hash,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// A minimal two-slide document used in several tests.
    fn two_slide_doc() -> &'static str {
        r#"---
title: Slide 1
---
Content 1

---
title: Slide 2
---
Content 2"#
    }

    // Collect all hunk ranges from a Diff into (before_start, before_end, after_start, after_end).
    fn hunk_ranges(diff: &Diff) -> Vec<(u32, u32, u32, u32)> {
        diff.hunks()
            .map(|h| (h.before.start, h.before.end, h.after.start, h.after.end))
            .collect()
    }

    // ── compute_slide_hash ───────────────────────────────────────────────────

    #[test]
    fn test_slide_hash_stability() {
        let content = r#"---
bg_color: bg-blue-500
text_color: text-white
title: Test Slide
---
This is test content"#;

        let hash1 = compute_slide_hash(content);
        let hash2 = compute_slide_hash(content);

        assert_eq!(hash1, hash2, "Same content should produce the same hash");
    }

    #[test]
    fn test_slide_hash_uniqueness() {
        let content1 = r#"---
bg_color: bg-blue-500
---
Content 1"#;

        let content2 = r#"---
bg_color: bg-red-500
---
Content 2"#;

        assert_ne!(
            compute_slide_hash(content1),
            compute_slide_hash(content2),
            "Different content should produce different hashes"
        );
    }

    // ── compute_slide_hashes ─────────────────────────────────────────────────

    #[test]
    fn test_compute_slide_hashes_count_and_uniqueness() {
        let hashes = compute_slide_hashes(two_slide_doc()).unwrap();
        assert_eq!(hashes.data.len(), 2, "Should produce one hash per slide");
        assert_ne!(
            hashes.data[0], hashes.data[1],
            "Different slides should have different hashes"
        );
    }

    #[test]
    fn test_compute_slide_hashes_stability() {
        // Hashing the same document twice must yield identical vectors.
        let h1 = compute_slide_hashes(two_slide_doc()).unwrap();
        let h2 = compute_slide_hashes(two_slide_doc()).unwrap();
        assert_eq!(h1, h2, "compute_slide_hashes must be deterministic");
    }

    // ── detect_slide_changes ─────────────────────────────────────────────────

    #[test]
    fn test_detect_no_changes_when_identical() {
        let old = VecSlideHashes::create_from(vec![11, 22, 33]);
        let new = VecSlideHashes::create_from(vec![11, 22, 33]);

        let diff = detect_slide_changes(&old, &new);
        assert_eq!(
            diff.hunks().count(),
            0,
            "Identical hash sequences should produce zero hunks"
        );
    }

    #[test]
    fn test_detect_slide_appended() {
        // A new slide is appended at the end.
        let old = VecSlideHashes::create_from(vec![11, 22]);
        let new = VecSlideHashes::create_from(vec![11, 22, 33]);

        let diff = detect_slide_changes(&old, &new);
        let hunks = hunk_ranges(&diff);
        assert_eq!(hunks.len(), 1, "Should be exactly one hunk");
        let (bs, be, as_, ae) = hunks[0];
        assert_eq!(bs, be, "Pure insertion: before range should be empty");
        assert_eq!(ae - as_, 1, "Exactly one slide inserted");
    }

    #[test]
    fn test_detect_slide_prepended() {
        // A new slide is inserted at the front.
        let old = VecSlideHashes::create_from(vec![11, 22]);
        let new = VecSlideHashes::create_from(vec![99, 11, 22]);

        let diff = detect_slide_changes(&old, &new);
        let hunks = hunk_ranges(&diff);
        assert_eq!(hunks.len(), 1);
        let (bs, be, _as, ae) = hunks[0];
        assert_eq!(bs, be, "Pure insertion: before range should be empty");
        assert_eq!(ae - _as, 1);
    }

    #[test]
    fn test_detect_slide_removed_middle() {
        // Slide at index 1 is removed.
        let old = VecSlideHashes::create_from(vec![1, 2, 3]);
        let new = VecSlideHashes::create_from(vec![1, 3]);

        let diff = detect_slide_changes(&old, &new);
        let hunks = hunk_ranges(&diff);
        assert_eq!(hunks.len(), 1);
        let (bs, be, as_, ae) = hunks[0];
        assert_eq!(be - bs, 1, "Exactly one slide removed");
        assert_eq!(as_, ae, "Pure removal: after range should be empty");
    }

    #[test]
    fn test_detect_slide_replaced() {
        // Middle slide swapped for a different one (hash changes).
        let old = VecSlideHashes::create_from(vec![1, 2, 3]);
        let new = VecSlideHashes::create_from(vec![1, 99, 3]);

        let diff = detect_slide_changes(&old, &new);
        // Should see a replacement of slide at index 1.
        let total_before: u32 = diff.hunks().map(|h| h.before.len() as u32).sum();
        let total_after: u32 = diff.hunks().map(|h| h.after.len() as u32).sum();
        assert_eq!(total_before, 1, "One slide should be marked as removed");
        assert_eq!(total_after, 1, "One slide should be marked as added");
    }

    #[test]
    fn test_detect_all_slides_replaced() {
        let old = VecSlideHashes::create_from(vec![1, 2, 3]);
        let new = VecSlideHashes::create_from(vec![4, 5, 6]);

        let diff = detect_slide_changes(&old, &new);
        let total_before: u32 = diff.hunks().map(|h| h.before.len() as u32).sum();
        let total_after: u32 = diff.hunks().map(|h| h.after.len() as u32).sum();
        assert_eq!(total_before, 3);
        assert_eq!(total_after, 3);
    }

    // ── create_slide_change_events ───────────────────────────────────────────

    #[test]
    fn test_create_events_no_changes() {
        let hashes = compute_slide_hashes(two_slide_doc()).unwrap();
        let diff = detect_slide_changes(&hashes, &hashes);
        let events = create_slide_change_events(&hashes, two_slide_doc(), diff, "").unwrap();
        assert!(events.is_empty(), "No changes → no events");
    }

    #[test]
    fn test_create_events_slide_appended() {
        let old_content = two_slide_doc();
        let new_content = r#"---
title: Slide 1
---
Content 1

---
title: Slide 2
---
Content 2

---
title: Slide 3
---
Content 3"#;

        let old_hashes = compute_slide_hashes(old_content).unwrap();
        let new_hashes = compute_slide_hashes(new_content).unwrap();
        let diff = detect_slide_changes(&old_hashes, &new_hashes);
        let events = create_slide_change_events(&old_hashes, new_content, diff, "").unwrap();

        assert_eq!(events.len(), 1);
        assert!(
            matches!(events[0], SlideChangeType::Added { index: 2, .. }),
            "New slide should be Added at index 2"
        );
    }

    #[test]
    fn test_create_events_slide_removed() {
        let old_content = two_slide_doc();
        let new_content = r#"---
title: Slide 1
---
Content 1"#;

        let old_hashes = compute_slide_hashes(old_content).unwrap();
        let new_hashes = compute_slide_hashes(new_content).unwrap();
        let diff = detect_slide_changes(&old_hashes, &new_hashes);
        let events = create_slide_change_events(&old_hashes, new_content, diff, "").unwrap();

        assert_eq!(events.len(), 1);
        assert!(
            matches!(events[0], SlideChangeType::Removed { index: 1, .. }),
            "Removed slide should be Removed at index 1"
        );
    }

    #[test]
    fn test_create_events_slide_modified() {
        // Modifying a slide is represented as one Removed + one Added.
        let old_content = two_slide_doc();
        let new_content = r#"---
title: Slide 1
---
Content 1

---
title: Slide 2 MODIFIED
---
Updated content"#;

        let old_hashes = compute_slide_hashes(old_content).unwrap();
        let new_hashes = compute_slide_hashes(new_content).unwrap();
        let diff = detect_slide_changes(&old_hashes, &new_hashes);
        let events = create_slide_change_events(&old_hashes, new_content, diff, "").unwrap();

        let removed_count = events
            .iter()
            .filter(|e| matches!(e, SlideChangeType::Removed { .. }))
            .count();
        let added_count = events
            .iter()
            .filter(|e| matches!(e, SlideChangeType::Added { .. }))
            .count();

        assert_eq!(removed_count, 1, "Modified slide should emit one Removed");
        assert_eq!(added_count, 1, "Modified slide should emit one Added");
    }

    #[test]
    fn test_create_events_removed_old_hash_preserved() {
        // The Removed event must carry the correct old hash.
        let old_hashes = compute_slide_hashes(two_slide_doc()).unwrap();
        let expected_hash = old_hashes.data[1];

        let new_content = r#"---
title: Slide 1
---
Content 1"#;
        let new_hashes = compute_slide_hashes(new_content).unwrap();
        let diff = detect_slide_changes(&old_hashes, &new_hashes);
        let events = create_slide_change_events(&old_hashes, new_content, diff, "").unwrap();

        if let SlideChangeType::Removed { old_hash, .. } = &events[0] {
            assert_eq!(
                *old_hash, expected_hash,
                "Removed event must carry the old hash"
            );
        } else {
            panic!("Expected a Removed event");
        }
    }

    // ── multi-change sequence (simulates multiple consecutive file saves) ────

    #[test]
    fn test_incremental_pipeline_end_to_end() {
        // v1 → v2: slide 2 is modified + slide 3 is added.
        let v1 = two_slide_doc();
        let v2 = r#"---
title: Slide 1
---
Content 1

---
title: Modified Slide 2
---
Modified Content 2

---
title: New Slide 3
---
Content 3"#;

        let h1 = compute_slide_hashes(v1).unwrap();
        let h2 = compute_slide_hashes(v2).unwrap();
        let diff = detect_slide_changes(&h1, &h2);
        let events = create_slide_change_events(&h1, v2, diff, "").unwrap();

        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, SlideChangeType::Added { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, SlideChangeType::Removed { .. })));
    }

    #[test]
    fn test_unchanged_slides_produce_no_events() {
        // Slide 1 stays the same; only slide 2 changes.
        let old_content = two_slide_doc();
        let new_content = r#"---
title: Slide 1
---
Content 1

---
title: Slide 2 changed
---
Different content"#;

        let old_hashes = compute_slide_hashes(old_content).unwrap();
        let new_hashes = compute_slide_hashes(new_content).unwrap();

        // Slide 1 hash must be stable across versions.
        assert_eq!(
            old_hashes.data[0], new_hashes.data[0],
            "Slide 1 hash should be identical because its content did not change"
        );

        let diff = detect_slide_changes(&old_hashes, &new_hashes);
        let events = create_slide_change_events(&old_hashes, new_content, diff, "").unwrap();

        // No event should reference slide index 0.
        for event in &events {
            match event {
                SlideChangeType::Added { index, .. } => {
                    assert_ne!(
                        *index, 0,
                        "Slide 0 was not changed; no Added event expected"
                    )
                }
                SlideChangeType::Removed { index, .. } => {
                    assert_ne!(
                        *index, 0,
                        "Slide 0 was not changed; no Removed event expected"
                    )
                }
            }
        }
    }

    #[test]
    fn test_multiple_sequential_changes() {
        // Simulates three consecutive file-save cycles.
        //
        // save 1 → save 2: append slide 3
        // save 2 → save 3: remove slide 2, modify slide 1
        // After all three saves the test verifies each transition independently.

        let save1 = r#"---
title: Slide A
---
Alpha

---
title: Slide B
---
Beta"#;

        let save2 = r#"---
title: Slide A
---
Alpha

---
title: Slide B
---
Beta

---
title: Slide C
---
Gamma"#;

        let save3 = r#"---
title: Slide A UPDATED
---
Alpha updated

---
title: Slide C
---
Gamma"#;

        // ── transition 1 ────────────────────────────────────────────────────
        let h1 = compute_slide_hashes(save1).unwrap();
        let h2 = compute_slide_hashes(save2).unwrap();
        let diff12 = detect_slide_changes(&h1, &h2);
        let events12 = create_slide_change_events(&h1, save2, diff12, "").unwrap();

        assert_eq!(events12.len(), 1, "Only one slide added in transition 1");
        assert!(matches!(
            events12[0],
            SlideChangeType::Added { index: 2, .. }
        ));

        // ── transition 2 ────────────────────────────────────────────────────
        let h3 = compute_slide_hashes(save3).unwrap();
        let diff23 = detect_slide_changes(&h2, &h3);
        let events23 = create_slide_change_events(&h2, save3, diff23, "").unwrap();

        // Slide A changed (remove old + add new) and Slide B was removed.
        let removed: Vec<_> = events23
            .iter()
            .filter(|e| matches!(e, SlideChangeType::Removed { .. }))
            .collect();
        let added: Vec<_> = events23
            .iter()
            .filter(|e| matches!(e, SlideChangeType::Added { .. }))
            .collect();

        // Two slides removed (A and B), one slide added (updated A).
        assert_eq!(
            removed.len(),
            2,
            "Slide A (old) and Slide B should be removed"
        );
        assert_eq!(added.len(), 1, "Updated Slide A should be added");

        // ── hash stability across save1 → save3 for unchanged slide C ───────
        // Slide C appears at index 2 in save2 and index 1 in save3.
        assert_eq!(
            h2.data[2], h3.data[1],
            "Slide C content unchanged: hash must be the same across transitions"
        );
    }

    #[test]
    fn test_idempotent_reprocessing() {
        // Applying the same diff twice on unchanged content should produce
        // zero events the second time.
        let content = two_slide_doc();
        let hashes = compute_slide_hashes(content).unwrap();

        let diff = detect_slide_changes(&hashes, &hashes);
        let events = create_slide_change_events(&hashes, content, diff, "").unwrap();

        assert!(
            events.is_empty(),
            "Re-processing identical content must produce no events"
        );
    }

    #[test]
    fn test_broken_then_fixed_produces_events() {
        let v1 = two_slide_doc();
        let broken = "No dividers here";
        let v1_hashes = compute_slide_hashes(v1).unwrap();

        let diff_broken = detect_slide_changes(&v1_hashes, &VecSlideHashes::new());
        let changes = diff_broken.hunks().collect::<Vec<_>>();
        assert!(!changes.is_empty(), "Broken content should produce changes");

        let broken_hashes = compute_slide_hashes(broken).unwrap_err();
        assert!(broken_hashes.to_string().contains("No slide dividers"));

        let diff_fixed = detect_slide_changes(&VecSlideHashes::new(), &v1_hashes);
        let changes_fixed = diff_fixed.hunks().collect::<Vec<_>>();
        assert!(
            !changes_fixed.is_empty(),
            "Fixed content should produce changes"
        );
    }
}
