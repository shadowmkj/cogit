// ==============================================================================
// LLM Output Sanitization
// ==============================================================================
//
// Cleans and normalizes raw text returned by LLMs to ensure pure conventional
// commit messages without markdown code fences or conversational preamble.

/// Strips markdown code blocks, conversational introductions, and surrounding quotes.
pub fn clean_commit_message(raw_output: &str) -> String {
    let mut cleaned = raw_output.trim();

    // 1. Remove markdown code fences (``` or ```git / ```commit)
    if cleaned.starts_with("```") {
        if let Some(first_newline) = cleaned.find('\n') {
            cleaned = cleaned[first_newline + 1..].trim();
        } else {
            cleaned = cleaned.trim_start_matches('`').trim();
        }

        if cleaned.ends_with("```") {
            cleaned = cleaned.trim_end_matches('`').trim();
        }
    }

    // 2. Remove common preamble phrases
    let common_preambles = [
        "here is the commit message:",
        "here is your commit message:",
        "commit message:",
        "here is a suggested commit message:",
        "suggested commit message:",
    ];

    for preamble in &common_preambles {
        if cleaned.to_lowercase().starts_with(preamble) {
            cleaned = cleaned[preamble.len()..].trim();
            cleaned = cleaned.trim_start_matches([':', '-']).trim();
            break;
        }
    }

    // 3. Remove enclosing quotes if the entire message is quoted
    if ((cleaned.starts_with('"') && cleaned.ends_with('"'))
        || (cleaned.starts_with('\'') && cleaned.ends_with('\'')))
        && cleaned.len() >= 2
    {
        cleaned = &cleaned[1..cleaned.len() - 1];
    }

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_raw_message() {
        let raw = "feat(git): add diff truncation logic";
        assert_eq!(
            clean_commit_message(raw),
            "feat(git): add diff truncation logic"
        );
    }

    #[test]
    fn test_clean_markdown_fence() {
        let raw = "```git\nfeat(cli): support detailed mode\n```";
        assert_eq!(
            clean_commit_message(raw),
            "feat(cli): support detailed mode"
        );
    }

    #[test]
    fn test_clean_preamble_and_quotes() {
        let raw = "Here is the commit message:\n\"fix(tui): handle window resize event\"";
        assert_eq!(
            clean_commit_message(raw),
            "fix(tui): handle window resize event"
        );
    }

    #[test]
    fn test_clean_multiline_detailed_message() {
        let raw = "```\nrefactor(config): simplify env expansion\n\n- Use replace_range for inline interpolation\n- Add unit tests for edge cases\n```";
        let expected = "refactor(config): simplify env expansion\n\n- Use replace_range for inline interpolation\n- Add unit tests for edge cases";
        assert_eq!(clean_commit_message(raw), expected);
    }
}
