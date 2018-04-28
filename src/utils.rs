use std::collections::HashSet;

use ammonia::Builder;

pub fn strip_tags(html: &str) -> String {
   Builder::default()
        .tags(HashSet::new())
        .clean(html)
        .to_string()
}

mod UtilTest {
    use super::strip_tags;
    #[test]
    fn test_strip_tags() {
        let input = "hey, <b>my name is <i>carl</i>";
        let input = strip_tags(input).to_string();
        assert_eq!(input, "hey, my name is carl");
    }
}
