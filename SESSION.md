# Session Notes & Future Roadmap

## Deferred Enhancements & Features

### 1. Interactive API Key Setup Wizard (`cogit --setup` / `cogit auth`)
- **Issue**: [#9](https://github.com/shadowmkj/cogit/issues/9)
- **Description**: Add an interactive CLI onboarding wizard (`inquire::Password` + `inquire::Select`) that allows users to configure and test their API keys for Gemini, OpenAI, Grok, and Groq without manually editing TOML files or setting environment variables.
