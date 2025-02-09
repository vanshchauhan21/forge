[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![CI Status](https://img.shields.io/github/actions/workflow/status/antinomyhq/forge/ci.yml?style=for-the-badge)](https://github.com/antinomyhq/forge/actions)
[![GitHub Release](https://img.shields.io/github/v/release/antinomyhq/forge?style=for-the-badge)](https://github.com/antinomyhq/forge/releases)
[![Discord](https://img.shields.io/badge/Discord-Join%20Us-blue?style=for-the-badge)](https://discord.gg/Rdyu7hgSWm)

An open-source AI powered interactive shell

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

Accelerate your commands with intelligent autocompletion using the `@` symbol, reducing keystrokes and minimizing errors.

**Example**:
While typing a command, input `forge @` to get suggestions for completing the command or path.

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

If you find Forge useful, please consider giving us a star ⭐ on GitHub. It helps make the project more visible and encourages further development.

<p align="center">
  <a href="https://github.com/antinomyhq/forge">
    <img src="https://img.shields.io/github/stars/antinomyhq/forge?style=social" alt="Give us a Star">
  </a>
</p>

Your support means a lot to us! Here's what starring does:

- Shows appreciation to the developers
- Helps others discover the project
- Builds credibility in the open-source community
- Motivates us to keep improving Forge
