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

**Table of Contents**

- [Installation](#installation)
  - [Mac](#mac)
  - [Linux](#linux)
- [Get Started](#get-started)
- [Features](#features)
  - [Interactive Shell](#interactive-shell)
  - [Model Flexibility](#model-flexibility)
  - [Autocomplete](#autocomplete)
  - [Custom Instructions](#custom-instructions)
  - [System Prompts](#system-prompts)
- [Capabilities](#capabilities)
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
OPEN_ROUTER_KEY=[Enter your Open Router Key]

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

### Interactive Shell

Transform your command-line experience with natural language interaction while maintaining the power and flexibility of traditional shell commands.

**Example**:
Start the interactive shell with:

```bash
forge
```

### Model Flexibility

Optimize your workflow by choosing the right AI model for each task:

- **Quick Tasks**: Use lightweight models for rapid responses
- **Complex Operations**: Leverage more powerful models for in-depth analysis

**Configuration**:

```bash
export FORGE_LARGE_MODEL=anthropic/claude-3.5-sonnet
export FORGE_SMALL_MODEL=anthropic/claude-3.5-haiku
```

### Autocomplete

Boost your productivity with intelligent command completion:

- Type `@` and press Tab for contextual file/path completion
- Use Right Arrow to complete previously executed commands
- Access command history with Up Arrow
- Quick history search with Ctrl+R

### Custom Instructions

Tailor Code-Forge to your specific needs with custom instruction sets:

```bash
forge --custom-instructions path/to/instructions.yml
```

### System Prompts

Leverage pre-configured expert modes for specialized tasks:

```bash
forge --system-prompt prompts/technical_writer_prompt.txt
```

## Capabilities

## Why Shell?

We have been using shells (bash, zsh, fish) as their primary interface for decades because they're fast, powerful, and close to the system. The shell is our natural habitat - it's where we navigate code, run tests, manage processes, and orchestrate our development environment. That's why Code-Forge reimagines the shell as an AI-powered environment where you can interact with an intelligent agent using natural language.

Why a shell-based AI assistant makes sense:

- **Rich Tool Ecosystem**: The shell gives you immediate access to powerful Unix tools (grep, awk, sed, find) that LLMs already understand deeply. This means the AI can leverage `ripgrep` for code search, `jq` for JSON processing, `git` for version control, and hundreds of other battle-tested tools without reinventing them.

- **Context is Everything**: Your shell session already has your complete development context - current directory, project structure, environment variables, and system state. This rich context makes the AI interactions more accurate and relevant.

- **Speed Matters**: Code-Forge revolutionizes development workflows through its Rust-powered performance, delivering immediate value with sub-50ms startup times. This exceptional speed unlocks powerful capabilities that directly enhance your productivity: seamlessly manage multiple feature developments in parallel, effortlessly coordinate across git worktrees, and instantly access AI assistance in any directory. By eliminating the frustrating delays and context switches common to web-based AI assistants, Code-Forge keeps you in your flow state. The result? A development experience where your tools match your thinking speed, enabling faster iterations, smoother task transitions, and more efficient project management across all your development contexts.

- **Workflow Integration**: Engineers context-switch about 13.3 times per hour between tools (according to Microsoft research). A shell-based AI assistant stays in your existing workflow - no need to switch windows, lose context, or break your flow.

- **Tool Composition**: Unix philosophy teaches us to make tools that compose well. The AI can pipe commands together, combining tools like `find | xargs | sort | uniq` in ways that solve complex problems elegantly.

- **Ephemeral by Default**: Unlike chat interfaces that accumulate state, shell sessions are naturally ephemeral and reproducible. This matches how developers work - focused sessions for specific tasks, with reliable, reproducible environments.

Think of Code-Forge as your productivity amplifier, where natural communication meets system-level power. By eliminating the cognitive overhead of command memorization, it lets you focus on what matters - solving problems and building features. You gain immediate access to the entire Unix toolkit through simple, conversational requests, while sophisticated code analysis and generation capabilities accelerate your development workflow. This natural interaction layer transforms complicated system operations into intuitive conversations, making development more accessible and efficient without sacrificing any of the power that experienced developers expect.

## Support Us

Your support drives Code-Forge's continued evolution! By starring our GitHub repository, you:

- Help others discover this powerful tool
- Motivate our development team
- Enable us to prioritize new features
- Strengthen our open-source community

Recent community feedback has helped us implement features like improved autocomplete and cross-platform optimization. Join our growing community of developers who are reshaping the future of AI-powered development!
