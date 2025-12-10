#!/bin/bash
# Wrapper to run commands with the virtual environment activated

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Activate virtual environment
source "$SCRIPT_DIR/venv/bin/activate"

# Clean up any old trace files
rm -f /tmp/lldb_function_trace.json

# Run the command
exec "$@"