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

        let book_id = book_id_elem.inner_html().parse::<u64>().ok()?;

        // CSS to grab the title of a search result
        let title_cell_selector =
            Selector::parse(&format!("td[width='500'] > a[id='{}']", book_id)).unwrap();

        let title_cell = result_row.select(&title_cell_selector).next()?;

        let search_result_title = title_cell.text().nth(0)?;

        // If the search result title doesnt contain/match the title parameter return none. We know it isn't the correct book
        if !search_result_title
            .to_ascii_lowercase()
            .contains(&title.to_ascii_lowercase())
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

        // TODO: Add as impl
        let book_group_id = calculate_group_id(book_id);

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
            build_direct_download_url(book_id, href_book_link, &title.to_string(), file_type).ok();

        Some(LibgenBook {
            title: title.to_owned(),
            libgen_id: book_id,
            libgen_group_id: book_group_id,
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
