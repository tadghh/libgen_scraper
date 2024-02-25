#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
pub struct LibgenBook {
    pub libgen_id: u64,
    pub libgen_group_id: u64,
    pub title: String,
    pub authors: Vec<String>,
    pub publisher: String,
    pub direct_link: Option<String>,
}
