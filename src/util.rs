pub fn parsemd5_from_url(url: String) -> Option<String> {
    Some(url.split("md5=").next()?.to_lowercase())
}

pub fn calculate_group_id(id: u64) -> u64 {
    (id / 1000) * 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_group_id_equal_zero() {
        assert_eq!(calculate_group_id(0), 0);
    }
    #[test]
    fn test_group_id_below_1000() {
        assert_eq!(calculate_group_id(531), 0);
    }
    #[test]
    fn test_group_id_equal_1000() {
        assert_eq!(calculate_group_id(1000), 1000);
    }
    #[test]
    fn test_group_id_above_1000() {
        assert_eq!(calculate_group_id(1999), 1000);
    }
    #[test]
    fn test_group_id_large() {
        assert_eq!(calculate_group_id(19992123), 19992000);
    }
}
