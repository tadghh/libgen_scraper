use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
};

use urlencoding::encode;

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
    /// Calculates the group id for a book based on its id.
    ///
    /// Libgen (Library Genesis) sorts books by the thousandth of their ids. This function
    /// takes a book's id as input and returns the corresponding group id, which represents
    /// the group of books that share the same thousandth in their ids.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the book for which the group id needs to be calculated.
    ///
    /// # Returns
    ///
    /// The group id for the given book id.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// use libgen_scraper::util::calculate_group_id;
    /// let book_id = 123456;
    /// let group_id = calculate_group_id(book_id);
    /// assert_eq!(group_id, 123000);
    /// ```
    ///
    /// # Notes
    ///
    /// - The group id is calculated by dividing the book's id by 1000 and then multiplying
    ///   the result by 1000, effectively rounding down the id to the nearest thousandth.
    ///
    fn calculate_group_id(&self, id: u64) -> u64 {
        (id / 1000) * 1000
    }
    #[doc = r"Build the books download link."]
    pub fn build_direct_download_url(&self) -> Result<String, String> {
        Ok(format!(
            "https://download.library.lol/main/{}/{}/{}.{}",
            self.calculate_group_id(self.libgen_id),
            self.libgen_md5,
            encode(&self.title),
            self.file_type
        ))
    }
    #[doc = r"Downloads the book based on its direct link."]
    pub fn download(&self) -> Result<(), Box<dyn Error>> {
        let binding = self.build_direct_download_url();
        let mut download_path = binding.as_ref().unwrap().as_str();
        if let Some(index) = download_path.find("/main/") {
            download_path = &download_path[index..];
        }
        // TODO: Take another look at this
        // Storing this massive string with every book
        let request_ops = "\r\n\
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

        let built_request = format!("GET {} HTTP/1.1 {}", download_path, request_ops);

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
        let mut file = File::create("test.epub")?;
        file.write_all(&buffer)?;

        Ok(())
    }
}
