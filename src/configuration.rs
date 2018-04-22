use std::path::Path;

pub trait Configuration {
    fn html_folder(&self) -> &str;
    fn post_folder(&self) -> &Path;
    fn cache_file(&self) -> &Path;
}
