use scraper::{Html, Selector};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
};
use urlencoding::encode;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

struct LibgenBookData {
    libgen_id: u64,
    libgen_group_id: u64,
    title: String,
    authors: Vec<String>,
    publishers: String,
    direct_link: String,
}

fn parsemd5_from_url(url: String) -> String {
    return url.split("md5=").next().unwrap_or("").to_lowercase();
}

fn calculate_group_id(id: u64) -> u64 {
    return id - (id % 1000);
}

fn build_direct_download_url(
    book_id: u64,
    url: String,
    title: &String,
    file_type: String,
) -> String {
    return format!(
        "https://download.library.lol/main/{}/{}/{}.{}",
        calculate_group_id(book_id),
        parsemd5_from_url(url),
        encode(title),
        file_type
    );
}
// Search for the book on libgen and return the direct download link, link is created with info on the search result page
fn search_libgen(title: &String) -> Option<LibgenBookData> {
    let book_title = encode(&title);
    // make book_title html encoded
    let libgen_search_url: String = format!("https://www.libgen.is/search.php?&req={}&phrase=1&view=simple&column=def&sort=year&sortmode=DESC", book_title);

    if let Ok(response) = reqwest::blocking::get(&libgen_search_url) {
        if let Ok(text) = response.text() {
            let document = Html::parse_document(&text);
            let mut index = 0;
            if let Ok(book_row_selector) = Selector::parse("table.c tbody tr") {
                //Rows of search results
                for row in document.select(&book_row_selector) {
                    println!("{}", index);
                    index = index + 1;
                    let title_cell_selector = Selector::parse("td[width='500'] > a").unwrap();
                    let book_group_id_selector = Selector::parse("td:first-child").unwrap();
                    let publisher_selector = Selector::parse("td:nth-child(4)").unwrap(); // Assuming the file type is in the 9th column
                    let file_type_selector = Selector::parse("td:nth-child(9)").unwrap(); // Assuming the file type is in the 9th column
                                                                                          //Get authors
                    let authors_selector = Selector::parse("td ~ a:not([title])").unwrap();

                    //Search result itsel
                    if let Some(title_cell) = row.select(&title_cell_selector).next() {
                        // TODO: Add parameter to prefer a file type
                        if let Some(file_type_element) = row.select(&file_type_selector).next() {
                            //Get the books title and encode it
                            let title = title_cell.text().nth(0).unwrap_or("Missing Title").trim();

                            //Might cause issues
                            // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
                            let href_book_link = title_cell
                                .value()
                                .attr("href")
                                .unwrap_or("missing")
                                .to_owned();

                            //Books are sorted in groups of divisable by 1000
                            let book_id = row
                                .select(&book_group_id_selector)
                                .next()
                                .unwrap()
                                .inner_html()
                                .parse::<u64>()
                                .unwrap();
                            let book_group_id = calculate_group_id(book_id);

                            let authors: Vec<String> = row
                                .select(&authors_selector)
                                .map(|author| author.inner_html())
                                .collect();

                            let file_type_str = file_type_element.inner_html();
                            return Some(LibgenBookData {
                                title: title.to_owned(),
                                libgen_id: book_id,
                                libgen_group_id: book_group_id,
                                publishers: String::from("temp"),
                                authors,
                                direct_link: build_direct_download_url(
                                    book_id,
                                    href_book_link,
                                    &title.to_string(),
                                    file_type_str,
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    return None;
}

// Downloads the book from a given url
fn download_book_url(url: &String) -> Result<(), Box<dyn Error>> {
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
