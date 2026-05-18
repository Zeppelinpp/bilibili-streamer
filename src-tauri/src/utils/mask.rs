pub fn mask_string(s: &str, visible_start: usize, visible_end: usize) -> String {
    if s.len() <= visible_start + visible_end {
        return "***".to_string();
    }
    let start = &s[..visible_start.min(s.len())];
    let end = &s[s.len().saturating_sub(visible_end)..];
    format!("{}***{}", start, end)
}
