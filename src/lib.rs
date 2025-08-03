pub mod command;
pub mod core;

pub use core::config;
pub use core::constants;
pub use core::git;
pub use core::utils;

#[macro_export]
macro_rules! strings {
    ($($s:expr),* $(,)?) => {
        vec![$($s.to_string()),*]
    };
}

#[cfg(test)]
mod tests {
    use super::strings;

    #[test]
    fn strings_macro_works_as_expected() {
        let result = strings!["hello", "world", "123"];
        let expected = vec!["hello".to_string(), "world".to_string(), "123".to_string()];
        assert_eq!(result, expected);
    }
}
