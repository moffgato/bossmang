
use {
    crate::error::Error,
    directories::UserDirs,
    rocksdb::{DB, Options},
    std::path::PathBuf,
};


pub struct Database {
    db: DB,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        let user_dirs = UserDirs::new().expect("Failed to get user directories");
        let home_dir = user_dirs.home_dir();
        let db_path = home_dir.join(".bossmang/db");

        std::fs::create_dir_all(&db_path)?;

        let mut opts = Options::default();
        opts.create_if_missing(true);


        let db = DB::open(&opts, db_path)?;

        Ok(Self { db })

    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<(), Error> {
        self.db.put(key.as_bytes(), value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        self.db.get(key.as_bytes())?
            .ok_or_else(|| Error::KeyNotFound(key.to_string()))
    }

    pub fn delete(&self, key: &str) -> Result<(), Error> {
        self.db.delete(key.as_bytes())?;
        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<String>, Error> {
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        let mut keys = Vec::new();

        for item in iter {
            let (key, _) = item?;
            if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                keys.push(key_str);
            }
        }

        Ok(keys)
    }
}


