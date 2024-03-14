use urlencoding::encode;

use crate::util::calculate_group_id;

#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
pub struct LibgenBook {
    /// The books id on libgen
    pub libgen_id: u64,
    /// Books title
    pub title: String,
    /// Authors who made the book
    pub authors: Vec<String>,
    /// The publisher of the book (some books have multiple which is not supported)
    pub publisher: String,
    /// The direct download link for the book
    pub libgen_md5: String,
    /// File type
    pub file_type: String,
}

impl LibgenBook {
    #[doc = r"Build the books download link."]
    pub fn build_direct_download_url(&self) -> Result<String, String> {
        // TODO: URL hardcoding?
        Ok(format!(
            "https://download.library.lol/main/{}/{}/{}.{}",
            calculate_group_id(self.libgen_id),
            self.libgen_md5,
            encode(&self.title),
            self.file_type
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::LibgenBook;

    #[test]
    fn build_direct_download_url() {
        let valid_cat_result = LibgenBook {
            libgen_id: 3750,
            libgen_md5: "5fa82be26689a4e6f4415ea068d35a9d".to_owned(),
            file_type: "pdf".to_owned(),
            title: "Abstract and concrete categories: the joy of cats".to_owned(),
            authors: vec![
                "Jiri Adamek".to_string(),
                " Horst Herrlich".to_string(),
                " George E. Strecker".to_string(),
            ],
            publisher: "Wiley-Interscience".to_owned(),
        };

        let valid_download_link = "https://download.library.lol/main/3000/5fa82be26689a4e6f4415ea068d35a9d/Abstract%20and%20concrete%20categories%3A%20the%20joy%20of%20cats.pdf";

        let download_link = valid_cat_result.build_direct_download_url();
        assert_eq!(valid_download_link, download_link.unwrap());
    }
}
