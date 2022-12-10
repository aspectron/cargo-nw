use std::collections::HashMap;
use async_std::path::{Path, PathBuf};

use crate::prelude::*;

#[derive(Debug)]
pub struct Tpl {
    map : HashMap<String,String>,
}

impl TryFrom<&[(&str,String)]> for Tpl {
    type Error = Error;
    fn try_from(value: &[(&str,String)]) -> Result<Self> {
        let map: HashMap<String,String> = value
            .into_iter()
            .map(|(k,v)|(k.to_string(),v.to_string()))
            .collect();
        Ok(Tpl { map })
        }
}

impl TryFrom<&[(&str,&PathBuf)]> for Tpl {
    type Error = Error;
    fn try_from(value: &[(&str,&PathBuf)]) -> Result<Self> {
        let map: HashMap<String,String> = value
            .into_iter()
            .map(|(k,v)|(k.to_string(),v.to_string_lossy().to_string()))
            .collect();
        Ok(Tpl { map })
    }
}

impl Tpl {
    pub fn new() -> Tpl {
        Tpl {
            map : HashMap::new(),
        }
    }

    pub fn transform(&self, text: &str) -> String {
        let mut text = text.to_string();
        for (k,v) in self.map.iter() {
            text = text.replace(k,v);
        }
        text
    }
}
