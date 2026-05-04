use crate::{Document, FrameData};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn reflow(doc: &mut Document) {
        let chains = Self::find_text_chains(doc);
        for chain in chains {
            Self::reflow_chain(doc, &chain);
        }
    }

    fn find_text_chains(doc: &Document) -> Vec<Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut chains = Vec::new();

        let all_text_frames: Vec<_> = doc
            .spreads
            .iter()
            .flat_map(|s| s.pages.iter())
            .flat_map(|p| p.frames.iter())
            .filter_map(|f| {
                if let FrameData::Text(tf) = &f.data {
                    Some((f.id.clone(), tf))
                } else {
                    None
                }
            })
            .collect();

        for (id, tf) in &all_text_frames {
            if visited.contains(id) {
                continue;
            }

            // Only start from the head of the chain
            if tf.prev_frame_id.is_none() {
                let mut current_chain = Vec::new();
                let mut current_id = Some(id.clone());

                while let Some(id) = current_id {
                    if visited.contains(&id) {
                        break; // Cycle detected
                    }
                    visited.insert(id.clone());
                    current_chain.push(id.clone());

                    // Find next frame ID
                    current_id = all_text_frames
                        .iter()
                        .find(|(fid, _)| fid == &id)
                        .and_then(|(_, tf)| tf.next_frame_id.clone());
                }
                chains.push(current_chain);
            }
        }

        chains
    }

    fn reflow_chain(doc: &mut Document, chain: &[String]) {
        if chain.is_empty() {
            return;
        }

        // 1. Collect all content
        let mut full_content = String::new();
        for id in chain {
            if let Some(content) = Self::get_frame_content(doc, id) {
                full_content.push_str(&content);
            }
        }

        // 2. Distribute content
        // For now, use a very simple heuristic:
        // capacity = width * height / (average_char_area)
        // average_char_area = 12pt * 7pt = 84 sq pt (rough estimate for 12pt font)
        let average_char_area = 60.0;

        let mut remaining_content = full_content.as_str();
        for (i, id) in chain.iter().enumerate() {
            let is_last = i == chain.len() - 1;

            let capacity = if is_last {
                remaining_content.len()
            } else {
                let (w, h) = Self::get_frame_dimensions(doc, id);
                let cap = (w * h / average_char_area) as usize;
                std::cmp::min(cap, remaining_content.len())
            };

            // Try to find a good split point (space) near capacity if possible
            let split_idx = if capacity >= remaining_content.len() {
                remaining_content.len()
            } else {
                // Look for the last space before capacity
                match remaining_content[..capacity].rfind(' ') {
                    Some(idx) if idx > capacity / 2 => idx + 1,
                    _ => capacity,
                }
            };

            let (current, rest) = remaining_content.split_at(split_idx);
            Self::set_frame_content(doc, id, current.to_string());
            remaining_content = rest;
        }
    }

    fn get_frame_content(doc: &Document, id: &str) -> Option<String> {
        for spread in &doc.spreads {
            for page in &spread.pages {
                for frame in &page.frames {
                    if frame.id == id {
                        if let FrameData::Text(tf) = &frame.data {
                            return Some(tf.content.clone());
                        }
                    }
                }
            }
        }
        None
    }

    fn set_frame_content(doc: &mut Document, id: &str, content: String) {
        for spread in &mut doc.spreads {
            for page in &mut spread.pages {
                for frame in &mut page.frames {
                    if frame.id == id {
                        if let FrameData::Text(tf) = &mut frame.data {
                            tf.content = content;
                            return;
                        }
                    }
                }
            }
        }
    }

    fn get_frame_dimensions(doc: &Document, id: &str) -> (f64, f64) {
        for spread in &doc.spreads {
            for page in &spread.pages {
                for frame in &page.frames {
                    if frame.id == id {
                        return (frame.width.0, frame.height.0);
                    }
                }
            }
        }
        (0.0, 0.0)
    }
}
