use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HeaderView {
    index: HashMap<String, usize>,
}

impl HeaderView {
    pub fn new(raw_headers: &[String], aliases: &HashMap<&str, &str>) -> Self {
        let mut index = HashMap::new();
        for (i, h) in raw_headers.iter().enumerate() {
            let norm = h.trim().to_lowercase();
            let canon = aliases.get(norm.as_str()).copied().unwrap_or(norm.as_str()).to_string();
            index.entry(canon).or_insert(i);
        }
        Self {
            index,
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.index.contains_key(&name.to_lowercase())
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        self.index.get(&name.to_lowercase()).copied()
    }

    pub fn contains_all(&self, req: &[String]) -> bool {
        req.iter().all(|n| self.has(n))
    }
}
