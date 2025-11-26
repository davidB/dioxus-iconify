use anyhow::{Result, anyhow};
use heck::ToPascalCase;

/// Represents a parsed icon identifier (collection:icon-name)
#[derive(Debug, Clone)]
pub struct IconIdentifier {
    pub collection: String,
    pub icon_name: String,
    pub full_name: String,
}

impl IconIdentifier {
    /// Parse an icon identifier from the format "collection:icon-name"
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid icon identifier format. Expected 'collection:icon-name', got '{}'",
                input
            ));
        }

        let collection = parts[0].trim().to_string();
        let icon_name = parts[1].trim().to_string();

        if collection.is_empty() || icon_name.is_empty() {
            return Err(anyhow!(
                "Both collection and icon name must be non-empty in '{}'",
                input
            ));
        }

        Ok(Self {
            collection,
            icon_name,
            full_name: input.to_string(),
        })
    }

    /// Get the module name for this collection (e.g., "mdi")
    pub fn module_name(&self) -> String {
        self.collection.replace('-', "_")
    }

    /// Convert the icon name to a valid Rust constant name (PascalCase)
    pub fn to_const_name(&self) -> String {
        // Convert to PascalCase
        let mut const_name = self.icon_name.to_pascal_case();

        // Handle leading numbers (Rust identifiers can't start with numbers)
        if const_name.chars().next().is_some_and(|c| c.is_numeric()) {
            const_name = format!("_{}", const_name);
        }

        // Check for Rust keywords and append suffix if needed
        if is_rust_keyword(&const_name) {
            const_name.push_str("Icon");
        }

        const_name
    }
}

/// Check if a string is a Rust keyword
fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_identifier() {
        let id = IconIdentifier::parse("mdi:home").unwrap();
        assert_eq!(id.collection, "mdi");
        assert_eq!(id.icon_name, "home");
        assert_eq!(id.full_name, "mdi:home");
    }

    #[test]
    fn test_parse_invalid_identifier() {
        assert!(IconIdentifier::parse("invalid").is_err());
        assert!(IconIdentifier::parse("too:many:colons").is_err());
        assert!(IconIdentifier::parse(":empty-collection").is_err());
        assert!(IconIdentifier::parse("empty-name:").is_err());
    }

    #[test]
    fn test_module_name() {
        let id = IconIdentifier::parse("mdi:home").unwrap();
        assert_eq!(id.module_name(), "mdi");

        let id = IconIdentifier::parse("simple-icons:github").unwrap();
        assert_eq!(id.module_name(), "simple_icons");
    }

    #[test]
    fn test_to_const_name() {
        let id = IconIdentifier::parse("mdi:home").unwrap();
        assert_eq!(id.to_const_name(), "Home");

        let id = IconIdentifier::parse("heroicons:arrow-left").unwrap();
        assert_eq!(id.to_const_name(), "ArrowLeft");

        let id = IconIdentifier::parse("lucide:shopping-cart").unwrap();
        assert_eq!(id.to_const_name(), "ShoppingCart");

        let id = IconIdentifier::parse("mdi:numeric-1-box").unwrap();
        assert_eq!(id.to_const_name(), "Numeric1Box");

        // Test leading number
        let id = IconIdentifier::parse("mdi:1password").unwrap();
        assert_eq!(id.to_const_name(), "_1password");

        // Test keyword
        let id = IconIdentifier::parse("mdi:type").unwrap();
        assert_eq!(id.to_const_name(), "TypeIcon");
    }
}
