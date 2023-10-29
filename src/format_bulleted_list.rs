use std::fmt::Display;

use itertools::Itertools;

pub fn format_bulleted_list(items: impl IntoIterator<Item = impl Display>) -> String {
    items.into_iter().map(|item| format!("• {item}")).join("\n")
}
