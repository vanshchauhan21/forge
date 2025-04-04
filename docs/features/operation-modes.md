---
layout: default
title: Operation Modes
parent: Features
nav_order: 6
---

# Operation Modes

Forge operates in two distinct modes to provide flexible assistance based on your needs:

## ACT Mode (Default)

In ACT mode, which is the default when you start Forge, the assistant is empowered to directly implement changes to your codebase and execute commands:

- **Full Execution**: Forge can modify files, create new ones, and execute shell commands
- **Implementation**: Directly implements the solutions it proposes
- **Verification**: Performs verification steps to ensure changes work as intended
- **Best For**: When you want Forge to handle implementation details and fix issues directly

**Example**:

```bash
# Switch to ACT mode within a Forge session
/act
```

## PLAN Mode

In PLAN mode, Forge analyzes and plans but doesn't modify your codebase:

- **Read-Only Operations**: Can only read files and run non-destructive commands
- **Detailed Analysis**: Thoroughly examines code, identifies issues, and proposes solutions
- **Structured Planning**: Provides step-by-step action plans for implementing changes
- **Best For**: When you want to understand what changes are needed before implementing them yourself

**Example**:

```bash
# Switch to PLAN mode within a Forge session
/plan
```

You can easily switch between modes during a session using the `/act` and `/plan` commands. PLAN mode is especially useful for reviewing potential changes before they're implemented, while ACT mode streamlines the development process by handling implementation details for you.