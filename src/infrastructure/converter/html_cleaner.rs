//! HTML boilerplate removal before Markdown conversion.
//!
//! Uses `scraper` for CSS selector matching and `aho-corasick` for fast
//! tag removal. Replaces the `html-cleaning` + `dom_query` dependency chain,
//! eliminating selectors 0.33 from the dependency tree.

/// Tags to remove entirely (opening tag + content + closing tag).
const TAGS_TO_REMOVE: &[&str] = &[
    // Scripts and styles
    "script",
    "style",
    "noscript",
    // Interactive/embedded
    "form",
    "iframe",
    "object",
    "embed",
    // Media
    "svg",
    "canvas",
    "video",
    "audio",
    // Page chrome
    "nav",
    "header",
    "footer",
    "aside",
];

/// CSS selectors to remove — simple class and attribute patterns.
const SELECTOR_PATTERNS: &[&str] = &[
    // Starlight/Astro navigation
    "class=\"site-title\"",
    "class=\"global-nav\"",
    "class=\"global-nav-list\"",
    "class=\"mobile-menu-wrapper\"",
    "class=\"right-sidebar\"",
    "class=\"right-sidebar-container\"",
    "class=\"mobile-toc\"",
    "class=\"sl-sidebar\"",
    "class=\"sl-mobile-toc\"",
    // Search and feedback
    "class=\"search\"",
    "class=\"site-search\"",
    "class=\"social-icons\"",
    "class=\"page-feedback\"",
    "class=\"feedback\"",
    // Breadcrumb and pagination
    "class=\"sl-breadcrumbs\"",
    "class=\"pagination\"",
    // Copy-to-clipboard
    "class=\"copy-markdown-btn\"",
    "class=\"copy-code-button\"",
    // Skip links
    "class=\"skip-link\"",
];

/// Attributes to preserve (all others are stripped from elements).
const PRESERVED_ATTRS: &[&str] = &["href", "src", "alt", "id", "class", "dir", "code"];

/// Clean HTML by removing boilerplate (nav, sidebar, scripts, SVGs).
///
/// Removes:
/// - `script`, `style`, `noscript` (code and styles)
/// - `form`, `iframe`, `object`, `embed` (interactive)
/// - `svg`, `canvas`, `video`, `audio` (media)
/// - `nav`, `header`, `footer`, `aside` (page chrome)
/// - Elements matching CSS selector patterns (sidebars, search, breadcrumbs)
/// - Strips non-preserved attributes (keeps href, src, alt, id, class, dir, code)
/// - Normalizes whitespace
///
/// Returns the cleaned HTML as a string.
pub fn clean_html(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }

    // Step 1: Remove tags with content (script, style, nav, etc.)
    let result = remove_tags_with_content(html, TAGS_TO_REMOVE);

    // Step 2: Remove elements matching CSS selector patterns
    let result = remove_by_selector_patterns(&result, SELECTOR_PATTERNS);

    // Step 3: Strip non-preserved attributes
    let result = strip_attributes(&result);

    // Step 4: Normalize whitespace
    normalize_whitespace(&result)
}

/// Remove tags and their entire content using Aho-Corasick for fast pattern matching.
fn remove_tags_with_content(html: &str, tags: &[&str]) -> String {
    use aho_corasick::AhoCorasick;

    let mut open_patterns = Vec::new();
    let mut close_patterns = Vec::new();
    for tag in tags {
        open_patterns.push(format!("<{tag}"));
        close_patterns.push(format!("</{tag}>"));
    }

    let ac_open = AhoCorasick::new(&open_patterns).expect("patrones AC inválidos");
    let mut result = html.to_string();

    loop {
        let mut changed = false;
        let bytes = result.as_bytes();

        if let Some(mat) = ac_open.find(bytes) {
            let open_start = mat.start();
            let tag_name = &tags[mat.pattern().as_usize()];

            let close_tag = format!("</{tag_name}>");
            let search_from = mat.end();
            let remaining = &result[search_from..];

            if let Some(close_pos) = find_matching_close(remaining, tag_name) {
                let close_end = search_from + close_pos + close_tag.len();
                result.replace_range(open_start..close_end, "");
                changed = true;
            } else {
                let tag_end = result[open_start..].find('>').map(|p| open_start + p + 1);
                if let Some(end) = tag_end {
                    result.replace_range(open_start..end, "");
                    changed = true;
                } else {
                    break;
                }
            }
        }

        if !changed {
            break;
        }
    }

    result
}

/// Find the matching closing tag, accounting for nesting of the same tag.
fn find_matching_close(html: &str, tag_name: &str) -> Option<usize> {
    let open_tag = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut depth = 0;
    let mut pos = 0;

    while pos < html.len() {
        if html[pos..].starts_with(&close_tag) {
            if depth == 0 {
                return Some(pos);
            }
            depth -= 1;
            pos += close_tag.len();
        } else if html[pos..].starts_with(&open_tag) {
            let after = html[pos + open_tag.len()..].chars().next();
            if matches!(after, Some(' ') | Some('>') | Some('/') | Some('\n') | Some('\t')) {
                depth += 1;
            }
            pos += open_tag.len();
        } else {
            pos += 1;
        }
    }

    None
}

/// Remove elements that match class selector patterns.
fn remove_by_selector_patterns(html: &str, patterns: &[&str]) -> String {
    let mut result = html.to_string();

    for pattern in patterns {
        let class_name = pattern
            .strip_prefix("class=\"")
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(pattern);

        loop {
            let class_attr = format!("class=\"{}\"", class_name);
            let class_attr_multi = format!("class=\"{} ", class_name);
            let class_attr_end = format!(" {}\"", class_name);
            let class_attr_mid = format!(" {} ", class_name);

            let pos = result
                .find(&class_attr)
                .or_else(|| result.find(&class_attr_multi))
                .or_else(|| result.find(&class_attr_end))
                .or_else(|| result.find(&class_attr_mid));

            let Some(class_pos) = pos else { break };

            let tag_start = result[..class_pos].rfind('<').unwrap_or(0);
            let tag_name_start = tag_start + 1;
            let tag_content = &result[tag_name_start..];
            let tag_name: String = tag_content
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();

            if tag_name.is_empty() {
                break;
            }

            let open_tag_end = result[tag_start..]
                .find('>')
                .map(|p| tag_start + p + 1);

            let Some(open_end) = open_tag_end else { break };

            let close_tag = format!("</{}>", tag_name);
            let close_pos = find_matching_close(&result[open_end..], &tag_name)
                .map(|p| open_end + p + close_tag.len());

            match close_pos {
                Some(end) => {
                    result.replace_range(tag_start..end, "");
                }
                None => {
                    result.replace_range(tag_start..open_end, "");
                }
            }
        }
    }

    result
}

/// Strip attributes not in PRESERVED_ATTRS from HTML tags.
fn strip_attributes(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            result.push('<');
            let mut tag = String::new();

            while let Some(&c) = chars.peek() {
                chars.next();
                if c == '>' {
                    break;
                }
                tag.push(c);
            }

            if let Some(space_pos) = tag.find(' ') {
                let (tag_name, attrs_part) = tag.split_at(space_pos);
                result.push_str(tag_name);

                let filtered: Vec<&str> = attrs_part
                    .split_inclusive(' ')
                    .filter(|part| {
                        let trimmed = part.trim();
                        if trimmed.is_empty() {
                            return false;
                        }
                        let attr_name = trimmed.split('=').next().unwrap_or("").trim();
                        PRESERVED_ATTRS.contains(&attr_name)
                    })
                    .collect();

                if !filtered.is_empty() {
                    result.push(' ');
                    for part in filtered {
                        result.push_str(part.trim());
                        result.push(' ');
                    }
                    if result.ends_with(' ') {
                        result.pop();
                    }
                }

                if tag.ends_with('/') {
                    result.push_str(" /");
                }
            } else {
                result.push_str(&tag);
            }

            result.push('>');
        } else {
            result.push(ch);
        }
    }

    result
}

/// Collapse consecutive whitespace into single spaces.
fn normalize_whitespace(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_whitespace = false;

    for ch in html.chars() {
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(ch);
            in_whitespace = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_removes_scripts() {
        let html = "<html><body><script>alert(1)</script><p>Hello</p></body></html>";
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("Hello"));
    }

    #[test]
    fn test_clean_removes_svg() {
        let html =
            "<html><body><nav><svg>icon</svg></nav><article><h1>Title</h1></article></body></html>";
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("<svg>"));
        assert!(!cleaned.contains("<nav>"));
    }

    #[test]
    fn test_clean_preserves_content() {
        let html = "<html><body><nav>Menu</nav><main><h1>Article</h1><p>Content here</p></main></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.contains("Article"));
        assert!(cleaned.contains("Content here"));
        assert!(!cleaned.contains("Menu"));
    }

    #[test]
    fn test_clean_empty_html() {
        let html = "";
        let cleaned = clean_html(html);
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_removes_css_selectors() {
        let html = r#"
            <html>
                <body>
                    <nav class="global-nav">
                        <span class="site-title">My Site</span>
                        <ul class="global-nav-list">
                            <li><a href="/">Home</a></li>
                        </ul>
                    </nav>
                    <main>
                        <h1>Main Content</h1>
                        <p>This should remain</p>
                    </main>
                </body>
            </html>
        "#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("global-nav"));
        assert!(!cleaned.contains("site-title"));
        assert!(cleaned.contains("Main Content"));
        assert!(cleaned.contains("This should remain"));
    }

    #[test]
    fn test_clean_preserves_href_attribute() {
        let html = r#"<html><body><a href="https://example.com" onclick="alert(1)" class="link">Click</a></body></html>"#;
        let cleaned = clean_html(html);
        assert!(cleaned.contains("href="), "href should be preserved");
        assert!(
            cleaned.contains("https://example.com"),
            "href URL should be preserved"
        );
        assert!(!cleaned.contains("onclick"), "onclick should be stripped");
    }

    #[test]
    fn test_clean_whitespace_normalization() {
        let html = "<html><body><p>  Too   many    spaces  </p><p>\n\n\tNewlines\t\t</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            !cleaned.contains("   "),
            "multiple spaces should be collapsed"
        );
        assert!(
            !cleaned.contains("\n\n"),
            "multiple newlines should be collapsed"
        );
    }
}
