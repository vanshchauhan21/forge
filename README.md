<!--
Tone: Maintain a professional and informative tone throughout. Ensure that explanations are clear and technical terms are used appropriately to engage a technical audience.
Best Practices:
- Use consistent terminology and formatting for commands and examples.
- Clearly highlight unique aspects of 'forge' to distinguish it from other tools.
-->

[![CI Status](https://img.shields.io/github/actions/workflow/status/antinomyhq/forge/ci.yml?style=for-the-badge)](https://github.com/antinomyhq/forge/actions)
[![GitHub Release](https://img.shields.io/github/v/release/antinomyhq/forge?style=for-the-badge)](https://github.com/antinomyhq/forge/releases)

Code-Forge is an AI-powered interactive shell that stands out through:

- Lightning-fast performance with sub-50ms startup times
- Seamless integration with existing Unix tools and workflows
- Context-aware assistance that understands your development environment and workflows
- Natural language interface to powerful system operations
- Enhanced security features with optional restricted shell mode

**Table of Contents**

- [Installation](#installation)
  - [Mac](#mac)
  - [Linux](#linux)
- [Get Started](#get-started)
- [Features](#features)
  - [1. Interactive Shell](#1-interactive-shell)
  - [2. Enhanced Security](#2-enhanced-security)
  - [3. Model Flexibility](#3-model-flexibility)
  - [4. Autocomplete](#4-autocomplete)
  - [5. Custom Instructions](#5-custom-instructions)
  - [6. WYSIWYG Shell Integration](#6-wysiwyg-shell-integration)
  - [7. Command Interruption](#7-command-interruption)
- [Why Shell?](#why-shell)
- [Support Us](#support-us)

## Installation

### Mac

Using Homebrew (macOS package manager):

```bash
# Add Code-Forge's package repository to Homebrew
brew tap antinomyhq/code-forge
# Install Code-Forge
brew install code-forge
```

### Linux

Choose either method to install:

```bash
# Using curl (common download tool)
curl -L https://raw.githubusercontent.com/antinomyhq/forge/main/install.sh | bash

# Or using wget (alternative download tool)
wget -qO- https://raw.githubusercontent.com/antinomyhq/forge/main/install.sh | bash
```

## Get Started

1. Create a `.env` file in your home directory with your API credentials and model preferences:

```bash
# Your OpenRouter API key for accessing AI models
OPEN_ROUTER_KEY=<Enter your Open Router Key>

# Preferred model for complex tasks requiring deeper analysis
FORGE_LARGE_MODEL=anthropic/claude-3.5-sonnet

# Efficient model for quick, routine tasks
FORGE_SMALL_MODEL=anthropic/claude-3.5-haiku
```

2. Start an interactive shell by typing `forge`:

```bash
forge
⚡ # Write your task here and press enter or type
```

For additional configuration options and features, use `forge --help`.

## Features

### 1. Interactive Shell

Transform your command-line experience with natural language interaction while maintaining the power and flexibility of traditional shell commands.

**Example**:
Start the interactive shell with:

```bash
forge
```

### 2. Enhanced Security

Code-Forge prioritizes security by providing a restricted shell mode (rbash) that limits potentially dangerous operations:

- **Flexible Security Options**: Choose between standard and restricted modes based on your needs
- **Restricted Mode**: Enable with `-r` flag to prevent potentially harmful operations
- **Standard Mode**: Uses regular shell by default (bash on Unix/Mac, cmd on Windows)
- **Security Controls**: Restricted mode prevents:
  - Changing directories
  - Setting/modifying environment variables
  - Executing commands with absolute paths
  - Modifying shell options

**Example**:

```bash
# Standard mode (default)
forge

# Restricted secure mode
forge -r
```

### 3. Model Flexibility

Optimize your workflow by choosing the right AI model for each task:

- **Quick Tasks**: Use lightweight models for rapid responses
- **Complex Operations**: Leverage more powerful models for in-depth analysis

**Configuration**:

```bash
export FORGE_LARGE_MODEL=anthropic/claude-3.5-sonnet
export FORGE_SMALL_MODEL=anthropic/claude-3.5-haiku
```

### 4. Autocomplete

Boost your productivity with intelligent command completion:

- Type `@` and press Tab for contextual file/path completion
- Use Right Arrow to complete previously executed commands
- Access command history with Up Arrow
- Quick history search with Ctrl+R

### 5. Custom Instructions

Tailor Code-Forge to your specific needs with custom instruction sets:

```bash
forge --custom-instructions path/to/instructions.yml
```

### 6. WYSIWYG Shell Integration

Enhance your interactive shell experience with WYSIWYG (What You See Is What You Get) integration. 'forge' now visualizes each command executed, complete with colorful formatting, allowing you to see command outputs just as if you were typing them directly into your terminal. This feature ensures clarity and enhances interaction, making every command visible in rich detail.

### 7. Command Interruption

Stay in control of your shell environment with intuitive command handling:

- **Cancel with `CTRL+C`:** Gracefully interrupt ongoing operations, providing the flexibility to halt processes that no longer need execution.
- **Exit with `CTRL+D`:** Easily exit the shell session without hassle, ensuring you can quickly terminate your operations when needed.

## Why Shell?

There's a reason why the shell stood the test of time for all dev tools and still remains a cornerstone of development environments across the globe: it's fast, versatile, and seamlessly integrated with the system. The shell is where we navigate code, run tests, manage processes, and orchestrate our development environments, providing an unmatched level of control and productivity.

**Why a shell-based AI assistant like Code-Forge makes sense:**

- **Rich Tool Ecosystem**: The shell gives you immediate access to powerful Unix tools (grep, awk, sed, find) that LLMs already understand deeply. This means the AI can leverage `ripgrep` for code search, `jq` for JSON processing, `git` for version control, and hundreds of other battle-tested tools without reinventing them.

- **Context is Everything**: Your shell session already has your complete development context - current directory, project structure, environment variables, installed tools, and system state. This rich context makes the AI interactions more accurate and relevant.

- **Speed Matters**: Unlike IDEs and Web UI, Code Forge's shell is extremely light weight. This exceptional speed unlocks powerful capabilities that directly enhance your productivity: seamlessly get in and out of workflows, managing multiple feature developments in parallel, effortlessly coordinate across git worktrees, and instantly access AI assistance in any directory.

- **Tool Composition**: Unix philosophy teaches us to make tools that compose well. The AI can pipe commands together, combining tools like `find | xargs forge -p | grep "foo"` in ways that solve complex problems elegantly.

## Support Us

Your support drives Code-Forge's continued evolution! By starring our GitHub repository, you:

- Help others discover this powerful tool
- Motivate our development team
- Enable us to prioritize new features
- Strengthen our open-source community

Recent community feedback has helped us implement features like improved autocomplete, cross-platform optimization, and enhanced security features. Join our growing community of developers who are reshaping the future of AI-powered development!
