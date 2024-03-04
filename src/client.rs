pub struct Client();

impl Client {
    pub fn new() {
        let book_libgen_id_selector = Selector::parse("td:first-child").unwrap();
        let publisher_selector: Selector = Selector::parse("td:nth-child(4)").unwrap();
        let authors_selector = Selector::parse("td:nth-child(2) > a:not([title])").unwrap();
    }
}
