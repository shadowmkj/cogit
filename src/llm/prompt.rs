// ==============================================================================
// Prompt Construction for Conventional Commits
// ==============================================================================
//
// Formulates targeted system and user prompts instructing LLMs to generate
// clean, compliant Conventional Commit messages from staged Git diffs.

/// Returns the system prompt enforcing Conventional Commit guidelines.
pub fn get_system_prompt(detailed: bool) -> &'static str {
    if detailed {
        r#"You are an expert software developer and Git assistant. Your task is to generate a clean, accurate, and professional Conventional Commit message based on the provided staged Git diff.

Follow the Conventional Commits specification strictly:
1. Format:
   <type>(<optional scope>): <subject>

   - <bullet point 1 describing key motivation or change>
   - <bullet point 2 describing implementation detail>

2. Commit Types:
   - feat: A new feature
   - fix: A bug fix
   - docs: Documentation only changes
   - style: Code style/formatting changes that do not affect meaning
   - refactor: A code change that neither fixes a bug nor adds a feature
   - perf: A code change that improves performance
   - test: Adding missing tests or correcting existing tests
   - build: Changes affecting build system or external dependencies
   - ci: Changes to CI configuration files and scripts
   - chore: Other changes that don't modify src or test files
   - revert: Reverting a previous commit

3. Subject Rules:
   - Use the imperative, present tense: "add" not "added", "change" not "changed"
   - Lowercase the first letter of the subject
   - Do NOT end the subject line with a period
   - Maximum 72 characters for the subject line

4. Body Rules:
   - Leave exactly one blank line between the subject and the body
   - Use concise bullet points starting with '-'
   - Focus on WHY the change was made and WHAT was modified

5. Output Format:
   - Return ONLY the commit message.
   - Do NOT enclose the response in markdown code blocks or quotes.
   - Do NOT include introductory or conversational commentary."#
    } else {
        r#"You are an expert software developer and Git assistant. Your task is to generate a concise, accurate Conventional Commit message based on the provided staged Git diff.

Follow the Conventional Commits specification strictly:
1. Format:
   <type>(<optional scope>): <subject>

2. Commit Types:
   - feat: A new feature
   - fix: A bug fix
   - docs: Documentation only changes
   - style: Code style/formatting changes that do not affect meaning
   - refactor: A code change that neither fixes a bug nor adds a feature
   - perf: A code change that improves performance
   - test: Adding missing tests or correcting existing tests
   - build: Changes affecting build system or external dependencies
   - ci: Changes to CI configuration files and scripts
   - chore: Other changes that don't modify src or test files
   - revert: Reverting a previous commit

3. Rules:
   - Return ONLY a single line commit subject.
   - Use imperative, present tense: "add" not "added", "change" not "changed"
   - Lowercase the first letter after the colon
   - Do NOT end with a period
   - Maximum 72 characters

4. Output Format:
   - Return ONLY the single line commit message.
   - Do NOT enclose in markdown code blocks or quotes.
   - Do NOT include introductory or conversational commentary."#
    }
}

/// Builds the user prompt containing the staged diff, file list, and optional user instructions.
pub fn build_user_prompt(
    diff_content: &str,
    staged_files: &[String],
    custom_hint: Option<&str>,
) -> String {
    let mut prompt = String::new();

    if let Some(hint) = custom_hint
        && !hint.trim().is_empty()
    {
        prompt.push_str(&format!("User Guidance / Context:\n{}\n\n", hint.trim()));
    }

    if !staged_files.is_empty() {
        prompt.push_str("Staged Files:\n");
        for file in staged_files {
            prompt.push_str(&format!("- {}\n", file));
        }
        prompt.push('\n');
    }

    prompt.push_str("Staged Git Diff:\n");
    prompt.push_str(diff_content);

    prompt
}

/// Helper to build combined prompt request structure for commits.
pub fn build_prompt(
    diff_content: &str,
    staged_files: &[String],
    detailed: bool,
    custom_hint: Option<&str>,
) -> (String, String) {
    let system_prompt = get_system_prompt(detailed).to_string();
    let user_prompt = build_user_prompt(diff_content, staged_files, custom_hint);
    (system_prompt, user_prompt)
}

/// Constructs system and user prompts for generating a GitHub Pull Request description.
pub fn build_pr_prompt(
    commits: &[String],
    diff_content: &str,
    custom_prompt: Option<&str>,
) -> (String, String) {
    let system_prompt = r#"You are an expert software engineer and technical lead.
Your task is to generate a comprehensive, well-structured Pull Request (PR) title and Markdown description based on the provided commit history and code diff.

Follow this exact format:

# Title: <Type>(<scope>): <concise imperative summary under 72 chars>

## Summary
<2-3 concise sentences summarizing the objective and impact of these changes.>

## Key Changes
- <Bulleted list of meaningful architectural and functional changes>

## Verification & Testing
- <Bulleted list of test cases, verification steps, and automated test commands run>
"#;

    let mut user_prompt = String::new();
    user_prompt.push_str("Commit History on this branch:\n");
    if commits.is_empty() {
        user_prompt.push_str("(No commits yet - uncommitted diff provided)\n");
    } else {
        for c in commits {
            user_prompt.push_str(&format!("- {}\n", c));
        }
    }

    user_prompt.push_str("\nCode Diff:\n```diff\n");
    user_prompt.push_str(diff_content);
    user_prompt.push_str("\n```\n");

    if let Some(hint) = custom_prompt.filter(|h| !h.trim().is_empty()) {
        user_prompt.push_str(&format!(
            "\nAdditional User Instructions:\n{}\n",
            hint.trim()
        ));
    }

    (system_prompt.to_string(), user_prompt)
}

/// Constructs system and user prompts for generating clean Git branch names.
pub fn build_branch_prompt(
    diff_content: Option<&str>,
    user_hint: Option<&str>,
    branch_type: Option<&str>,
) -> (String, String) {
    let system_prompt = r#"You are a developer workflow assistant.
Generate 3 to 5 clean, standard-compliant Git branch names.
Rules:
1. Use kebab-case with standard prefixes (e.g. feat/, fix/, chore/, refactor/, docs/).
2. Keep names concise (under 40 characters), descriptive, and lower-case.
3. Return ONLY a plain list of branch names, one per line. Do not include markdown formatting or numbering.
"#;

    let mut user_prompt = String::new();
    if let Some(t) = branch_type.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Preferred Prefix / Type: {}\n", t.trim()));
    }
    if let Some(hint) = user_hint.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Task Description: {}\n", hint.trim()));
    }
    if let Some(diff) = diff_content.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Working Diff:\n```diff\n{}\n```\n", diff));
    }

    (system_prompt.to_string(), user_prompt)
}

/// Parses raw LLM response into sanitized kebab-case branch names.
pub fn parse_branch_suggestions(raw_response: &str) -> Vec<String> {
    raw_response
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let without_bullet = trimmed
                .trim_start_matches(|c: char| {
                    c.is_numeric() || c == '.' || c == '-' || c == '*' || c == ' '
                })
                .trim()
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            without_bullet.to_string()
        })
        .filter(|s| !s.is_empty() && (s.contains('/') || s.contains('-')))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_detailed_difference() {
        let standard = get_system_prompt(false);
        let detailed = get_system_prompt(true);

        assert!(standard.contains("Return ONLY a single line"));
        assert!(detailed.contains("Leave exactly one blank line"));
    }

    #[test]
    fn test_build_user_prompt_with_hint() {
        let files = vec!["src/main.rs".to_string()];
        let prompt = build_user_prompt("+println!(\"test\");", &files, Some("Added greeting"));

        assert!(prompt.contains("User Guidance / Context:\nAdded greeting"));
        assert!(prompt.contains("Staged Files:\n- src/main.rs"));
        assert!(prompt.contains("+println!(\"test\");"));
    }

    #[test]
    fn test_build_pr_prompt_structure() {
        let commits = vec!["abc1234 feat(auth): add jwt support".to_string()];
        let diff = "+ let token = generate_jwt();";
        let (sys, user) = build_pr_prompt(&commits, diff, Some("ticket #123"));

        assert!(sys.contains("Pull Request (PR) title"));
        assert!(user.contains("abc1234 feat(auth)"));
        assert!(user.contains("ticket #123"));
    }

    #[test]
    fn test_parse_branch_suggestions() {
        let raw = "1. feat/jwt-auth\n2. `feat/auth-token-generation`\n* feat/jwt-support";
        let branches = parse_branch_suggestions(raw);
        assert_eq!(branches.len(), 3);
        assert_eq!(branches[0], "feat/jwt-auth");
        assert_eq!(branches[1], "feat/auth-token-generation");
        assert_eq!(branches[2], "feat/jwt-support");
    }
}
