fn has_word(before: &str, words: &[&str]) -> bool {
    before.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|word| words.contains(&word))
}

pub fn public(_before: &str, _name: &str) -> bool {
    false
}

pub fn private_keyword(before: &str, _name: &str) -> bool {
    has_word(before, &["private", "fileprivate"])
}

pub fn underscore(_before: &str, name: &str) -> bool {
    name.starts_with('_')
}

pub fn local(before: &str, _name: &str) -> bool {
    before.trim_start().starts_with("local")
}

pub fn defp(before: &str, _name: &str) -> bool {
    has_word(before, &["defp", "defmacrop"])
}

pub fn without_pub(before: &str, _name: &str) -> bool {
    !has_word(before, &["pub", "export"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_rule_reads_only_the_words_before_the_name() {
        assert!(private_keyword("@objc\n    fileprivate func ", "helper") && !private_keyword("func ", "privateKey"));
        assert!(underscore("", "_helper") && !underscore("", "helper"));
        assert!(local("local function ", "helper") && !local("function ", "localize"));
        assert!(defp("defp ", "helper") && !defp("def ", "defp_like"));
        assert!(without_pub("fn ", "helper") && !without_pub("pub fn ", "helper") && !without_pub("export fn ", "helper"));
    }
}
