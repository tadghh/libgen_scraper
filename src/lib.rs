use scraper::{Html, Selector};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    result,
};
use urlencoding::encode;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, PartialEq)]
struct LibgenBookData {
    libgen_id: u64,
    libgen_group_id: u64,
    title: String,
    authors: Vec<String>,
    publisher: String,
    direct_link: Option<String>,
}

fn parsemd5_from_url(url: String) -> Option<String> {
    if let Some(md5_hash) = url.split("md5=").next() {
        return Some(md5_hash.to_lowercase());
    }
    None
}

fn calculate_group_id(id: u64) -> u64 {
    return id - (id % 1000);
}

// TODO: should accept mirrors
fn build_direct_download_url(
    book_id: u64,
    url: String,
    title: &String,
    file_type: String,
) -> Result<String, String> {
    if let Some(md5_value) = parsemd5_from_url(url) {
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
                    let authors_selector = Selector::parse("td > a:not([title])").unwrap();

                    //Make sure there is a title atleast
                    if let Some(title_cell) = row.select(&title_cell_selector).next() {
                        // TODO: Add parameter to prefer a file type
                        //Get the books title and encode it
                        if let Some(result_title) = title_cell.text().nth(0) {
                            if !result_title
                                .to_ascii_lowercase()
                                .contains(&title.to_ascii_lowercase())
                            {
                                //the result does not contain the given example/shortened title (comparing to method parameter this is the books title)
                                break;
                            }
                        } else {
                            // There is no title for this result
                            break;
                        }

                        //let title = title_cell.text().nth(0).unwrap_or("Missing Title").trim();

                        //Might cause issues
                        // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
                        let file_type = row
                            .select(&file_type_selector)
                            .next()
                            .unwrap()
                            .inner_html()
                            .to_owned();
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

                        let publisher =
                            row.select(&publisher_selector).next().unwrap().inner_html();

                        let direct_link = match build_direct_download_url(
                            book_id,
                            href_book_link,
                            &title.to_string(),
                            file_type,
                        ) {
                            Ok(dog) => Some(dog), // If build_direct_download_url succeeds, assign the direct link
                            Err(_) => None, // If build_direct_download_url fails, assign an empty string
                        };
                        return Some(LibgenBookData {
                            title: title.to_owned(),
                            libgen_id: book_id,
                            libgen_group_id: book_group_id,
                            publisher,
                            authors,
                            direct_link,
                        });
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

    #[test]
    fn test_calculate_group_id() {
        // if the function input is somehow negative thats nmp, libgen broke.

        // Test cases where id is divisible by 1000
        assert_eq!(calculate_group_id(0), 0);
        assert_eq!(calculate_group_id(1000), 1000);
        assert_eq!(calculate_group_id(5000), 5000);

        // Test cases where id is not divisible by 1000
        assert_eq!(calculate_group_id(1), 0);
        assert_eq!(calculate_group_id(999), 0);
        assert_eq!(calculate_group_id(1001), 1000);
        assert_eq!(calculate_group_id(1499), 1000);
        assert_eq!(calculate_group_id(1555), 1000);
        assert_eq!(calculate_group_id(1999), 1000);
    }

    #[test]
    fn test_search_libgen() {
        // This may change but it correct as of Feb 12, 2024
        let valid_result = LibgenBookData {
            libgen_id: 3759134,
            libgen_group_id: 3759000,
            title: "Python for Security and Networking".to_owned(),
            authors: vec!["José Manuel Ortega".to_string()],
            publisher: "Packt Publishing".to_owned(),
            direct_link: Some("https://download.library.lol/main/3759000/book/index.php?/Python%20for%20Security%20and%20Networking.epub".to_owned())
        };

        let title = "Python for Security and Networking".to_string();

        let live_return = search_libgen(&title).unwrap();
        assert_eq!(valid_result, live_return);
        println!("{:?}", live_return);
    }
}
