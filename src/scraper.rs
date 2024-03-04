use std::{fmt, thread, time::Duration};

use reqwest::StatusCode;
use scraper::{ElementRef, Html, Selector};
use urlencoding::encode;

use crate::{
    book::LibgenBook,
    util::{calculate_group_id, parse_md5_from_url},
};

#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
pub enum LibgenError {
    ConnectionError,
    TimeoutError,
    NotFoundError,
    NetworkError,
    ParsingError,
}

impl fmt::Display for LibgenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_str = match self {
            LibgenError::ConnectionError => "ConnectionError",
            LibgenError::TimeoutError => "TimeoutError",
            LibgenError::NotFoundError => "NotFoundError",
            LibgenError::NetworkError => "NetworkError",
            LibgenError::ParsingError => "ParsingError",
        };
        write!(f, "{}", error_str)
    }
}

// TODO: should accept mirrors
fn build_direct_download_url(
    book_id: u64,
    url: String,
    title: &String,
    file_type: String,
) -> Result<String, String> {
    if let Some(md5_value) = parse_md5_from_url(url) {
        Ok(format!(
            "https://download.library.lol/main/{}/{}/{}.{}",
            calculate_group_id(book_id),
            md5_value,
            encode(title),
            file_type
        ))
    } else {
        Err("No 'md5' parameter found in the URL".to_string())
    }
}
// Private function to process a libgen search result
fn process_libgen_search_result(title: &String, result_row: ElementRef<'_>) -> Option<LibgenBook> {
    // CSS Selectors
    // Books libgen id
    let book_libgen_id_selector = Selector::parse("td:first-child").unwrap();

    let book_id_elem = result_row.select(&book_libgen_id_selector).next()?;
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
        println!("No title");
        println!("{:?}", search_result_title);
        //the result does not contain the given example/shortened title (comparing to method parameter this is the books title)
        return None;
    }

    // Books publisher selector
    let publisher_selector: Selector = Selector::parse("td:nth-child(4)").unwrap();

    // Book file type
    let file_type_selector = Selector::parse("td:nth-child(9)").unwrap();

    // Get all the authors for the book
    let authors_selector = Selector::parse("td:nth-child(2) > a:not([title])").unwrap();

    // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
    let file_type = result_row
        .select(&file_type_selector)
        .next()
        .unwrap()
        .inner_html()
        .to_owned();

    let href_book_link = title_cell.value().attr("href")?.to_owned();

    let book_group_id = calculate_group_id(book_id);

    let authors: Vec<_> = result_row
        .select(&authors_selector)
        .into_iter()
        .map(|auth| auth.inner_html())
        .collect();

    let publisher = result_row
        .select(&publisher_selector)
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

// TODO: Clean
const MAX_RETRIES: usize = 3;
const TIMEOUT_DURATION: u64 = 15;
const LIBGEN_MIRRORS: [&str; 3] = ["is", "rs", "st"];
#[doc = r" Search for the book on libgen and return the direct download link, link is created with info on the search result page."]
pub fn search_libgen(title: &String) -> Result<Option<LibgenBook>, LibgenError> {
    // make book_title html encoded
    let encoded_title = encode(&title);
    let mut libgen_search_url: String =
    format!("https://www.libgen.{}/search.php?&req={}&phrase=1&view=simple&column=title&sort=year&sortmode=DESC", LIBGEN_MIRRORS[0], encoded_title);

    let book_row_selector =
        Selector::parse("table.c tbody tr").map_err(|_| LibgenError::ParsingError)?;

    let mut retries = 0;
    let mut retries_domain = 0;
    let client = reqwest::blocking::Client::new();

    // If we send requests to quickly, response 503/server is busy requiring us to loop and retry
    while retries <= MAX_RETRIES {
        let response = match client
            .get(&libgen_search_url)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .send()
        {
            Ok(response) => response,
            Err(_) => {
                eprintln!("Failed to get response");
                return Err(LibgenError::ConnectionError);
            }
        };

        // We need to be gentlemen and not spam libgen
        if response.status() == StatusCode::SERVICE_UNAVAILABLE {
            retries += 1;
            retries_domain = if retries_domain < LIBGEN_MIRRORS.len() - 1 {
                retries_domain + 1
            } else {
                thread::sleep(Duration::from_secs(TIMEOUT_DURATION)); // Adding a delay between retries
                0
            };
            libgen_search_url = format!("https://www.libgen.{}/search.php?&req={}&phrase=1&view=simple&column=title&sort=year&sortmode=DESC", LIBGEN_MIRRORS[retries_domain], encoded_title);
            eprintln!("Waiting...");

            continue;
        }

        if response.status() == StatusCode::OK {
            let document =
                Html::parse_document(&response.text().map_err(|_| LibgenError::ParsingError)?);

            let book_data = document
                .select(&book_row_selector)
                .find_map(|srch_result| process_libgen_search_result(title, srch_result));
            // TODO: Return error if no book, its up to the user to handle it
            return Ok(book_data);
        }
        eprintln!("Server responded with {}", response.status());
        return Err(LibgenError::NetworkError);
    }
    Err(LibgenError::TimeoutError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_book_with_single_author() {
        let generic_book = "Python for Security and Networking".to_string();
        let valid_result = LibgenBook {
            libgen_id: 3759134,
            libgen_group_id: 3759000,
            title: "Python for Security and Networking".to_owned(),
            authors: vec!["José Manuel Ortega".to_string()],
            publisher: "Packt Publishing".to_owned(),
            direct_link: Some("https://download.library.lol/main/3759000/6bed397b612b9e3994a7dc2d6b5440ba/Python%20for%20Security%20and%20Networking.epub".to_owned())
        };
        let search_result = search_libgen(&generic_book);
        match search_result {
            Ok(live_multi_author_return) => {
                // Handle the Option inside the Result
                match live_multi_author_return {
                    Some(actual_result) => {
                        // Assert equality
                        assert_eq!(valid_result, actual_result);
                    }
                    None => {
                        // If search result is None, fail the test
                        panic!("Expected result not found");
                    }
                }
            }
            Err(_) => {
                // If search function returns an error, fail the test
                panic!("Error occurred during search");
            }
        }
    }

    #[test]
    fn search_book_with_multiple_authors() {
        let coauthored_book = "Abstract and concrete categories: the joy of cats".to_string();
        let valid_cat_result = LibgenBook{
            libgen_id: 3750,
            libgen_group_id: 3000,
            title: "Abstract and concrete categories: the joy of cats".to_owned(),
            authors: vec!["Jiri Adamek".to_string(), " Horst Herrlich".to_string(), " George E. Strecker".to_string()],
            publisher: "Wiley-Interscience".to_owned(),
            direct_link: Some("https://download.library.lol/main/3000/5fa82be26689a4e6f4415ea068d35a9d/Abstract%20and%20concrete%20categories%3A%20the%20joy%20of%20cats.pdf".to_owned())
        };

        let search_result = search_libgen(&coauthored_book);
        match search_result {
            Ok(live_multi_author_return) => {
                // Handle the Option inside the Result
                match live_multi_author_return {
                    Some(actual_result) => {
                        // Assert equality
                        assert_eq!(valid_cat_result, actual_result);
                    }
                    None => {
                        // If search result is None, fail the test
                        panic!("Expected result not found");
                    }
                }
            }
            Err(err) => {
                // If search function returns an error, fail the test
                panic!("Error occurred durijng seargch: {:?}", err.to_string());
            }
        }
    }
}
