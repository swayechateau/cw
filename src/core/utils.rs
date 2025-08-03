// core::utils.rs

/// Capitalizes the first letter of a string
pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Checks if a string is blank (empty or whitespace only)
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first_letter_normal() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
    }

    #[test]
    fn test_capitalize_first_letter_uppercase_input() {
        assert_eq!(capitalize_first_letter("Hello"), "Hello");
    }

    #[test]
    fn test_capitalize_first_letter_empty_string() {
        assert_eq!(capitalize_first_letter(""), "");
    }

    #[test]
    fn test_capitalize_first_letter_single_char() {
        assert_eq!(capitalize_first_letter("a"), "A");
    }

    #[test]
    fn test_capitalize_first_letter_non_ascii() {
        assert_eq!(capitalize_first_letter("ßeta"), "SSeta"); // ß uppercases to SS
    }

    #[test]
    fn test_is_blank_true() {
        assert!(is_blank(""));
        assert!(is_blank(" "));
        assert!(is_blank("\t\n"));
    }

    #[test]
    fn test_is_blank_false() {
        assert!(!is_blank("a"));
        assert!(!is_blank("  b  "));
    }
}
