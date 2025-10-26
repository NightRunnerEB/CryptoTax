use std::collections::HashMap;

use csv_async::StringRecord;

#[derive(Debug, Clone)]
pub struct HeaderView {
    index: HashMap<String, usize>,
}

impl HeaderView {
    pub fn new(raw: &StringRecord, aliases: &HashMap<String, String>) -> Self {
        let mut index = HashMap::new();

        for (i, h) in raw.iter().enumerate() {
            let h = h.trim();
            index.insert(h.to_string(), i);

            let canon = aliases.get(h).map(String::as_str).unwrap_or(h);
            index.insert(canon.to_string(), i);
        }

        Self { index }
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
