use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abstraction {
    pub id: String,
    pub content: String,
    pub hierarchy_level: u32,
    pub children: Vec<Abstraction>,
    pub parent_id: Option<String>,
    pub compressed_size: usize,
    pub semantic_hash: String,
}

pub struct HierarchicalWormhole {
    cache: HashMap<String, Abstraction>,
    max_level: u32,
}

impl HierarchicalWormhole {
    pub fn new(max_level: u32) -> Self {
        Self {
            cache: HashMap::new(),
            max_level,
        }
    }

    pub fn compress(&mut self, text: &str) -> String {
        if let Some(cached) = self.cache.get(text) {
            return self.serialize_abstraction(cached);
        }

        let sentences: Vec<&str> = text.split('.').filter(|s| !s.is_empty()).collect();
        let root = self.build_abstraction(&sentences, 0);
        self.cache.insert(text.to_string(), root.clone());
        self.serialize_abstraction(&root)
    }

    fn build_abstraction(&self, segments: &[&str], level: u32) -> Abstraction {
        if level >= self.max_level || segments.len() <= 2 {
            let content = segments.join(". ");
            let truncated = if content.len() > 200 {
                format!("{}...", &content[..197])
            } else {
                content
            };
            Abstraction {
                id: format!("abs-{}-{}", level, segments.len()),
                content: truncated.clone(),
                hierarchy_level: level,
                children: Vec::new(),
                parent_id: None,
                compressed_size: truncated.len() / 4,
                semantic_hash: blake3::hash(truncated.as_bytes()).to_hex().to_string(),
            }
        } else {
            let half = segments.len() / 2;
            let left = self.build_abstraction(&segments[..half], level + 1);
            let right = self.build_abstraction(&segments[half..], level + 1);
            let content = format!("{} | {}", left.content, right.content);
            let truncated = if content.len() > 200 {
                format!("{}...", &content[..197])
            } else {
                content
            };
            Abstraction {
                id: format!("abs-{}-{}", level, segments.len()),
                content: truncated.clone(),
                hierarchy_level: level,
                children: vec![left, right],
                parent_id: None,
                compressed_size: truncated.len() / 4,
                semantic_hash: blake3::hash(truncated.as_bytes()).to_hex().to_string(),
            }
        }
    }

    fn serialize_abstraction(&self, abs: &Abstraction) -> String {
        if abs.children.is_empty() {
            format!("[{}]", abs.content)
        } else {
            let children_str: Vec<String> = abs.children.iter()
                .map(|c| self.serialize_abstraction(c))
                .collect();
            format!("({})", children_str.join(","))
        }
    }

    pub fn decompress(&self, compressed: &str) -> String {
        compressed.replace('(', "").replace(')', "").replace(',', ". ")
    }
}
