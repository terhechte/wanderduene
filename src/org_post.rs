use dune_base::{DunePost, DunePostTime};

pub struct OrgPost {
    identifier: String,
    path: String,
    title: String,
    released: DunePostTime,
    contents: String,
    tags: Vec<String>,
    keywords: Vec<String>,
    description: String,
    enabled: bool
}

impl OrgPost {
    pub fn new(title: String, year: String, month: String, day: String) -> OrgPost {
        OrgPost {
            identifier: title.clone(),
            path: title.clone(),
            title: title.clone(),
            released: DunePostTime {
                year: year,
                month: month,
                day: day,
                values: (0, 0, 0),
                timestamp: 0
            },
            contents: title.clone(),
            tags: vec![title.clone()],
            keywords: Vec::new(),
            description: title.clone(),
            enabled: true
        }
    }
}

impl DunePost for OrgPost {
    fn identifier(&self) -> &str { &self.identifier }
    fn path(&self) -> &str { &self.path }
    fn title(&self) -> &str { &self.title }
    fn released(&self) -> &DunePostTime { &self.released }
    fn contents(&self) -> &str { &self.contents }
    fn tags(&self) -> &[String] { &self.tags }
    fn keywords(&self) -> &[String] { &self.keywords }
    fn description(&self) -> &str { &self.description }
    fn enabled(&self) -> bool { self.enabled }
}
