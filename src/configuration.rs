use std::path::Path;

trait Configuration {
    fn html_folder(&self) -> &str;
    fn cache_file(&self) -> &Path;
}
