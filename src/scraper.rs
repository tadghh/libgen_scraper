use reqwest::{Client, Error, Response, StatusCode};
use scraper::Html;
use std::{fmt, thread, time::Duration};
use urlencoding::encode;

use crate::{book::LibgenBook, processor::Processor};

const MAX_RETRIES: usize = 3;
const TIMEOUT_DURATION: u64 = 15;
const LIBGEN_MIRRORS: [&str; 3] = ["is", "rs", "st"];

#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
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
}

impl LibgenClient {
    /// Create a reqwest client :3
    pub fn new() -> LibgenClient {
        LibgenClient {
            client: Client::new(),
            processor: Processor::new(),
        }
    }

    /// Request logic
    async fn send_request(&self, url: &str) -> Result<Response, Error> {
        self.client
            .get(url)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .send()
            .await
    }

    async fn search_book_by_title_internal(
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
                Ok(response) => response,
                Err(_) => {
                    return Err(LibgenError::ConnectionError);
                }
            };

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

            if response.status() == StatusCode::OK {
                let document = Html::parse_document(
                    &response
                        .text()
                        .await
                        .map_err(|_| LibgenError::ParsingError)?,
                );

                let book_data = document
                    .select(&self.processor.book_search_result_selector)
                    .find_map(|srch_result| self.processor.parse_search_result(title, srch_result));

                return Ok(book_data);
            }

            return Err(LibgenError::NetworkError);
        }

        Err(LibgenError::TimeoutError)
    }

    /// Search for a single title
    pub async fn search_book_by_title(
        &self,
        title: &str,
    ) -> Result<Option<LibgenBook>, LibgenError> {
        self.search_book_by_title_internal(title).await
    }
    /// Search for a group of titles
    pub async fn search_books_by_titles(
        &self,
        titles: Vec<&str>,
    ) -> Vec<Result<Option<LibgenBook>, LibgenError>> {
        let mut results = Vec::new();

        for title in titles {
            let result = self.search_book_by_title_internal(title).await;
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info, Level};
    use tracing_subscriber::FmtSubscriber;

    #[tokio::test]
    async fn search_book_with_single_author() {
        let test_client = LibgenClient::new();
        let generic_book = "Python for Security and Networking".to_string();
        let valid_result = LibgenBook {
            libgen_id: 3759134,
            libgen_group_id: 3759000,
            title: "Python for Security and Networking".to_owned(),
            authors: vec!["José Manuel Ortega".to_string()],
            publisher: "Packt Publishing".to_owned(),
            direct_link: Some("https://download.library.lol/main/3759000/6bed397b612b9e3994a7dc2d6b5440ba/Python%20for%20Security%20and%20Networking.epub".to_owned())
        };

        match test_client.search_book_by_title(&generic_book).await {
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

    #[tokio::test]
    async fn search_book_with_multiple_authors() {
        let test_client = LibgenClient::new();

        let coauthored_book = "Abstract and concrete categories: the joy of cats".to_string();
        let valid_cat_result = LibgenBook{
            libgen_id: 3750,
            libgen_group_id: 3000,
            title: "Abstract and concrete categories: the joy of cats".to_owned(),
            authors: vec!["Jiri Adamek".to_string(), " Horst Herrlich".to_string(), " George E. Strecker".to_string()],
            publisher: "Wiley-Interscience".to_owned(),
            direct_link: Some("https://download.library.lol/main/3000/5fa82be26689a4e6f4415ea068d35a9d/Abstract%20and%20concrete%20categories%3A%20the%20joy%20of%20cats.pdf".to_owned())
        };

        match test_client.search_book_by_title(&coauthored_book).await {
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
                panic!("Error occurred during search: {:?}", err);
            }
        }
    }

    #[tokio::test]
    async fn bench_book_search() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .without_time()
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        // Inner test to search for five books
        search_for_five_books().await;

        // Inner test to search for fifteen books
        search_for_fifteen_books().await;
    }

    async fn search_for_five_books() {
        // Initialize a LibgenClient instance
        let libgen_client = LibgenClient::new();

        // Define a list of titles to search
        let titles = vec![
            "The Predator (Animorphs)",
            "Color Atlas of Pharmacology",
            "Physics of life",
            "Physics and Chemistry Basis of Biotechnology",
            "Medical Imaging Physics",
        ];

        // Call the search_books_by_titles method and collect the results
        let start_time = std::time::Instant::now();
        let results = libgen_client.search_books_by_titles(titles).await;
        let end_time = std::time::Instant::now();

        // Calculate the duration
        let duration = end_time - start_time;

        // Iterate over the results and perform assertions
        for result in results {
            // Check that each result is not an error
            assert!(result.is_ok());

            // Unwrap the result and check that the option is not None
            let book_data = result.unwrap();
            assert!(book_data.is_some());

            // Print or perform further assertions with the book data if needed
            if let Some(book) = book_data {
                info!("Found book: {:?}", book.title);
            }
        }
        info!(
            "Search for five books took {} seconds",
            duration.as_secs_f64()
        );
    }

    async fn search_for_fifteen_books() {
        // Initialize a LibgenClient instance
        let libgen_client = LibgenClient::new();

        // Define a list of titles to search
        let titles = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
        "Lectures On Statistical Physics And Protein Folding",
        "Structural theory of automata, semigroups, and universal algebra",
        "Computer Algebra Recipes for Mathematical Physics",
        "Quantum Information: An Introduction to Basic Theoretical Concepts and Experiments",
        "Terahertz Optoelectronics",
        "Leaving Earth: Space Stations, Rival Superpowers, and the Quest for Interplanetary Travel",
        "Classical Banach-Lie algebras and Banach-Lie groups of operators in Hilbert space",
        "Grammar, punctuation, and capitalization: a handbook for technical writers and editors",
        "Canonical Perturbation Theories: Degenerate Systems and Resonance",
        "The rigged hilbert space and quantum mechanics",
        "The mathematical theory of symmetry in solids; representation theory for point groups and space groups",
    ];

        // Call the search_books_by_titles method and collect the results
        let start_time = std::time::Instant::now();
        let results = libgen_client.search_books_by_titles(titles).await;
        let end_time = std::time::Instant::now();

        // Calculate the duration
        let duration = end_time - start_time;

        // Iterate over the results and perform assertions
        for result in results {
            // Check that each result is not an error
            assert!(result.is_ok());

            // Unwrap the result and check that the option is not None
            let book_data = result.unwrap();
            assert!(book_data.is_some());

            // Print or perform further assertions with the book data if needed
            if let Some(book) = book_data {
                info!("Found book: {:?}", book.title);
            }
        }
        info!(
            "Search for fifteen books took {} seconds",
            duration.as_secs_f64()
        );
    }
}
