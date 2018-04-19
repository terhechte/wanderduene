use rustbreak::Database;
use configuration::Configuration;

struct CacheDB {
    database: Database<String>
}

impl CacheDB {
    pub fn new(configuration: &Configuration) -> CacheDB {
        let database = Database::<String>::open("rustest.cache").unwrap();
        CacheDB {
            database
        }
    }

    pub fn cache_entry(&self, identifier: &str) -> Option<String> {
        database.retrieve(identifier)
    }

    pub fn set_cache_entry(&self, identifier: &str, contents: &str) {
        self.database.insert(identifier, contents_html.clone());
        self.database.flush().unwrap();
    }
}
