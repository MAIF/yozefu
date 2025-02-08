use serde::Deserialize;

/// Shortcuts are keyboards shortcuts available to the user to interact with the UI. For instance:
/// ```ignore
/// let shortcut = Shortcut::new("CTRL + P", "Show details");
/// ```
/// will be visible as `[CTRL + P]: Show details` in the footer.

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}

impl Shortcut {
    pub fn new(key: &'static str, description: &'static str) -> Self {
        Self { key, description }
    }
}
