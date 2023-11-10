use std::fmt::Display;

use itertools::Itertools;

pub fn format_bulleted_list(items: impl IntoIterator<Item = impl Display>) -> String {
    items.into_iter().map(|item| format!("• {item}")).join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bulleted_list() {
        assert_eq!(
            format_bulleted_list(["puppy", "dog", "city"]).to_string(),
            "• puppy\n\
            • dog\n\
            • city"
        );
    }
}
