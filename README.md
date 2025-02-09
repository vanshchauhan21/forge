[![CI Status](https://img.shields.io/github/actions/workflow/status/antinomyhq/forge/ci.yml?style=for-the-badge)](https://github.com/antinomyhq/forge/actions)
[![GitHub Release](https://img.shields.io/github/v/release/antinomyhq/forge?style=for-the-badge)](https://github.com/antinomyhq/forge/releases)

<!-- Keep: The explanation of why a shell-based AI assistant makes sense, particularly the points on rich tool ecosystem and workflow integration, as they aptly communicate the advantages to knowledgeable users. -->
<!-- Improve: Clarify the unique selling points at the very beginning, highlighting 'forge's distinct differences from other tools to immediately capture interest. -->
An open-source AI powered interactive shell

We have been using shells (bash, zsh, fish) as their primary interface for decades because they're fast, powerful, and close to the system. The shell is our natural habitat - it's where we navigate code, run tests, manage processes, and orchestrate our development environment. That's why Code-Forge reimagines the shell as an AI-powered environment where you can interact with an intelligent agent using natural language.

Why a shell-based AI assistant makes sense:

- **Rich Tool Ecosystem**: The shell gives you immediate access to powerful Unix tools (grep, awk, sed, find) that LLMs already understand deeply. This means the AI can leverage `ripgrep` for code search, `jq` for JSON processing, `git` for version control, and hundreds of other battle-tested tools without reinventing them.

- **Context is Everything**: Your shell session already has your complete development context - current directory, project structure, environment variables, and system state. This rich context makes the AI interactions more accurate and relevant.

- **Speed Matters**: Code-Forge revolutionizes development workflows through its Rust-powered performance, delivering immediate value with sub-50ms startup times. This exceptional speed unlocks powerful capabilities that directly enhance your productivity: seamlessly manage multiple feature developments in parallel, effortlessly coordinate across git worktrees, and instantly access AI assistance in any directory. By eliminating the frustrating delays and context switches common to web-based AI assistants, Code-Forge keeps you in your flow state. The result? A development experience where your tools match your thinking speed, enabling faster iterations, smoother task transitions, and more efficient project management across all your development contexts.

- **Workflow Integration**: Engineers context-switch about 13.3 times per hour between tools (according to Microsoft research). A shell-based AI assistant stays in your existing workflow - no need to switch windows, lose context, or break your flow.

- **Tool Composition**: Unix philosophy teaches us to make tools that compose well. The AI can pipe commands together, combining tools like `find | xargs | sort | uniq` in ways that solve complex problems elegantly.

- **Ephemeral by Default**: Unlike chat interfaces that accumulate state, shell sessions are naturally ephemeral and reproducible. This matches how developers work - focused sessions for specific tasks, with reliable, reproducible environments.

Think of Code-Forge as your productivity amplifier, where natural communication meets system-level power. By eliminating the cognitive overhead of command memorization, it lets you focus on what matters - solving problems and building features. You gain immediate access to the entire Unix toolkit through simple, conversational requests, while sophisticated code analysis and generation capabilities accelerate your development workflow. This natural interaction layer transforms complicated system operations into intuitive conversations, making development more accessible and efficient without sacrificing any of the power that experienced developers expect.

**Table of Contents**

- [Installation](#installation)
  - [Mac](#mac)
  - [Linux](#linux)
- [Get Started](#get-started)
- [Features](#features)
  - [Interactive Shell](#interactive-shell)
  - [Model Flexibility](#model-flexibility)
  - [Cross-Platform](#cross-platform)
  - [Autocomplete](#autocomplete)
  - [Custom Instructions](#custom-instructions)
  - [System Prompts](#system-prompts)
- [Support Us](#support-us)

## Installation
<!-- Keep: Clear and concise instructions for installation on both macOS and Linux, ensuring users have straightforward guidance to get started with 'forge'. -->
<!-- Improve: Add brief explanations on what 'brew tap' and 'curl' commands do for users unfamiliar with these terms, providing a slightly more detailed context. -->

### Mac

```
brew tap antinomyhq/code-forge
brew install code-forge
```

### Linux

```bash
# Download and install in one command
curl -L https://raw.githubusercontent.com/antinomyhq/forge/main/install.sh | bash

# Or with wget
wget -qO- https://raw.githubusercontent.com/antinomyhq/forge/main/install.sh | bash
```

## Get Started
<!-- Keep: Simple step-by-step instructions for setting up the initial environment and starting the interactive shell, which helps users to easily follow along and begin using the tool. -->
<!-- Improve: Provide a brief explanation of the purpose of each `.env` variable and how they impact 'forge's operation to avoid any confusion for less experienced users. -->

1. Create a `.env` file in your home directory and set the following variables:

```bash
OPEN_ROUTER_KEY=[Enter your Open Router Key]
FORGE_LARGE_MODEL=anthropic/claude-3.5-sonnet
FORGE_SMALL_MODEL=anthropic/claude-3.5-haiku
```

2. Start an interactive shell by typing `forge`:

```bash
forge
⚡ # Write your task here and press enter or type
```

Use `forge --help` to configure additional parameters.

## Features
<!-- Keep: Comprehensive coverage of different features like Model Flexibility and Interactive Shell, which highlights the tool's capabilities and gives users insight into its versatility. -->
<!-- Improve: Introduce consistent subsections for each feature starting with a value proposition followed by detailed technical specifics, to enhance understanding and engagement. -->

Leveraging Navigational Shortcuts

Enhance your productivity by utilizing convenient keyboard shortcuts like `Ctrl+R` for reverse search and the `Up Arrow` key to cycle through your command history effortlessly.

### Interactive Shell

Seamlessly integrate with your existing command-line workflow while leveraging advanced AI capabilities.

**Example**:
Start the interactive shell with the basic command:

```bash
forge
```

### Model Flexibility

Choose between different AI models to optimize for your specific needs:

- Use lightweight models for quick tasks
- Leverage more powerful models for complex operations

**Example**:
Set environment variables to choose models:

```bash
export FORGE_LARGE_MODEL=anthropic/claude-3.5-sonnet
export FORGE_SMALL_MODEL=anthropic/claude-3.5-haiku
```

### Cross-Platform

Works seamlessly on both macOS and Linux systems, adapting its behavior for optimal performance.

**Example**:
Forge recognizes the operating system and optimizes its behavior accordingly:

- On **macOS**, `forge launch` might open a file using Finder.
- On **Linux**, the same command would use the default file manager like Nautilus.

### Autocomplete

Accelerate your command entry with intelligent autocompletion in the interactive shell by typing `@` and pressing Tab to autocomplete file paths or commands contextually. If the command has been executed before, use the `Right Arrow` key to complete it quickly.

**Example**:
While in the interactive shell, if you type `forge open @` followed by the Tab key, it will suggest files in the current directory to complete your command.

### Custom Instructions

Create and execute project-specific commands to meet your unique requirements.

**Example**:
Apply a custom instruction set:

```bash
forge --custom-instructions path/to/instructions.yml
```

### System Prompts

Use predefined system prompts like "technical writer" to generate comprehensive documentation or execute routine tasks.

**Example**:
Execute a system prompt for documentation:

```bash
forge --system-prompt prompts/technical_writer_prompt.txt
```

## Support Us
<!-- Keep: The call to action to star the project on GitHub, effectively encouraging community support and contributions, which is crucial for open-source success. -->
<!-- Improve: Add specific examples of how past community feedback or support has helped 'forge' grow, making the benefit of community engagement more tangible and appealing to potential contributors. -->

If you find Forge useful, please consider giving us a star on GitHub. It helps make the project more visible and encourages further development.

Your support means a lot to us! Here's what starring does:

- Shows appreciation to the developers
- Helps others discover the project
- Builds credibility in the open-source community
- Motivates us to keep improving Forge
