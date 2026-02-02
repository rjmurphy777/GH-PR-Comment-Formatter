# 🚀 PR Comments CLI

> **Transform your GitHub PR review comments into LLM-ready context in seconds!**

[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests: 99](https://img.shields.io/badge/tests-99%20passing-brightgreen.svg)](#testing)
[![Coverage: 98%](https://img.shields.io/badge/coverage-98%25-brightgreen.svg)](#testing)

---

## ✨ What is this?

**PR Comments CLI** is a powerful command-line tool that fetches GitHub Pull Request review comments and formats them perfectly for consumption by Large Language Models (LLMs) like Claude, GPT-4, and others.

Ever wanted to ask an AI to help you address PR feedback? Now you can pipe those comments directly into your favorite LLM with all the context it needs! 🎯

---

## 🎬 Quick Start

```bash
# Install the package
pip install -e .

# Fetch PR comments with a URL
pr-comments https://github.com/owner/repo/pull/123

# Or use the shorthand format
pr-comments owner/repo#123
```

**That's it!** You'll get beautifully formatted, context-rich output ready for any LLM. 🎉

---

## 🌟 Features

### 📥 **Flexible Input**
- Full GitHub PR URLs: `https://github.com/owner/repo/pull/123`
- Shorthand notation: `owner/repo#123`
- Explicit flags: `--owner`, `--repo`, `--pr-number`

### 📤 **Multiple Output Formats**

| Format | Description | Best For |
|--------|-------------|----------|
| `claude` | 🤖 LLM-optimized with full context | AI assistants |
| `grouped` | 📁 Organized by file | Code review |
| `flat` | 📋 Chronological list | Timeline view |
| `minimal` | ⚡ Quick overview | Fast scanning |
| `json` | 🔧 Raw data | Automation |

### 🔍 **Powerful Filtering**
- **`--author`** - Filter by reviewer username
- **`--most-recent`** - Only the latest comment per file

### 🎨 **Rich Code Context**
- Automatic code snippet extraction from diffs
- Configurable snippet length (`--snippet-lines`)
- Full file path and line number tracking

---

## 📖 Usage Examples

### Basic Usage

```bash
# Get all comments formatted for Claude
pr-comments https://github.com/facebook/react/pull/27632

# Save output to a file
pr-comments owner/repo#123 --output comments.md
```

### Filtering Comments

```bash
# Only show comments from a specific reviewer
pr-comments owner/repo#123 --author "senior-dev"

# Get the most recent comment per file
pr-comments owner/repo#123 --most-recent
```

### Different Output Formats

```bash
# Grouped by file (great for systematic review)
pr-comments owner/repo#123 --format grouped

# Minimal overview (quick scan)
pr-comments owner/repo#123 --format minimal

# JSON for scripting
pr-comments owner/repo#123 --format json | jq '.[] | .file'
```

### Customizing Code Snippets

```bash
# Show more context (20 lines per snippet)
pr-comments owner/repo#123 --snippet-lines 20

# Hide code snippets entirely
pr-comments owner/repo#123 --no-snippet
```

---

## 🛠️ Installation

### Prerequisites

- **Python 3.10+**
- **GitHub CLI (`gh`)** - [Install here](https://cli.github.com/)
- Authenticated with `gh auth login`

### Install from Source

```bash
git clone https://github.com/your-org/pr-comments.git
cd pr-comments
pip install -e .
```

### For Development

```bash
pip install -e ".[dev]"
```

---

## 🎯 Output Formats Deep Dive

### 🤖 Claude Format (Default)

The `claude` format is specifically designed for AI consumption:

```markdown
# Pull Request Review Comments
**PR Title:** Add authentication middleware
**PR URL:** https://github.com/owner/repo/pull/123
**Total Comments:** 5
**Files Affected:** 3

Below are the review comments that need to be addressed...

---

## File: `src/auth/middleware.py`

### line 42
**Reviewer:** alice

**Code being reviewed:**
```python
def authenticate(request):
    token = request.headers.get('Authorization')
    # ... more context
```

**Review comment:**
Consider adding rate limiting here to prevent brute force attacks.

---
```

### 📁 Grouped Format

Comments organized by file, perfect for systematic code review:

```markdown
# PR Comments Summary
Total comments: 12
Files with comments: 4

## src/api/routes.py
(3 comment(s))

### src/api/routes.py (line 15)
**Author:** bob
...
```

### ⚡ Minimal Format

Quick overview when you just need the gist:

```
PR Comments: 12 total across 4 files

📄 src/api/routes.py
  └─ line 15 (bob): Consider using async here...
  └─ line 42 (alice): This could be simplified...

📄 src/models/user.py
  └─ line 8 (charlie): Add validation for email...
```

---

## 🧪 Testing

This project is **extensively tested** with real-world scenarios:

```bash
# Run all tests
pytest

# Run with coverage report
pytest --cov=pr_comments --cov-report=term-missing

# Skip integration tests (no GitHub API)
pytest -m "not integration"
```

| Metric | Value |
|--------|-------|
| Total Tests | 99 |
| Coverage | 98% |
| Integration Tests | ✅ Real GitHub PRs |

---

## 🏗️ Architecture

```
pr_comments/
├── cli.py          # 🎮 Command-line interface
├── fetcher.py      # 🌐 GitHub API integration via `gh`
├── parser.py       # 🔄 JSON parsing & filtering
├── formatter.py    # ✨ Output formatting magic
└── models.py       # 📦 Data structures
```

The tool leverages the **GitHub CLI (`gh`)** for authentication and API access, meaning:
- No API tokens to manage
- Automatic rate limit handling
- Works with your existing GitHub auth

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 💡 Pro Tips

1. **Pipe to `pbcopy`** (macOS) for instant clipboard access:
   ```bash
   pr-comments owner/repo#123 | pbcopy
   ```

2. **Combine with AI CLI tools**:
   ```bash
   pr-comments owner/repo#123 | claude-cli
   ```

3. **Use JSON format for scripts**:
   ```bash
   pr-comments owner/repo#123 -f json | jq '.[] | select(.author == "bot")'
   ```

---

<div align="center">

**Made with ❤️ for developers who love automation**

🌟 **Star this repo** if you find it useful! 🌟

</div>
