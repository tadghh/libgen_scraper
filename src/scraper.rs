use reqwest::{Client, Error, Response, StatusCode};
use scraper::Html;
use std::{fmt, thread, time::Duration};
use urlencoding::encode;

use crate::{
    book::LibgenBook,
    downloader::{DownloadError, Downloader},
    processor::Processor,
};

const MAX_RETRIES: usize = 3;
const TIMEOUT_DURATION: u64 = 15;
const LIBGEN_MIRRORS: [&str; 3] = ["is", "rs", "st"];

#[derive(Debug, PartialEq)]
pub enum LibgenError {
    /// Connection error while collecting data.
    ConnectionError,
    /// Timeout error while collecting data.
    TimeoutError,
    /// Data not found during collection.
    NotFoundError,
    /// Network error during data collection.
    NetworkError,
    /// Error encountered while parsing collected data.
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

/// The client object for acting agaisnt libgen
pub struct LibgenClient {
    // The request client
    client: Client,
    processor: Processor,
    downloader: Downloader,
}

impl LibgenClient {
    /// Create a reqwest client :3
    pub fn new() -> LibgenClient {
        LibgenClient {
            client: Client::new(),
            processor: Processor::new(),
            downloader: Downloader::new(None),
        }
    }
    pub fn set_download_path(&mut self, new_path: String) {
        self.downloader.change_download_path(new_path);
    }
    pub fn download_book(self, book: &LibgenBook) -> Result<(), DownloadError> {
        self.downloader.download(book)
    }
    /// Request logic
    async fn send_request(&self, url: &str) -> Result<Response, Error> {
        self.client
            .get(url)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .send()
            .await
    }
    /// Search for a book based on its title
    pub async fn search_book_by_title(
        &self,
        title: &str,
    ) -> Result<Option<LibgenBook>, LibgenError> {
        let encoded_title = encode(&title);
        // struct impl new client
        let mut retries = 0;
        let mut retries_domain = 0;

        while retries <= MAX_RETRIES {
            let libgen_search_url: String = format!("https://www.libgen.{}/search.php?&req={}&phrase=1&view=simple&column=title&sort=year&sortmode=DESC", LIBGEN_MIRRORS[retries_domain], encoded_title);

            let response = match self.send_request(&libgen_search_url).await {
                Ok(response) => {
                    // We need to be gentlemen and not spam libgen
                    if response.status() == StatusCode::SERVICE_UNAVAILABLE {
                        retries += 1;
                        retries_domain = if retries_domain < LIBGEN_MIRRORS.len() - 1 {
                            retries_domain + 1
                        } else {
                            thread::sleep(Duration::from_secs(TIMEOUT_DURATION));
                            0
                        };

                        continue;
                    }
                    response
                }
                Err(_) => {
                    return Err(LibgenError::ConnectionError);
                }
            };

            if response.status() == StatusCode::OK {
                let document = Html::parse_document(
                    &response
                        .text()
                        .await
                        .map_err(|_| LibgenError::ParsingError)?,
                );
                return Ok(self.processor.search_title_in_document(&document, title)?);
            }

            return Err(LibgenError::NetworkError);
        }

        Err(LibgenError::TimeoutError)
    }

    // Search for a group of titles
    // pub async fn search_books_by_titles(
    //     &self,
    //     titles: Vec<&str>,
    // ) -> Vec<Result<Option<LibgenBook>, LibgenError>> {
    //     let mut results = Vec::new();

    //     for title in titles {
    //         results.push(self.search_book_by_title(title).await);
    //     }

    //     results
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_book_with_single_author() {
        let test_client = LibgenClient::new();

        let generic_book = "Python for Security and Networking".to_string();

        let valid_result = LibgenBook {
            libgen_id: 3759134,
            libgen_md5: "6bed397b612b9e3994a7dc2d6b5440ba".to_owned(),
            file_type: "epub".to_owned(),
            title: "Python for Security and Networking: Leverage Python modules and tools in securing your network and applications".to_owned(),
            authors: vec!["José Manuel Ortega".to_string()],
            publisher: "Packt Publishing".to_owned(),
        };
        let result = test_client.search_book_by_title(&generic_book);

        match tokio::runtime::Runtime::new().unwrap().block_on(result) {
            Ok(actual_result) => {
                // Assert equality
                match actual_result {
                    Some(result) => {
                        assert_eq!(valid_result, result);
                    }
                    None => panic!("search result was None"),
                }
            }
            Err(err) => {
                // If search function returns an error, fail the test
                panic!("Error occurred during search: {:?}", err);
            }
        }
    }

    #[test]
    fn search_book_with_multiple_authors() {
        let test_client = LibgenClient::new();

        let coauthored_book = "Abstract and concrete categories: the joy of cats".to_string();

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

        let result = test_client.search_book_by_title(&coauthored_book);

        match tokio::runtime::Runtime::new().unwrap().block_on(result) {
            Ok(actual_result) => {
                // Assert equality
                match actual_result {
                    Some(result) => {
                        assert_eq!(valid_cat_result, result);
                    }
                    None => panic!("search result was None"),
                }
            }
            Err(err) => {
                // If search function returns an error, fail the test
                panic!("Error occurred during search: {:?}", err);
            }
        }
    }
}
