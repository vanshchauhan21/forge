---
layout: default
title: Provider Configuration
parent: Features
nav_order: 12
---

# Provider Configuration

Forge supports multiple AI providers and allows custom configuration to meet your specific needs.

## Supported Providers

Forge automatically detects and uses your API keys from environment variables in the following priority order:

1. `FORGE_KEY` - Antinomy's provider (OpenAI-compatible)
2. `OPENROUTER_API_KEY` - Open Router provider (aggregates multiple models)
3. `OPENAI_API_KEY` - Official OpenAI provider
4. `ANTHROPIC_API_KEY` - Official Anthropic provider

To use a specific provider, set the corresponding environment variable in your `.env` file.

```bash
# Examples of different provider configurations (use only one)

# For Open Router (recommended, provides access to multiple models)
OPENROUTER_API_KEY=your_openrouter_key_here

# For official OpenAI
OPENAI_API_KEY=your_openai_key_here

# For official Anthropic
ANTHROPIC_API_KEY=your_anthropic_key_here

# For Antinomy's provider
FORGE_KEY=your_forge_key_here
```

## Custom Provider URLs

For OpenAI-compatible providers (including Open Router), you can customize the API endpoint URL by setting the `OPENAI_URL` environment variable:

```bash
# Custom OpenAI-compatible provider
OPENAI_API_KEY=your_api_key_here
OPENAI_URL=https://your-custom-provider.com/v1

# Or with Open Router but custom endpoint
OPENROUTER_API_KEY=your_openrouter_key_here
OPENAI_URL=https://alternative-openrouter-endpoint.com/v1
```

For Anthropic, you can customize the API endpoint URL by setting the `ANTHROPIC_URL` environment variable:

```bash
# Custom Anthropic endpoint
ANTHROPIC_API_KEY=your_anthropic_key_here
ANTHROPIC_URL=https://your-custom-anthropic-endpoint.com/v1
```

This is particularly useful when:

- Using self-hosted models with OpenAI-compatible APIs
- Connecting to enterprise OpenAI deployments
- Using proxy services or API gateways
- Working with regional API endpoints