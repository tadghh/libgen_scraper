use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
};

use crate::book::LibgenBook;

#[derive(Debug, PartialEq)]
#[doc = r" The data collected from a search result."]
pub struct Downloader {
    /// Request headers
    request_ops: String,
    download_path: String,
    hosts: Vec<String>,
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader {
            request_ops: "\r\n\
			Host: download.library.lol\r\n\
			Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
			Accept-Encoding: gzip, deflate, br\r\n\
			Accept-Language: en-CA,en;q=0.9\r\n\
			Cache-Control: max-age=0\r\n\
			Connection: keep-alive\r\n\
			\r\n".to_owned() ,
			download_path: "".to_owned(),
			hosts:  vec!["download.library.lol:80".to_string()]

        }
    }

    pub fn change_download_path(&mut self, new_path: String) {
        self.download_path = new_path
    }

    pub fn get_download_path(&self) -> String {
        self.download_path.to_string()
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
        format!("{}{}", book.title, book.file_type)
    }
    #[doc = r"Downloads the book based on its direct link."]
    pub fn download(&self, book: &LibgenBook) -> Result<(), Box<dyn Error>> {
        let built_request = format!(
            "GET {:?} HTTP/1.1 {}",
            Self::get_book_download_headers(&book),
            self.request_ops
        );
        println!("request preview: \n {:?}", built_request);

        // Connect to the server
        // TODO: This should be and option that can be set
        let mut stream = TcpStream::connect("download.library.lol:80")?;
        // Send the request
        stream.write_all(built_request.as_bytes())?;
        // Read the response body into a buffer
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        // Write the buffer to a file
        // TODO: Need a way to set the download path on the client, having it with all the book objects is a waste
        let download_filename = Self::create_book_download_name(&book);
        let mut file = File::create(download_filename)?;
        file.write_all(&buffer)?;

        Ok(())
    }
}
