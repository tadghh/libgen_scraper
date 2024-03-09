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

// // TODO: should accept mirrors
// pub fn build_direct_download_url(
//     book_id: u64,
//     url: String,
//     title: &String,
//     file_type: String,
// ) -> Result<String, String> {
//     if let Some(md5_value) = parse_md5_from_url(url) {
//         Ok(format!(
//             "https://download.library.lol/main/{}/{}/{}.{}",
//             calculate_group_id(book_id),
//             md5_value,
//             encode(title),
//             file_type
//         ))
//     } else {
//         Err("No 'md5' parameter found in the URL".to_string())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn group_id_equal_zero() {
//         assert_eq!(calculate_group_id(0), 0);
//     }
//     #[test]
//     fn group_id_below_1000() {
//         assert_eq!(calculate_group_id(531), 0);
//     }
//     #[test]
//     fn group_id_equal_1000() {
//         assert_eq!(calculate_group_id(1000), 1000);
//     }
//     #[test]
//     fn group_id_above_1000() {
//         assert_eq!(calculate_group_id(1999), 1000);
//     }
//     #[test]
//     fn group_id_large() {
//         assert_eq!(calculate_group_id(19992123), 19992000);
//     }
//     #[test]
//     fn md5_happy_path_from_url() {
//         let url = "http://libgen.example.com/book?id=12345&md5=abcde".to_string();
//         let md5 = parse_md5_from_url(url);
//         assert_eq!(md5, Some("abcde".to_string()));
//     }
//     #[test]
//     fn md5_mixed_case_from_url() {
//         let url = "http://libgen.example.com/book?id=12345&md5=123CbbDeeeFGS".to_string();
//         let md5 = parse_md5_from_url(url);
//         assert_eq!(md5, Some("123cbbdeeefgs".to_string()));
//     }
//     #[test]
//     fn md5_missing_from_url() {
//         let url = "http://libgen.example.com/book?id=12345".to_string();
//         let md5 = parse_md5_from_url(url);
//         assert_eq!(md5, None);
//     }
// }
