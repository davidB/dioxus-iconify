use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::api::IconifyIcon;

/// Parse a single SVG file and extract icon data
pub fn parse_svg_file(path: &Path) -> Result<IconifyIcon> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read SVG file: {}", path.display()))?;

    let doc = roxmltree::Document::parse(&content).context("Failed to parse SVG as XML")?;

    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(anyhow!("Not a valid SVG file (root element is not <svg>)"));
    }

    // Extract attributes
    let width_attr = root.attribute("width");
    let height_attr = root.attribute("height");
    let viewbox_attr = root.attribute("viewBox");

    // Parse dimension attributes, stripping units like "px", "em", etc.
    let width = width_attr.and_then(parse_dimension);
    let height = height_attr.and_then(parse_dimension);
    let view_box = viewbox_attr.map(|s| s.to_string());

    // Infer missing dimensions (following api.rs logic)
    let (final_width, final_height, final_viewbox) = infer_dimensions(width, height, view_box)?;

    // Extract SVG body (inner content only, strip <svg> wrapper)
    let body = extract_svg_body(&root)?;

    Ok(IconifyIcon {
        body,
        width: Some(final_width),
        height: Some(final_height),
        view_box: Some(final_viewbox),
    })
}

/// Extract collection name from a directory path
/// Example: "/path/to/my-icons" → "my-icons"
pub fn extract_collection_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Invalid directory name: {}", path.display()))
}

/// Scan directory recursively for SVG files and build icon names
/// Returns Vec of (svg_file_path, icon_name)
/// Example: base="my-icons/", file="arrows/left.svg" → ("my-icons/arrows/left.svg", "arrows-left")
pub fn scan_svg_directory(dir_path: &Path) -> Result<Vec<(PathBuf, String)>> {
    if !dir_path.is_dir() {
        return Err(anyhow!("Not a directory: {}", dir_path.display()));
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process .svg files
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("svg") {
            let icon_name = build_icon_name(dir_path, path)?;
            results.push((path.to_path_buf(), icon_name));
        }
    }

    if results.is_empty() {
        eprintln!("  ⚠ No SVG files found in {}", dir_path.display());
    }

    Ok(results)
}

/// Build icon name from file path relative to base directory
/// Example: base="my-icons/", path="my-icons/arrows/left.svg" → "arrows-left"
/// Example: base="my-icons/", path="my-icons/home.svg" → "home"
fn build_icon_name(base_path: &Path, svg_path: &Path) -> Result<String> {
    let relative = svg_path
        .strip_prefix(base_path)
        .context("SVG path is not under base directory")?;

    // Build icon name from path segments
    let mut parts = Vec::new();

    for component in relative.components() {
        if let Some(s) = component.as_os_str().to_str() {
            parts.push(s.to_string());
        }
    }

    // Remove the file extension from the last part
    if let Some(last) = parts.last_mut()
        && let Some(stem) = Path::new(last).file_stem().and_then(|s| s.to_str())
    {
        *last = stem.to_string();
    }

    // Join parts with hyphens
    let icon_name = parts.join("-");

    if icon_name.is_empty() {
        return Err(anyhow!(
            "Failed to build icon name from path: {}",
            svg_path.display()
        ));
    }

    Ok(icon_name)
}

/// Parse a dimension attribute, stripping units
/// Examples: "24" → Some(24), "24px" → Some(24), "1.5em" → None, "100%" → None
fn parse_dimension(attr: &str) -> Option<u32> {
    // Try to parse as integer first
    if let Ok(val) = attr.parse::<u32>() {
        return Some(val);
    }

    // Percentages are not supported
    let trimmed = attr.trim();
    if trimmed.ends_with('%') {
        return None;
    }

    // Try to strip common units and parse
    for unit in &["px", "pt", "em", "rem", "vh", "vw"] {
        if let Some(num_str) = trimmed.strip_suffix(unit)
            && let Ok(val) = num_str.trim().parse::<f64>()
        {
            // Only accept integer values
            if val.fract() == 0.0 && val > 0.0 {
                return Some(val as u32);
            }
        }
    }

    None
}

/// Infer missing dimensions using API logic (api.rs:166-174)
fn infer_dimensions(
    width: Option<u32>,
    height: Option<u32>,
    view_box: Option<String>,
) -> Result<(u32, u32, String)> {
    match (width, height, view_box) {
        // All present
        (Some(w), Some(h), Some(vb)) => Ok((w, h, vb)),

        // Only width and height
        (Some(w), Some(h), None) => Ok((w, h, format!("0 0 {} {}", w, h))),

        // Only viewBox - parse it to get dimensions
        (None, None, Some(vb)) => {
            let dims = parse_viewbox(&vb)?;
            Ok((dims.2, dims.3, vb))
        }

        // Only width - use for both
        (Some(w), None, None) => Ok((w, w, format!("0 0 {} {}", w, w))),

        // Only height - use for both
        (None, Some(h), None) => Ok((h, h, format!("0 0 {} {}", h, h))),

        // Width and viewBox
        (Some(w), None, Some(vb)) => {
            let dims = parse_viewbox(&vb)?;
            Ok((w, dims.3, vb))
        }

        // Height and viewBox
        (None, Some(h), Some(vb)) => {
            let dims = parse_viewbox(&vb)?;
            Ok((dims.2, h, vb))
        }

        // Nothing - use default 24x24
        (None, None, None) => {
            eprintln!("  ⚠ No dimensions found, using default 24x24");
            Ok((24, 24, "0 0 24 24".to_string()))
        }
    }
}

/// Parse viewBox attribute to extract dimensions
/// Format: "minX minY width height"
fn parse_viewbox(viewbox: &str) -> Result<(u32, u32, u32, u32)> {
    let parts: Vec<&str> = viewbox.split_whitespace().collect();

    if parts.len() != 4 {
        return Err(anyhow!(
            "Invalid viewBox format: expected 4 numbers, got {}",
            parts.len()
        ));
    }

    let min_x = parts[0].parse::<f64>().context("Invalid viewBox minX")?;
    let min_y = parts[1].parse::<f64>().context("Invalid viewBox minY")?;
    let width = parts[2].parse::<f64>().context("Invalid viewBox width")?;
    let height = parts[3].parse::<f64>().context("Invalid viewBox height")?;

    // Convert to u32, rounding if necessary
    Ok((
        min_x.round() as u32,
        min_y.round() as u32,
        width.round() as u32,
        height.round() as u32,
    ))
}

/// Extract inner content from SVG element (strip <svg> wrapper)
fn extract_svg_body(svg_element: &roxmltree::Node) -> Result<String> {
    let mut body_parts = Vec::new();

    for child in svg_element.children() {
        if let Some(xml) = node_to_xml(&child) {
            body_parts.push(xml);
        }
    }

    let body = body_parts.join("");

    if body.trim().is_empty() {
        eprintln!("  ⚠ SVG has no visible content");
    }

    Ok(body)
}

/// Convert XML node to string representation
fn node_to_xml(node: &roxmltree::Node) -> Option<String> {
    match node.node_type() {
        roxmltree::NodeType::Element => {
            let tag_name = node.tag_name().name();
            let mut xml = format!("<{}", tag_name);

            // Add attributes
            for attr in node.attributes() {
                xml.push_str(&format!(
                    " {}=\"{}\"",
                    attr.name(),
                    escape_xml(attr.value())
                ));
            }

            // Check if element has children
            if node.has_children()
                && node
                    .children()
                    .any(|c| !c.is_text() || !c.text().unwrap_or("").trim().is_empty())
            {
                xml.push('>');

                // Add children
                for child in node.children() {
                    if let Some(child_xml) = node_to_xml(&child) {
                        xml.push_str(&child_xml);
                    }
                }

                xml.push_str(&format!("</{}>", tag_name));
            } else {
                // Self-closing tag
                xml.push_str("/>");
            }

            Some(xml)
        }
        roxmltree::NodeType::Text => {
            let text = node.text()?;
            if !text.trim().is_empty() {
                Some(escape_xml(text))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_dimension() {
        assert_eq!(parse_dimension("24"), Some(24));
        assert_eq!(parse_dimension("100"), Some(100));
        assert_eq!(parse_dimension("24px"), Some(24));
        assert_eq!(parse_dimension("16pt"), Some(16));
        assert_eq!(parse_dimension("1.5em"), None); // Non-integer
        assert_eq!(parse_dimension("100%"), None); // Percentage not supported
        assert_eq!(parse_dimension("invalid"), None);
    }

    #[test]
    fn test_parse_viewbox() {
        let result = parse_viewbox("0 0 24 24").unwrap();
        assert_eq!(result, (0, 0, 24, 24));

        let result = parse_viewbox("0 0 100 50").unwrap();
        assert_eq!(result, (0, 0, 100, 50));

        assert!(parse_viewbox("invalid").is_err());
        assert!(parse_viewbox("0 0 24").is_err()); // Only 3 values
    }

    #[test]
    fn test_infer_dimensions_all_present() {
        let (w, h, vb) =
            infer_dimensions(Some(24), Some(24), Some("0 0 24 24".to_string())).unwrap();

        assert_eq!(w, 24);
        assert_eq!(h, 24);
        assert_eq!(vb, "0 0 24 24");
    }

    #[test]
    fn test_infer_dimensions_only_width_height() {
        let (w, h, vb) = infer_dimensions(Some(32), Some(32), None).unwrap();

        assert_eq!(w, 32);
        assert_eq!(h, 32);
        assert_eq!(vb, "0 0 32 32");
    }

    #[test]
    fn test_infer_dimensions_only_viewbox() {
        let (w, h, vb) = infer_dimensions(None, None, Some("0 0 48 48".to_string())).unwrap();

        assert_eq!(w, 48);
        assert_eq!(h, 48);
        assert_eq!(vb, "0 0 48 48");
    }

    #[test]
    fn test_infer_dimensions_defaults() {
        let (w, h, vb) = infer_dimensions(None, None, None).unwrap();

        assert_eq!(w, 24);
        assert_eq!(h, 24);
        assert_eq!(vb, "0 0 24 24");
    }

    #[test]
    fn test_build_icon_name() {
        let base = Path::new("/tmp/icons");

        let svg1 = Path::new("/tmp/icons/home.svg");
        assert_eq!(build_icon_name(base, svg1).unwrap(), "home");

        let svg2 = Path::new("/tmp/icons/arrows/left.svg");
        assert_eq!(build_icon_name(base, svg2).unwrap(), "arrows-left");

        let svg3 = Path::new("/tmp/icons/ui/buttons/primary.svg");
        assert_eq!(build_icon_name(base, svg3).unwrap(), "ui-buttons-primary");
    }

    #[test]
    fn test_extract_collection_name() {
        let path = Path::new("/tmp/my-icons");
        assert_eq!(extract_collection_name(path).unwrap(), "my-icons");

        let path2 = Path::new("./custom-icons");
        assert_eq!(extract_collection_name(path2).unwrap(), "custom-icons");
    }

    #[test]
    fn test_parse_svg_with_all_attributes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let svg_path = temp_dir.path().join("test.svg");

        let mut file = fs::File::create(&svg_path)?;
        write!(
            file,
            r#"<svg width="24" height="24" viewBox="0 0 24 24"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>"#
        )?;

        let icon = parse_svg_file(&svg_path)?;

        assert_eq!(icon.width, Some(24));
        assert_eq!(icon.height, Some(24));
        assert_eq!(icon.view_box, Some("0 0 24 24".to_string()));
        assert!(icon.body.contains("path"));
        assert!(!icon.body.contains("<svg")); // Should not include svg wrapper

        Ok(())
    }

    #[test]
    fn test_parse_svg_with_viewbox_only() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let svg_path = temp_dir.path().join("test.svg");

        let mut file = fs::File::create(&svg_path)?;
        write!(
            file,
            r#"<svg viewBox="0 0 48 48"><circle cx="24" cy="24" r="20"/></svg>"#
        )?;

        let icon = parse_svg_file(&svg_path)?;

        assert_eq!(icon.width, Some(48));
        assert_eq!(icon.height, Some(48));
        assert_eq!(icon.view_box, Some("0 0 48 48".to_string()));
        assert!(icon.body.contains("circle"));

        Ok(())
    }

    #[test]
    fn test_parse_svg_no_dimensions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let svg_path = temp_dir.path().join("test.svg");

        let mut file = fs::File::create(&svg_path)?;
        write!(
            file,
            r#"<svg><rect x="0" y="0" width="10" height="10"/></svg>"#
        )?;

        let icon = parse_svg_file(&svg_path)?;

        // Should default to 24x24
        assert_eq!(icon.width, Some(24));
        assert_eq!(icon.height, Some(24));
        assert_eq!(icon.view_box, Some("0 0 24 24".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_invalid_xml() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("test.svg");

        let mut file = fs::File::create(&svg_path).unwrap();
        write!(file, r#"<svg><path d="invalid"#).unwrap(); // Unclosed tag

        assert!(parse_svg_file(&svg_path).is_err());
    }

    #[test]
    fn test_scan_directory_recursive() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create directory structure
        fs::create_dir(temp_dir.path().join("arrows"))?;

        // Create SVG files
        fs::File::create(temp_dir.path().join("home.svg"))?;
        fs::File::create(temp_dir.path().join("arrows/left.svg"))?;
        fs::File::create(temp_dir.path().join("arrows/right.svg"))?;

        let results = scan_svg_directory(temp_dir.path())?;

        assert_eq!(results.len(), 3);

        let names: Vec<String> = results.iter().map(|(_, name)| name.clone()).collect();
        assert!(names.contains(&"home".to_string()));
        assert!(names.contains(&"arrows-left".to_string()));
        assert!(names.contains(&"arrows-right".to_string()));

        Ok(())
    }

    #[rstest]
    #[case("tests/fixtures/test-icons/simple.svg", 24, 24, "0 0 24 24")]
    #[case("tests/fixtures/test-icons/viewbox-only.svg", 48, 48, "0 0 48 48")]
    #[case("tests/fixtures/test-icons/no-dimensions.svg", 24, 24, "0 0 24 24")]
    fn test_parse_svg_fixtures(
        #[case] path: &str,
        #[case] expected_width: u32,
        #[case] expected_height: u32,
        #[case] expected_viewbox: &str,
    ) -> Result<()> {
        let icon = parse_svg_file(Path::new(path))?;

        assert_eq!(icon.width, Some(expected_width));
        assert_eq!(icon.height, Some(expected_height));
        assert_eq!(icon.view_box, Some(expected_viewbox.to_string()));
        assert!(!icon.body.is_empty());
        assert!(!icon.body.contains("<svg")); // Body should not include svg wrapper

        Ok(())
    }

    #[test]
    fn test_scan_fixtures_directory() -> Result<()> {
        let results = scan_svg_directory(Path::new("tests/fixtures/test-icons"))?;

        assert_eq!(results.len(), 5); // simple, viewbox-only, no-dimensions, arrows/left, arrows/right

        let names: Vec<String> = results.iter().map(|(_, name)| name.clone()).collect();
        assert!(names.contains(&"simple".to_string()));
        assert!(names.contains(&"viewbox-only".to_string()));
        assert!(names.contains(&"no-dimensions".to_string()));
        assert!(names.contains(&"arrows-left".to_string()));
        assert!(names.contains(&"arrows-right".to_string()));

        Ok(())
    }
}
