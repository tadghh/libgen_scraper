use scraper::{ElementRef, Html, Selector};
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

// TODO: Better naming style

// TODO: Make Error types

#[derive(Debug, PartialEq)]
struct LibgenBookData {
    libgen_id: u64,
    libgen_group_id: u64,
    title: String,
    authors: Vec<String>,
    publisher: String,
    direct_link: Option<String>,
}
impl LibgenBookData {
    // TODO: Do this, the struct will carry the original url anyway
    fn GetDirectDownloadLink() {}
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

// Private function to process a libgen search result
fn process_libgen_search_result(
    title: &String,
    result_row: ElementRef<'_>,
) -> Option<LibgenBookData> {
    // CSS Selectors
    // Books libgen id
    let book_libgen_id_selector = Selector::parse("td:first-child").unwrap();

    let book_id_elem = result_row.select(&book_libgen_id_selector).next()?;
    let book_id = book_id_elem.inner_html().parse::<u64>().ok()?;

    // CSS to grab the title of a search result
    let title_cell_selector =
        Selector::parse(&format!("td[width='500'] > a[id='{}']", book_id)).unwrap();

    let title_cell = result_row.select(&title_cell_selector).next()?;
    let search_result_title = title_cell.text().nth(0)?;

    // If the search result title doesnt contain/match the title parameter return none. We know it isn't the correct book
    if !search_result_title
        .to_ascii_lowercase()
        .contains(&title.to_ascii_lowercase())
    {
        println!("No title");
        println!("{:?}", search_result_title);
        //the result does not contain the given example/shortened title (comparing to method parameter this is the books title)
        return None;
    }

    // Books publisher selector
    let publisher_selector = Selector::parse("td:nth-child(4)").unwrap();
    // Book file type
    let file_type_selector = Selector::parse("td:nth-child(9)").unwrap();

    // Get all the authors for the book
    let authors_selector = Selector::parse("td:nth-child(2) > a:not([title])").unwrap();

    // TODO: Alternate path, going to the book download page on libgen and grabbin the url there instead of skipping it (since we are creating the direct link from the info on the search page).
    let file_type = result_row
        .select(&file_type_selector)
        .next()
        .unwrap()
        .inner_html()
        .to_owned();

    let href_book_link = title_cell.value().attr("href")?.to_owned();

    // let book_id = result_row
    //     .select(&book_libgen_id_selector)
    //     .next()
    //     .unwrap()
    //     .inner_html()
    //     .parse::<u64>()
    //     .unwrap();

    let book_group_id = calculate_group_id(book_id);

    //let mut authors: Vec<_> = result_row.select(&authors_selector).collect::<Vec<_>>();
    //authors = authors.into_iter().map(|auth| auth.inner_html()).collect();

    let authors: Vec<_> = result_row
        .select(&authors_selector)
        .into_iter()
        .map(|auth| auth.inner_html())
        .collect();

    let publisher = result_row
        .select(&publisher_selector)
        .next()
        .unwrap()
        .inner_html();

    let direct_link =
        build_direct_download_url(book_id, href_book_link, &title.to_string(), file_type).ok();

    Some(LibgenBookData {
        title: title.to_owned(),
        libgen_id: book_id,
        libgen_group_id: book_group_id,
        publisher,
        authors,
        direct_link,
    })
}

// Search for the book on libgen and return the direct download link, link is created with info on the search result page
fn search_libgen(title: &String) -> Option<LibgenBookData> {
    // make book_title html encoded
    let libgen_search_url: String = format!("https://www.libgen.is/search.php?&req={}&phrase=1&view=simple&column=def&sort=year&sortmode=DESC", encode(&title));

    // Get the response, using ok() turning it into an option ? then throws it up to the calling function
    let response = reqwest::blocking::get(&libgen_search_url).ok()?;

    // From the response access it as text, again using ok to turn the result in and option we can throw up. If we need to throw up ofc
    // Then parsing the text of the response into an HTML object
    let document = Html::parse_document(&response.text().ok()?);

    // Our CSS selector to grab relevant matching elements
    let book_row_selector = Selector::parse("table.c tbody tr").ok()?;

    // Go through our list of results, processing them until we are returned "Some" value.
    // If no value is found None is returned.
    document
        .select(&book_row_selector)
        .find_map(|srch_result| process_libgen_search_result(title, srch_result))
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
        let valid_cat_result = LibgenBookData{
            libgen_id: 3750,
            libgen_group_id: 3000,
            title: "Abstract and concrete categories: the joy of cats".to_owned(),
            authors: vec!["Jiri Adamek".to_string(), " Horst Herrlich".to_string(), " George E. Strecker".to_string()],
            publisher: "Wiley-Interscience".to_owned(),
            direct_link: Some("https://download.library.lol/main/3000/book/index.php?/Abstract%20and%20concrete%20categories%3A%20the%20joy%20of%20cats.pdf".to_owned())
        };

        let coauthored_book = "Abstract and concrete categories: the joy of cats".to_string();
        let live_multi_author_return = search_libgen(&coauthored_book).unwrap();
        // assert_eq!(valid_result, live_return);
        println!("{:?}", live_multi_author_return);
        assert_eq!(valid_cat_result, live_multi_author_return);
        // Add test for multi authors
    }
}
