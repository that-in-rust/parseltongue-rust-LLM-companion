#!/bin/bash

# API Switch Script for Parseltongue Project
# This script downloads and runs the Claude Code ZAI environment setup

echo "Switching API environment..."

# Download the environment setup script
curl -O "https://cdn.bigmodel.cn/install/claude_code_zai_env.sh"

# Make the script executable
chmod +x claude_code_zai_env.sh

# Run the environment setup
bash ./claude_code_zai_env.sh

echo "API environment switch completed!"