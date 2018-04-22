#[derive(Debug, Clone)]
pub struct DunePost {
    pub identifier: String,
    pub path: String,
    pub title: String,
    pub released: DunePostTime,
    pub contents: String,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub description: String,
    pub enabled: bool
}

#[derive(Debug, Clone)]
pub struct DunePostTime {
    pub year: String,
    pub month: String,
    pub day: String,
    pub values: (i32, i32, i32)
}

impl DunePostTime {
    pub fn timestamp(&self) -> i64 {
        let base_year: i32 = 2010;
        let year = self.values.0 - base_year;
        ((year * 12 * 31) + (self.values.1 * 31) + self.values.2) as i64
    }
}