//! HTML sanitization functions for cleaning PR comment bodies.

use std::borrow::Cow;

/// Strips HTML tags and comments from a string, preserving the text content.
///
/// This function:
/// - Removes HTML comments (<!-- ... -->)
/// - Removes HTML tags (<tag>, </tag>, <tag />)
/// - Preserves all text content between tags
/// - Collapses excessive blank lines (3+ consecutive newlines become 2)
///
/// # Examples
/// ```
/// use pr_comments::sanitizer::strip_html;
///
/// let html = "<details><summary>Click</summary>Content</details>";
/// assert_eq!(strip_html(html), "ClickContent");
///
/// let comment = "<!-- hidden -->Visible";
/// assert_eq!(strip_html(comment), "Visible");
/// ```
pub fn strip_html(input: &str) -> Cow<'_, str> {
    // Quick check: if there's no < character, nothing to strip
    if !input.contains('<') {
        return Cow::Borrowed(input);
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Check if this is an HTML comment
            if chars.peek() == Some(&'!') {
                let lookahead: String = chars.clone().take(3).collect();
                if lookahead.starts_with("!--") {
                    // Skip HTML comment: <!-- ... -->
                    // Consume the "!--"
                    chars.next(); // !
                    chars.next(); // -
                    chars.next(); // -

                    // Find the closing -->
                    let mut prev_prev = ' ';
                    let mut prev = ' ';
                    for ch in chars.by_ref() {
                        if prev_prev == '-' && prev == '-' && ch == '>' {
                            break;
                        }
                        prev_prev = prev;
                        prev = ch;
                    }
                    continue;
                }
            }

            // Regular HTML tag: skip until >
            for ch in chars.by_ref() {
                if ch == '>' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    // Collapse excessive blank lines (3+ newlines -> 2 newlines)
    let result = collapse_blank_lines(&result);

    Cow::Owned(result)
}

/// Collapses 3 or more consecutive newlines into 2 newlines.
fn collapse_blank_lines(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut newline_count = 0;

    for c in input.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(c);
            }
        } else {
            newline_count = 0;
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_html() {
        let input = "Plain text with no HTML";
        assert_eq!(strip_html(input), input);
    }

    #[test]
    fn test_simple_tags() {
        let input = "<p>Hello</p>";
        assert_eq!(strip_html(input), "Hello");
    }

    #[test]
    fn test_nested_tags() {
        let input = "<div><p>Hello <strong>World</strong></p></div>";
        assert_eq!(strip_html(input), "Hello World");
    }

    #[test]
    fn test_self_closing_tags() {
        let input = "Line 1<br/>Line 2";
        assert_eq!(strip_html(input), "Line 1Line 2");
    }

    #[test]
    fn test_html_comment() {
        let input = "Before<!-- comment -->After";
        assert_eq!(strip_html(input), "BeforeAfter");
    }

    #[test]
    fn test_html_comment_multiline() {
        let input = "Before<!-- \nmultiline\ncomment\n -->After";
        assert_eq!(strip_html(input), "BeforeAfter");
    }

    #[test]
    fn test_devin_review_comment() {
        let input = r#"<!-- devin-review-comment {"id": "BUG_123", "file_path": "test.py"} -->

🔴 **Bug found**"#;
        let expected = "\n\n🔴 **Bug found**";
        assert_eq!(strip_html(input), expected);
    }

    #[test]
    fn test_details_summary() {
        let input = "<details>\n<summary>Click to expand</summary>\n\nContent here\n\n</details>";
        let result = strip_html(input);
        assert!(!result.contains("<details>"));
        assert!(!result.contains("<summary>"));
        assert!(result.contains("Click to expand"));
        assert!(result.contains("Content here"));
    }

    #[test]
    fn test_devin_badge() {
        let input = r#"<!-- devin-review-badge-begin -->
<a href="https://app.devin.ai/review/rokt/canal/pull/14777">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://static.devin.ai/assets/gh-open-in-devin-review-dark.svg?v=1">
    <img src="https://static.devin.ai/assets/gh-open-in-devin-review-light.svg?v=1" alt="Open in Devin Review">
  </picture>
</a>
<!-- devin-review-badge-end -->"#;
        let result = strip_html(input);
        assert!(!result.contains("<a"));
        assert!(!result.contains("<picture>"));
        assert!(!result.contains("<source"));
        assert!(!result.contains("<img"));
        // The result should be mostly whitespace
        assert!(result.trim().is_empty() || result.trim().len() < 20);
    }

    #[test]
    fn test_collapse_blank_lines() {
        let input = "Line 1\n\n\n\n\nLine 2";
        let result = collapse_blank_lines(input);
        assert_eq!(result, "Line 1\n\nLine 2");
    }

    #[test]
    fn test_complex_devin_comment() {
        let input = r#"<!-- devin-review-comment {"id": "BUG_001"} -->

🔴 **QuerySet re-evaluation causes bulk_update to update unmodified objects**

The function has a bug.

<details>
<summary>Click to expand</summary>

### Root Cause
The queryset is re-evaluated.

### Code Flow
```python
for cache_entry in unlinked_caches:
    cache_entry.attribute_id = attr_id
```

</details>

**Recommendation:** Fix the code.

<!-- devin-review-badge-begin -->
<a href="https://app.devin.ai">
  <img src="badge.svg" alt="Badge">
</a>
<!-- devin-review-badge-end -->

---
*Was this helpful?*"#;

        let result = strip_html(input);

        // Should preserve meaningful content
        assert!(result.contains("🔴 **QuerySet re-evaluation"));
        assert!(result.contains("The function has a bug."));
        assert!(result.contains("Click to expand"));
        assert!(result.contains("### Root Cause"));
        assert!(result.contains("```python"));
        assert!(result.contains("**Recommendation:**"));
        assert!(result.contains("*Was this helpful?*"));

        // Should not contain HTML
        assert!(!result.contains("<details>"));
        assert!(!result.contains("<summary>"));
        assert!(!result.contains("</details>"));
        assert!(!result.contains("<a href="));
        assert!(!result.contains("<img"));
        assert!(!result.contains("<!-- devin"));
    }

    #[test]
    fn test_preserves_markdown() {
        let input = "**Bold** and *italic* and `code` and [link](url)";
        assert_eq!(strip_html(input), input);
    }

    #[test]
    fn test_preserves_code_blocks() {
        let input = "```python\nprint('hello')\n```";
        assert_eq!(strip_html(input), input);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn test_tag_with_attributes() {
        let input = r#"<a href="https://example.com" target="_blank">Link</a>"#;
        assert_eq!(strip_html(input), "Link");
    }

    #[test]
    fn test_mixed_content() {
        let input = "Normal text <strong>bold</strong> more text <!-- hidden --> end";
        assert_eq!(strip_html(input), "Normal text bold more text  end");
    }
}
