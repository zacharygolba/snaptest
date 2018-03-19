use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::sync::RwLock;

use bincode::{deserialize, serialize};

use Error;

type Data = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct Store {
    data: RwLock<Data>,
}

impl Store {
    pub fn new() -> Store {
        Default::default()
    }

    pub fn load() -> Result<Store, Error> {
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

    pub fn compare<F>(&self, key: &str, f: F) -> Result<(), Error>
    where
        F: FnOnce(&str) -> Result<(), Error>,
    {
        self.read_and_then(|data| match data.get(key) {
            Some(value) => Ok(f(value)?),
            None => Ok(()),
        })
    }

    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        self.read(|data| data.contains_key(key))
    }

    pub fn insert(&self, key: String, value: String) -> Result<Option<String>, Error> {
        self.write(|data| data.insert(key, value))
    }

    pub fn save(&self) -> Result<(), Error> {
        let mut file = find_or_create_snapfile()?;
        let mut bytes = self.read(serialize)??;

        Ok(file.write_all(&mut bytes)?)
    }

    fn read<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Data) -> T,
    {
        match self.data.read() {
            Ok(ref guard) => Ok(f(guard)),
            Err(_) => unimplemented!(),
        }
    }

    fn read_and_then<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Data) -> Result<T, Error>,
    {
        match self.data.read() {
            Ok(ref guard) => Ok(f(guard)?),
            Err(_) => unimplemented!(),
        }
    }

    fn write<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Data) -> T,
    {
        match self.data.write() {
            Ok(ref mut guard) => Ok(f(guard)),
            Err(_) => unimplemented!(),
        }
    }
}

fn find_or_create_snapfile() -> Result<File, Error> {
    let path = trail!("tests", ".snapfile");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(Error::from)
}
