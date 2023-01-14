use async_std::path::PathBuf;
use std::collections::HashMap;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Tpl {
    pub map: HashMap<String, String>,
}

impl TryFrom<&[(&str, String)]> for Tpl {
    type Error = Error;
    fn try_from(value: &[(&str, String)]) -> Result<Self> {
        let map: HashMap<String, String> = value
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Ok(Tpl { map })
    }
}

impl TryFrom<&[(&str, &PathBuf)]> for Tpl {
    type Error = Error;
    fn try_from(value: &[(&str, &PathBuf)]) -> Result<Self> {
        let map: HashMap<String, String> = value
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string_lossy().to_string()))
            .collect();
        Ok(Tpl { map })
    }
}

impl Tpl {
    pub fn new() -> Tpl {
        let mut map = HashMap::new();
        for (k, v) in std::env::vars() {
            map.insert(format!("${}", k.to_uppercase()), v.to_string());
        }

        Tpl { map }
    }

    pub fn set(&mut self, kv: &[(&str, &str)]) {
        for (k, v) in kv {
            self.map.insert(k.to_string(), v.to_string());
        }
    }

    pub fn extend(&self, tpl: &Tpl) -> Tpl {
        Tpl {
            map: self
                .map
                .clone()
                .into_iter()
                .chain(tpl.map.clone().into_iter())
                .collect(),
        }
    }

    pub fn transform(&self, text: &str) -> String {
        let mut text = text.to_string();
        for (k, v) in self.map.iter() {
            text = text.replace(k, v);
        }
        text
    }
}
