
pub fn is_all_whitespace(s: &str) -> bool {
    s.chars().all(|c| c.is_whitespace())
}
