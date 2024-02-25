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
pub mod book;
use book::LibgenBook;
use reqwest::StatusCode;
use scraper::{ElementRef, Html, Selector};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};
use urlencoding::encode;
pub mod util;
use util::{calculate_group_id, parse_md5_from_url};
// TODO: make docs

// TODO: Maybe this is impl on the struct
#[doc = r" Downloads a book from the given direct download url."]
pub fn download_book_url(url: &String) -> Result<(), Box<dyn Error>> {
    // Connect to the server

    // clean up the accept, add templating or do this headers another way
    let request = "GET /main/3759000/6bed397b612b9e3994a7dc2d6b5440ba/Jos%C3%A9%20Manuel%20Ortega%20-%20Python%20for%20Security%20and%20Networking_%20Leverage%20Python%20modules%20and%20tools%20in%20securing%20your%20network%20and%20applications-Packt%20Publishing%20%282023%29.epub HTTP/1.1\r\n\
        Host: download.library.lol\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        Accept-Language: en-CA,en;q=0.9\r\n\
        Cache-Control: max-age=0\r\n\
        Connection: keep-alive\r\n\
        DNT: 1\r\n\
        Referer: https://library.lol/\r\n\
        Sec-Fetch-Dest: document\r\n\
        Sec-Fetch-Mode: navigate\r\n\
        Sec-Fetch-Site: same-site\r\n\
        Sec-Fetch-User: ?1\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        sec-ch-ua: \"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"\r\n\
        sec-ch-ua-mobile: ?0\r\n\
        sec-ch-ua-platform: \"Windows\"\r\n\
        sec-gpc: 1\r\n\
        \r\n";

    // Connect to the server
    let mut stream = TcpStream::connect("download.library.lol:80")?;
    println!("Connected!");

    // Send the request
    stream.write_all(request.as_bytes())?;
    println!("Sent stream!");

    // Read the response body into a buffer
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;
    println!("Read response body!");

    // Write the buffer to a file
    let mut file = File::create("aa.epub")?;
    file.write_all(&buffer)?;
    println!("File downloaded successfully!");

    Ok(())
}
