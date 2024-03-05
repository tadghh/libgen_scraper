//! Simple Library Genesis web scraper.
//!
//! This crate will allow you to scrape information and download books, hosted on Library Genesis.
//! Please be careful to not accidentally DOS Libgen.
//!
//! ### Current Features
//! - Downloading books
//! - Pulling information about a book
//!
//! ### Planned
//! - Preferred file types
//! - Multithreading
//! - Just make it better
//!
//!
#![warn(missing_docs)]

/// Book module
pub mod book;
/// CSS Selectors
pub mod processor;
/// HTML libgen scraper
pub mod scraper;
/// One off methods
pub mod util;
