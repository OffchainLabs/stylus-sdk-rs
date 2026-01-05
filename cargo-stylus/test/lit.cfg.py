# -*- Python -*-

import os
import platform
import subprocess
import sys

import lit.formats

# Add venv site-packages to path for colorama
venv_path = os.path.join(os.path.dirname(__file__), 'venv', 'lib')
if os.path.exists(venv_path):
    for item in os.listdir(venv_path):
        if item.startswith('python'):
            site_packages = os.path.join(venv_path, item, 'site-packages')
            if os.path.exists(site_packages):
                sys.path.insert(0, site_packages)
                break

# Configuration file for the 'lit' test runner.

# name: The name of this test suite.
config.name = 'cargo-stylus'

# testFormat: The test format to use to interpret tests.
config.test_format = lit.formats.ShTest(True)

# suffixes: A list of file extensions to treat as test files.
config.suffixes = ['.test']

# test_source_root: The root path where tests are located.
config.test_source_root = os.path.dirname(__file__)

# test_exec_root: The root path where tests should be run.
config.test_exec_root = os.path.join(config.test_source_root, 'Output')

# Substitutions
import shutil

# Find cargo-stylus-beta
if hasattr(config, 'cargo_stylus') and config.cargo_stylus:
    cargo_stylus_path = config.cargo_stylus
else:
    # Try to find cargo-stylus-beta first (the version from stylus-sdk-rs)
    cargo_stylus_path = shutil.which('cargo-stylus-beta')
    if not cargo_stylus_path:
        # Fallback to cargo-stylus if beta not found
        cargo_stylus_path = shutil.which('cargo-stylus')
    if not cargo_stylus_path and hasattr(config, 'cargo_stylus_dir'):
        cargo_stylus_path = os.path.join(config.cargo_stylus_dir, 'target', 'release', 'cargo-stylus-beta')

# Use cargo stylus-beta if available, otherwise cargo stylus
cargo_cmd = 'cargo stylus-beta' if shutil.which('cargo-stylus-beta') else 'cargo stylus'
config.substitutions.append(('%cargo-stylus', cargo_cmd))

# Add wrapper for running with venv (for usertrace tests that need colorama)
venv_wrapper = os.path.join(config.test_source_root, 'run-with-venv.sh')
if os.path.exists(venv_wrapper):
    config.substitutions.append(('cargo-stylus-venv', f'{venv_wrapper} {cargo_cmd}'))
else:
    config.substitutions.append(('cargo-stylus-venv', cargo_cmd))
config.substitutions.append(('%{rpc_url}', getattr(config, 'rpc_url', 'http://localhost:8547')))
config.substitutions.append(('%{chain_id}', getattr(config, 'chain_id', '412346')))
config.substitutions.append(('%{private_key}', getattr(config, 'private_key', '')))

# Contract addresses and transaction hashes
if hasattr(config, 'test_contracts'):
    for key, value in config.test_contracts.items():
        config.substitutions.append(('%{' + key + '}', value))

# Test directories
config.substitutions.append(('%S', config.test_source_root))
config.substitutions.append(('%p', config.test_source_root))
config.substitutions.append(('%{inputs}', os.path.join(config.test_source_root, 'Inputs')))
config.substitutions.append(('%{contracts}', os.path.join(config.test_source_root, 'contracts')))

# Platform-specific features
if platform.system() == 'Darwin':
    config.available_features.add('darwin')
elif platform.system() == 'Linux':
    config.available_features.add('linux')

# Add features for available debuggers
def check_debugger(name):
    try:
        subprocess.run(['which', name], check=True, capture_output=True)
        return True
    except:
        return False

if check_debugger('gdb'):
    config.available_features.add('gdb')
if check_debugger('lldb'):
    config.available_features.add('lldb')
if check_debugger('stylusdb'):
    config.available_features.add('stylusdb')

# Add 'not' command
not_path = shutil.which('not')
if not not_path:
    # Try common locations
    for path in ['/usr/local/opt/llvm/bin', '/opt/homebrew/opt/llvm/bin', '/usr/bin']:
        candidate = os.path.join(path, 'not')
        if os.path.exists(candidate):
            not_path = candidate
            break
if not_path:
    config.substitutions.append(('not', not_path))

# Find and add FileCheck
filecheck_path = None
for path in ['/usr/local/opt/llvm/bin', '/opt/homebrew/opt/llvm/bin', '/usr/bin']:
    candidate = os.path.join(path, 'FileCheck')
    if os.path.exists(candidate):
        filecheck_path = candidate
        break

if filecheck_path:
    config.substitutions.append(('FileCheck', filecheck_path))
else:
    # Try to find FileCheck in PATH
    filecheck_path = shutil.which('FileCheck')
    if filecheck_path:
        config.substitutions.append(('FileCheck', filecheck_path))