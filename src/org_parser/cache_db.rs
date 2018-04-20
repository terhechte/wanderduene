use org_parser::rustbreak::Database;
use std::path::Path;

pub struct CacheDB {
    database: Database<String>
}

impl CacheDB {
    pub fn new<T: AsRef<Path>>(cache_file: T) -> CacheDB {
        let database = Database::<String>::open(cache_file.as_ref()).unwrap();
        CacheDB {
            database
        }
    }

    pub fn cache_entry(&self, identifier: &str) -> Option<String> {
        self.database.retrieve(identifier).ok()
    }

    pub fn set_cache_entry(&self, identifier: &str, contents: &str) {
        self.database.insert(identifier, contents.clone());
        self.database.flush().unwrap();
    }
}
