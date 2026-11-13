pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for letter in title.to_lowercase().chars() {
        let plain = match letter {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            other => other,
        };
        if plain.is_ascii_alphanumeric() {
            out.push(plain);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').chars().take(64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_becomes_a_plain_slug() {
        assert_eq!(slug("Ticket Tailor"), "ticket-tailor");
        assert_eq!(slug("  Atlassian Rovo (beta)!"), "atlassian-rovo-beta");
        assert_eq!(slug("Cañón"), "canon");
    }
}
