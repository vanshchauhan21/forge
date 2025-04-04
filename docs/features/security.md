
---

## layout: default
title: Enhanced Security
parent: Features
nav_order: 3

# Enhanced Security

Code-Forge prioritizes security by providing a restricted shell mode (rbash) that limits potentially dangerous operations:

## Security Features

* **Flexible Security Options**: Choose between standard and restricted modes based on your needs
* **Restricted Mode**: Enable with `-r` flag to prevent potentially harmful operations
* **Standard Mode**: Uses regular shell by default (bash on Unix/Mac, cmd on Windows)
* **Security Controls**: Restricted mode prevents:
  * Changing directories
  * Setting/modifying environment variables
  * Executing commands with absolute paths
  * Modifying shell options

**Example**:

```bash
# Standard mode (default)
forge

# Restricted secure mode
forge -r
```

## Additional Security Features

* Direct API connection to Open Router without intermediate servers
* Local terminal operation for maximum control and data privacy
* Configuration validation to prevent security vulnerabilities
* Secure handling of API keys and sensitive information


