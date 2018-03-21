use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::sync::RwLock;

use bincode::{deserialize, serialize};

type Data = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct Store {
    data: RwLock<Data>,
}

impl Store {
    pub fn new() -> Store {
        Default::default()
    }

    pub fn load() -> ::Result<Store> {
        let mut store = Store::new();
        let mut file = find_or_create_snapfile()?;

        if file.metadata()?.len() == 0 {
            store.save()?;
        } else {
            let mut bytes = Vec::new();

            file.read_to_end(&mut bytes)?;
            store.data = deserialize(&bytes)?;
        }

        Ok(store)
    }

    pub fn compare<D, F, T>(&self, key: &str, default: D, f: F) -> ::Result<T>
    where
        D: FnOnce() -> ::Result<T>,
        F: FnOnce(&str) -> ::Result<T>,
    {
        match self.read(|data| data.get(key).map(|value| f(value)))? {
            Some(result) => result,
            None => default(),
        }
    }

    pub fn insert(&self, key: String, value: String) -> ::Result<Option<String>> {
        self.write(|data| data.insert(key, value))
    }

    pub fn save(&self) -> ::Result<()> {
        let mut file = find_or_create_snapfile()?;
        let bytes = self.read(serialize)??;

        file.write_all(&bytes).map_err(|e| e.into())
    }

    fn read<F, T>(&self, f: F) -> ::Result<T>
    where
        F: FnOnce(&Data) -> T,
    {
        match self.data.read() {
            Ok(ref guard) => Ok(f(guard)),
            Err(e) => bail!("{}", e),
        }
    }

    fn write<F, T>(&self, f: F) -> ::Result<T>
    where
        F: FnOnce(&mut Data) -> T,
    {
        match self.data.write() {
            Ok(ref mut guard) => Ok(f(guard)),
            Err(e) => bail!("{}", e),
        }
    }
}

fn find_or_create_snapfile() -> ::Result<File> {
    let path = trail!("tests", ".snapfile");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| e.into())
}
