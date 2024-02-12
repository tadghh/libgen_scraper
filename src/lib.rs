

use std::{error::Error, fs::File, io::{Read, Write}, net::TcpStream};
use scraper::{Html, Selector};
use urlencoding::encode;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// Search for the book on libgen and return the direct download link, link is created with info on the search result page
fn search_libgen(title: &String) -> Option<String>{
    let book_title = encode(&title);
    // make book_title html encoded
    let libgen_search_url: String = format!("https://www.libgen.is/search.php?&req={}&phrase=1&view=simple&column=def&sort=year&sortmode=DESC", book_title);
    let response = reqwest::blocking::get(&libgen_search_url).unwrap().text().unwrap();
    let document = Html::parse_document(&response);
    let book_row_selector = Selector::parse("table.c tbody tr").unwrap();

    //Rows of search results
    for row in document.select(&book_row_selector) {
        let title_cell_selector = Selector::parse("td[width='500'] > a").unwrap();
        let book_group_id_selector = Selector::parse("td:first-child").unwrap();
        let file_type_selector = Selector::parse("td:nth-child(9)").unwrap(); // Assuming the file type is in the 9th column

        //Search result itsel
        if let Some(title_cell) = row.select(&title_cell_selector).next() {
            let file_type = row.select(&file_type_selector).next().unwrap().inner_html();

            // TODO: Add parameter to prefer a file type
            if file_type.contains("epub") {
                //Get the books title and encode it
                let title = title_cell.text().nth(0).unwrap().trim();
                let title_urlencoded = encode(&title);

                //Get the md5 value used by libgen to find the matching book later.
                let mut download_link = title_cell.value().attr("href").unwrap().to_string();

                //Formatting /trimming
                let mut download_link_md5 = download_link.split_off(download_link.find("md5=").unwrap() + 4);

                //Might cause issues
                // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
                download_link_md5 = download_link_md5.to_ascii_lowercase();

                //Books are sorted in groups of divisable by 1000
                let mut book_group_id = row.select(&book_group_id_selector).next().unwrap().inner_html().parse::<f64>().unwrap();
                book_group_id =book_group_id  - (book_group_id % 1000.0);

                let direct_download_url = format!("https://download.library.lol/main/{}/{}/{}.{}",book_group_id,download_link_md5,title_urlencoded,file_type);
                println!("Download Link: {}", direct_download_url);

                // TODO: check for dates and other downloads
                return Some(direct_download_url);
            }
        }
    }
    return None
}

// Downloads the book from a given url
fn download_book_url(url: &String) -> Result<(),  Box<dyn Error>>{
    // Connect to the server
    let mut stream = TcpStream::connect("download.library.lol:80")?;
    println!("Connected!");

    // Send the request
    let request = format!("GET {} HTTP/1.1\r\nHost: download.library.lol\r\n\r\n", url);
    stream.write_all(request.as_bytes())?;
    println!("Sent request!");

    let mut file = File::create("aa.epub")?;
    let mut total_bytes_read = 0;
    let mut buf = [0; 1024]; // buffer to read into
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break; // end of stream
        }
        total_bytes_read += bytes_read;
        file.write_all(&buf[..bytes_read])?;
        println!("Downloaded {} bytes", total_bytes_read);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
