use std::collections::HashMap;

use csv_async::StringRecord;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ParseContext {
    pub tenant_id: String,
    pub import_id: Uuid,
    pub wallet: String,
}

#[derive(Debug, Clone)]
pub struct HeaderView {
    pub index: HashMap<String, usize>,
}

impl HeaderView {
    pub fn new(raw: &StringRecord, aliases: &HashMap<String, String>) -> Self {
        let mut index = HashMap::new();

        for (i, h) in raw.iter().enumerate() {
            let h = h.trim();
            index.insert(h.to_lowercase(), i);

            if let Some(canon) = aliases.get(h) {
                index.insert(canon.to_lowercase(), i);
            }
        }

        println!("{:#?}", index);

        Self {
            index,
        }
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        self.index.get(&name.to_lowercase()).copied()
    }

    pub fn has(&self, name: &str) -> bool {
        self.index.contains_key(&name.to_lowercase())
    }

    pub fn contains_all(&self, req: &[String]) -> bool {
        req.iter().all(|n| self.has(n))
    }
}
