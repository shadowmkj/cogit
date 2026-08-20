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

/// Helper to build combined prompt request structure.
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
}
