use crate::book::LibgenBook;
use core::fmt;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    error::Error,
    f64::consts::E,
    fs::File,
    io::{self, Read, Write},
    net::TcpStream,
};
// Define your custom error enum
#[derive(Debug)]
pub enum DownloadError {
    /// Connection error while collecting data.
    ConnectionError(String),
    /// Timeout error while collecting data.
    DownloadError(String),
    /// Download directory was not found or was none.
    DirectoryError,
    /// Other IO error occurred.
    IOError(String),
}

// Implement Display for DownloadError
impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_str = match self {
            DownloadError::ConnectionError(ref err) => err.as_str(),
            DownloadError::DownloadError(ref err) => err.as_str(),
            DownloadError::DirectoryError => "DirectoryError",
            DownloadError::IOError(ref err) => err.as_str(),
        };
        write!(f, "{}", error_str)
    }
}
lazy_static! {
    static ref RE: Regex = Regex::new(r#"[\/:*?"<>|]"#).unwrap();
}

#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
pub struct Downloader {
    /// Request headers
    request_ops: String,
    download_path: Option<String>,
    hosts: Vec<String>,
}
impl From<io::Error> for DownloadError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::NotConnected
            | io::ErrorKind::TimedOut => DownloadError::ConnectionError(format!("{}", error)),
            _ => DownloadError::IOError(error.to_string()),
        }
    }
}
impl Downloader {
    /// Downloader object
    pub fn new(download_path: Option<String>) -> Downloader {
        Downloader {
            request_ops: "\r\n\
			Host: download.library.lol\r\n\
			Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
			Accept-Encoding: gzip, deflate, br\r\n\
			Accept-Language: en-CA,en;q=0.9\r\n\
			Cache-Control: max-age=0\r\n\
			Connection: keep-alive\r\n\
			\r\n".to_owned() ,
            download_path: download_path.or_else(|| Some(String::from("."))),
            hosts:  vec!["download.library.lol:80".to_string()]

        }
    }

    /// Changes the location to download into
    pub fn change_download_path(&mut self, new_path: String) {
        self.download_path = Some(new_path)
    }

    /// Gets the current download directory
    pub fn get_download_path(self) -> Option<String> {
        self.download_path
    }

    fn get_book_download_headers(book: &LibgenBook) -> Option<String> {
        let binding = book.build_direct_download_url();
        let download_path = binding.as_ref().unwrap();
        if let Some(index) = download_path.find("/main/") {
            return Some(download_path[index..].to_string());
        }
        None
    }

    fn create_book_download_name(book: &LibgenBook) -> String {
        let temp = [book.title.clone(), book.file_type.clone()].join(".");
        RE.replace_all(&temp, "_").to_string()
    }

    #[doc = r"Downloads the book based on its direct link."]
    pub fn download(&self, book: &LibgenBook) -> Result<(), DownloadError> {
        let book_headers = Self::get_book_download_headers(&book).ok_or_else(|| {
            DownloadError::ConnectionError("Failed to create download headers".to_string())
        })?;

        let built_request = format!("GET {} HTTP/1.1 {}", book_headers, self.request_ops);
        println!("request preview: \n {:?}", built_request);

        // Connect to the server
        // TODO: use the hosts vec array
        let mut stream = TcpStream::connect("download.library.lol:80")
            .ok()
            .expect("issue");

        // Send the request
        match stream.write_all(built_request.as_bytes()) {
            Ok(()) => {
                // Read the response body into a buffer
                println!("request preview: \n {:?}", "yo");
                // Write the buffer to a file
                // TODO: Make filenames OS Friendly
                let download_filename = match &self.download_path {
                    Some(path) => format!("{}/{}", path, Self::create_book_download_name(book)),
                    None => String::from("downloaded_book.pdf"),
                };
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer)?;
                println!("read");

                println!("request : \n {:?}", download_filename);
                let mut file = File::create(download_filename)?;
                file.write_all(&buffer)?;

                return Ok(());
            }
            Err(_) => panic!("Fuck"),
        }
    }
}
