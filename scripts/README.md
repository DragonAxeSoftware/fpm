# Scripts Directory

This directory contains various scripts for development, testing, and utilities.

## Directory Structure

### 🌐 `mcp/`
Model Context Protocol (MCP) server implementations organized by functionality.

#### `mcp/arch_viz/`
Architecture visualization MCP server for generating dependency diagrams.

- **`server.py`** - MCP server that accepts JSON-RPC requests
- **`visualizer.py`** - Core visualization using networkx and matplotlib
- **`visualizer_mock.py`** - Mock visualizer for testing without dependencies
- **`README.md`** - Module documentation and testing guide
- **`__init__.py`** - Python package initialization

##### `mcp/arch_viz/tests/`
Test scripts and output files for the architecture visualizer.

- `test_mcp_direct.py` - Direct subprocess test for MCP server
- `test_arch_viz_mcp.py` - Python test script
- `test_arch_viz_mcp.ps1` - PowerShell test script
- `simple_test_input.py` - Generates sample JSON-RPC requests
- `test_input.json` - Sample test input
- `*.png` - Generated test diagrams

### 🔧 `setup/`
Setup and installation scripts.

- **`install-pyenv-win.ps1`** - Installs pyenv-win for Python version management

### 🛠️ `utils/`
General utility scripts.

- **`_show_file_tree.ps1`** - Displays the project file structure

## Usage

### Running the Architecture Visualizer MCP Server

```bash
# As a Python module
pyenv exec python -m mcp.arch_viz.server

# Or directly
pyenv exec python mcp/arch_viz/server.py

# Test the MCP server
cd mcp/arch_viz/tests
pyenv exec python test_mcp_direct.py
```

### Setting up Development Environment

```powershell
# Install pyenv-win
.\setup\install-pyenv-win.ps1

# Install Python 3.13
pyenv install 3.13.0
pyenv local 3.13.0

# Install dependencies
pyenv exec python -m pip install matplotlib networkx
```

## Adding New MCP Servers

To add a new MCP server:

1. Create a new subdirectory under `mcp/` (e.g., `mcp/my_tool/`)
2. Add server implementation, core logic, and `__init__.py`
3. Create a `tests/` subdirectory for test files
4. Update this README with usage instructions
