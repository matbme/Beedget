use std::path::Path;
use std::fs::{File, remove_file};
use std::io::prelude::*;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub trait DataObject {
    fn load_from_file(path: &Path) -> Result<Self> where
        Self: DeserializeOwned {

        let mut file = File::open(&path)?;
        let mut fc = String::new();

        file.read_to_string(&mut fc)?;

        let val: Self = serde_json::from_str(&fc)?;

        Ok(val)
    }

    fn filename(&self) -> String;

    fn save_to_file(&self, path: &Path) -> Result<()> where
        Self: Serialize {

        let filename = path.join(self.filename());
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(filename)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }

    fn delete_file(&self, base_path: &Path) -> Result<()> {
        remove_file(base_path.join(self.filename()))?;

        Ok(())
    }
}
