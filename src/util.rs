/// Parses the MD5 hash from a Libgen URL.
///
/// Given a Libgen URL, this function attempts to extract the MD5 hash from it.
/// The MD5 hash is often included in Libgen URLs as a query parameter, typically
/// following the pattern `md5=<md5_hash>`. This function extracts the hash portion
/// and returns it in lowercase.
///
/// # Arguments
///
/// * `url` - The Libgen URL from which the MD5 hash needs to be parsed.
///
/// # Returns
///
/// An `Option<String>`:
/// - `Some(String)`: If the MD5 hash is successfully parsed from the URL, returns
///   the hash in lowercase.
/// - `None`: If the MD5 hash cannot be extracted from the URL or if the URL is empty.
///
/// # Examples
///
/// ```
/// use libgen_scraper::util::parse_md5_from_url;
/// // Note the mixed casing in the URL
/// let url = "http://libgen.example.com/book?id=12345&md5=abCdE".to_string();
/// let md5 = parse_md5_from_url(url);
/// assert_eq!(md5, Some("abcde".to_string()));
/// ```
///
/// # Notes
///
/// - If the URL contains multiple occurrences of `md5=`, this function will only
///   consider the first occurrence.
/// - The extracted MD5 hash is converted to lowercase before being returned.
///
pub fn parse_md5_from_url(url: String) -> Option<String> {
    Some(url.split("md5=").nth(1)?.to_lowercase())
}

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
pub fn calculate_group_id(id: u64) -> u64 {
    (id / 1000) * 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_id_equal_zero() {
        assert_eq!(calculate_group_id(0), 0);
    }
    #[test]
    fn group_id_below_1000() {
        assert_eq!(calculate_group_id(531), 0);
    }
    #[test]
    fn group_id_equal_1000() {
        assert_eq!(calculate_group_id(1000), 1000);
    }
    #[test]
    fn group_id_above_1000() {
        assert_eq!(calculate_group_id(1999), 1000);
    }
    #[test]
    fn group_id_large() {
        assert_eq!(calculate_group_id(19992123), 19992000);
    }
    #[test]
    fn md5_happy_path_from_url() {
        let url = "http://libgen.example.com/book?id=12345&md5=abcde".to_string();
        let md5 = parse_md5_from_url(url);
        assert_eq!(md5, Some("abcde".to_string()));
    }
    #[test]
    fn md5_mixed_case_from_url() {
        let url = "http://libgen.example.com/book?id=12345&md5=123CbbDeeeFGS".to_string();
        let md5 = parse_md5_from_url(url);
        assert_eq!(md5, Some("123cbbdeeefgs".to_string()));
    }
    #[test]
    fn md5_missing_from_url() {
        let url = "http://libgen.example.com/book?id=12345".to_string();
        let md5 = parse_md5_from_url(url);
        assert_eq!(md5, None);
    }
}
