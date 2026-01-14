use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn format_memo_line(display_time: &str, content: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let prefix = format!("{}  ", display_time);
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    let clean_content = sanitize_content(content);
    if max_width <= prefix_width {
        return truncate_with_ellipsis(display_time, max_width);
    }

    let content_width = max_width.saturating_sub(prefix_width);
    let truncated = truncate_with_ellipsis(&clean_content, content_width);
    format!("{}{}", prefix, truncated)
}

fn sanitize_content(content: &str) -> String {
    content
        .replace(['\n', '\r', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_with_ellipsis(value: &str, max_width: usize) -> String {
    let value_width = UnicodeWidthStr::width(value);
    if value_width <= max_width {
        return value.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut current_width = 0;
    let mut result = String::new();
    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
        if current_width + ch_width > max_width - 3 {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result.push_str("...");
    result
}
