pub fn smart_split(text: String) -> Vec<String> {
    let words = text.split_whitespace();
    let mut result = Vec::new();
    let mut word_finnished = true;
    for word in words {
        let count = word.matches('"').count();
        let count2 = word.matches("'").count();
        if word_finnished {
            if count % 2 != 0 || count2 % 2 != 0 {
                word_finnished = false;
            }
            result.push(word.to_string());
        } else {
            if let Some(last) = result.last_mut() {
                last.push_str(" ");
                last.push_str(word);
                if count % 2 != 0 || count2 % 2 != 0 {
                    word_finnished = true;
                }
            }
        }
    }
    return result;
}

#[cfg(test)]
#[test]
fn test_smart_split() {
    let v = smart_split("text".to_string());
    assert_eq!(v.len(), 1);
    let v = smart_split("text word1 word2".to_string());
    assert_eq!(v.len(), 3);
    let v = smart_split("text 'word1 word2'".to_string());
    assert_eq!(v.len(), 2);
    assert_eq!(v.last().unwrap(), "'word1 word2'");
    let v = smart_split(r#"text 'word1 "word2"'"#.to_string());
    assert_eq!(v.len(), 2);
    assert_eq!(v.last().unwrap(), r#"'word1 "word2"'"#);
}
