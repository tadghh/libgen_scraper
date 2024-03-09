use crate::{
    book::LibgenBook,
    scraper::LibgenError,
    util::{build_direct_download_url, calculate_group_id},
};
use scraper::{ElementRef, Html, Selector};

/// A html processor to grab needed elements
pub struct Processor {
    /// CSS selector
    pub book_libgen_id_selector: Selector,
    /// CSS selector
    pub book_publisher_selector: Selector,
    /// CSS selector
    pub book_file_type_selector: Selector,
    /// CSS selector
    pub book_authors_selector: Selector,
    /// CSS selector
    pub book_search_result_selector: Selector,
}

impl Processor {
    /// Creates a new html processor with the needed css
    pub fn new() -> Self {
        Self {
            book_libgen_id_selector: Selector::parse("td:first-child").unwrap(),
            book_publisher_selector: Selector::parse("td:nth-child(4)").unwrap(),
            book_file_type_selector: Selector::parse("td:nth-child(9)").unwrap(),
            book_authors_selector: Selector::parse("td:nth-child(2) > a:not([title])").unwrap(),
            book_search_result_selector: Selector::parse("table.c tbody tr").unwrap(),
        }
    }

    /// Parses the html from a search result on libgen
    fn parse_search_result(&self, title: &str, result_row: ElementRef<'_>) -> Option<LibgenBook> {
        let book_id_elem = result_row.select(&self.book_libgen_id_selector).next()?;

        let libgen_id = book_id_elem.inner_html().parse::<u64>().ok()?;

        // CSS to grab the title of a search result
        let title_cell_selector =
            Selector::parse(&format!("td[width='500'] > a[id='{}']", libgen_id)).unwrap();

        let title_cell = result_row.select(&title_cell_selector).next()?;

        let search_result_title = title_cell.text().nth(0)?.trim();

        // If the search result title doesnt contain/match the title parameter return none. We know it isn't the correct book
        // If two books end up with the same title, whichever is processed first is returned
        // TODO: add advanced search
        let search_result_title_trimmed = search_result_title.trim();
        let title_trimmed = title.trim();
        if !search_result_title_trimmed
            .to_ascii_lowercase()
            .starts_with(&title_trimmed.to_ascii_lowercase())
        {
            return None;
        }
        // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
        let file_type: String = result_row
            .select(&self.book_file_type_selector)
            .next()
            .unwrap()
            .inner_html();

        let href_book_link: String = title_cell.value().attr("href")?.to_string();

        let authors: Vec<_> = result_row
            .select(&self.book_authors_selector)
            .into_iter()
            .map(|auth| auth.inner_html())
            .collect();

        let publisher = result_row
            .select(&self.book_publisher_selector)
            .next()
            .unwrap()
            .inner_html();

        let direct_link =
            build_direct_download_url(libgen_id, href_book_link, &title.to_string(), file_type)
                .ok();

        Some(LibgenBook {
            title: search_result_title.to_owned(),
            libgen_id,
            publisher,
            authors,
            direct_link,
        })
    }

    /// Looks for a books title in the html reponse
    pub fn search_title_in_document(
        &self,
        html_document: &Html,
        title: &str,
    ) -> Result<Option<LibgenBook>, LibgenError> {
        let book_data = html_document
            .select(&self.book_search_result_selector)
            .find_map(|srch_result| self.parse_search_result(title, srch_result));

        Ok(book_data)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn parse_result_existing_title() {
        // Existing as in its in the downloaded html file in /benches
        let client_processor = Processor::new();
        let existing_book_title = "Performance Evaluation and Benchmarking".to_string();

        let html_content = fs::read_to_string("benches/benchmark_page.htm").unwrap();

        let document = Html::parse_document(&html_content);
        let search_value =
            client_processor.search_title_in_document(&document, &existing_book_title);
        match search_value {
            Ok(val) => {
                //assert!(val.is_some())
                match val {
                    Some(book) => {
                        assert!(book.title == existing_book_title)
                    }
                    None => panic!("There was no book found"),
                }
            }
            Err(_) => panic!("Failed to process document"),
        }
    }

    #[test]
    fn parse_result_partial_existing_title() {
        // Existing, as in its located in the downloaded html file /benches
        let client_processor = Processor::new();
        let existing_book_title_partial = "and Benchmarking".to_string();

        let html_content = fs::read_to_string("benches/benchmark_page.htm").unwrap();

        let document = Html::parse_document(&html_content);
        let search_value =
            client_processor.search_title_in_document(&document, &existing_book_title_partial);
        match search_value {
            Ok(val) => {
                assert!(val.is_none());
            }
            Err(_) => panic!("Failed to process document"),
        }
    }
}
