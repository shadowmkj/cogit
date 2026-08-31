## [v0.1.1](https://github.com/shadowmkj/cogit/releases/tag/v0.1.1) - 2026-08-31

### 🚀 Features

- *(cli)* Add -e/--edit flag for direct external editor workflow ([#10](https://github.com/shadowmkj/cogit/pull/10)) - ([880a574](https://github.com/shadowmkj/cogit/commit/880a5746041427c2f5b02fe5795da6d3760f597c))
- *(cli)* Add Cli parser with hook, pr, and branch subcommands - ([6dcf3ff](https://github.com/shadowmkj/cogit/commit/6dcf3ff42ec45aef4779b568c520cab91d7dc3d6))
- *(git)* Add hooks manager and branch context bridge - ([3a20c16](https://github.com/shadowmkj/cogit/commit/3a20c16c5859ea01da8696da1a8a3625b746d63c))
- *(llm)* Add prompt builders for PR descriptions and branch names - ([6cabe1c](https://github.com/shadowmkj/cogit/commit/6cabe1cf6ab4dacb590f599e728a520d87ebc567))
- *(ui)* Add clipboard copy, GitHub CLI launcher, and branch picker - ([107426b](https://github.com/shadowmkj/cogit/commit/107426bc154ebfda0de249cf4da6e6536dc05d24))
- *(workflows)* Modularize workflow coordinators and main entry point - ([37668bf](https://github.com/shadowmkj/cogit/commit/37668bf4d48ad131eb0e8baee5c2ee8e938ae57d))

### 🐛 Bug Fixes

- *(ci)* Format tag ref with refs/tags/ prefix in upload-rust-binary-action - ([203aa3d](https://github.com/shadowmkj/cogit/commit/203aa3d7ec93020eb3a5322e737044c09ca5b255))

### 👷 CI

- *(actions)* Bump actions/checkout from 4 to 7 ([#13](https://github.com/shadowmkj/cogit/pull/13)) - ([7963ab9](https://github.com/shadowmkj/cogit/commit/7963ab9af71343121c19f438f9d200174eeb8257))
- *(actions)* Bump softprops/action-gh-release from 2 to 3 ([#14](https://github.com/shadowmkj/cogit/pull/14)) - ([867593f](https://github.com/shadowmkj/cogit/commit/867593fa97ddba5c3fa63d81a6b8e808502d9954))
- *(actions)* Bump codecov/codecov-action from 5 to 7 ([#12](https://github.com/shadowmkj/cogit/pull/12)) - ([e75f164](https://github.com/shadowmkj/cogit/commit/e75f164c5d6c34635b831067fc8045db45864d9f))

### ⚙️ Misc

- Update copyright holder in LICENSE - ([bdc8b1e](https://github.com/shadowmkj/cogit/commit/bdc8b1ea3991d45997abbdaabf02ed791dc53f11))
- Update documentation and add workflow automation specification - ([c8ecb77](https://github.com/shadowmkj/cogit/commit/c8ecb770a7ed4368cc548d2a9afe3a52857ee06f))
- Remove implementation plan and todo task lists - ([019fffa](https://github.com/shadowmkj/cogit/commit/019fffa8a503e4b6381858b4c39f9f2bb3d5983e))
## [v0.1.0](https://github.com/shadowmkj/cogit/releases/tag/v0.1.0) - 2026-08-20

### 🚀 Features

- *(core)* Initialize Rust project structure with CLI and Git modules - ([08f7687](https://github.com/shadowmkj/cogit/commit/08f76874758d5a4def0bd7b4977c1763e58caedf))
- *(core)* Implement multi-provider LLM commit generator and interactive CLI ([#5](https://github.com/shadowmkj/cogit/pull/5)) - ([1fafb95](https://github.com/shadowmkj/cogit/commit/1fafb95a77683d4c2df7754be33962ace1efad3d))
- *(llm)* Implement multi-provider LLM support and config management - ([52ec116](https://github.com/shadowmkj/cogit/commit/52ec1168f9439b3db54b2c9e120dd645c3b6221c))
- *(ui)* Add interactive CLI prompt and editor integration for commits - ([ce12574](https://github.com/shadowmkj/cogit/commit/ce125741e79d561f9d1bd611043a2b02132b5661))
- *(ui)* Implement ratatui-based TUI mode and layout - ([5dc6bf1](https://github.com/shadowmkj/cogit/commit/5dc6bf14ffb1a87ccc973c86cff7f67e3098520d))
- *(llm)* Add GROQ and xAI provider support ([#8](https://github.com/shadowmkj/cogit/pull/8)) - ([5760a95](https://github.com/shadowmkj/cogit/commit/5760a9532acb156aaaeedae021e4da8e0bb63c52))
- *(ci)* Add CI/CD release workflow and installation scripts - ([c119da6](https://github.com/shadowmkj/cogit/commit/c119da662ddfc72f21250a225d7f1b5c482415de))

### 🐛 Bug Fixes

- *(llm)* Update default GEMINI model to `gemini-3.5-flash-lite` ([#7](https://github.com/shadowmkj/cogit/pull/7)) - ([646fdf7](https://github.com/shadowmkj/cogit/commit/646fdf79bd211446ad9b1ca85794ced5c6ee1d00))

### 👷 CI

- *(ci)* Add GitHub Actions CI workflow for Rust - ([6c87a09](https://github.com/shadowmkj/cogit/commit/6c87a09820ef7c7a2c8d0f3bdf073346348e1b2d))
- *(ci)* Add code coverage workflow and dependabot configuration - ([6a4ecfa](https://github.com/shadowmkj/cogit/commit/6a4ecfa91216adbbc5223aefa137d71c319f0e68))
- *(actions)* Bump actions/checkout from 4 to 7 ([#3](https://github.com/shadowmkj/cogit/pull/3)) - ([3cb9162](https://github.com/shadowmkj/cogit/commit/3cb91625a3c81279579f76e44bc4c56c045027c2))
- *(ci)* Update codecov configuration settings - ([e693ab7](https://github.com/shadowmkj/cogit/commit/e693ab7c42b84a0ac1b5bf271d9f0f638a829dfe))

### ⚙️ Misc

- Add comprehensive project README - ([b2948da](https://github.com/shadowmkj/cogit/commit/b2948dafafd4f22b2d74824536ce1daa07352213))
- Add GitHub issue and pull request templates - ([14bfbbd](https://github.com/shadowmkj/cogit/commit/14bfbbda3837f787308a245455b6616b99fc9f53))
- Add contributing guidelines and MIT license - ([a29886d](https://github.com/shadowmkj/cogit/commit/a29886dc91e040e04b08b8a6432ef49a18ecb63b))
- Update README with badges, features, and configuration details - ([55f268d](https://github.com/shadowmkj/cogit/commit/55f268d6a738bc5021312cc445bef0ac38afea77))
- Add Contributor Covenant Code of Conduct - ([65dd1a9](https://github.com/shadowmkj/cogit/commit/65dd1a92d9cf1b0b815064020d055df077343e69))

### 👥 New Contributors

- `@shadowmkj` made their first contribution

- `@dependabot[bot]` made their first contribution in [#2](https://github.com/shadowmkj/cogit/pull/2)

