pub mod mexc;
pub mod okx;

use std::collections::HashMap;

use serde::Deserialize;

use crate::domain::services::ParserFactory;
fn one_char<'de, D>(d: D) -> Result<char, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(d)?;
    let mut it = s.chars();
    let c = it.next().ok_or_else(|| serde::de::Error::custom("empty delimiter"))?;
    if it.next().is_some() {
        return Err(serde::de::Error::custom("delimiter must be 1 char"));
    }
    Ok(c)
}

#[derive(Deserialize)]
pub struct ExchangeCfg {
    #[serde(deserialize_with = "one_char")]
    delimiter: char,
    aliases: HashMap<String, String>,
    factories: Vec<Box<dyn ParserFactory>>,
}
