#!/bin/bash
# SETUP.sh
#   by Lut99
# 
# First-run setup script to prepare your system for running the server.
# 
# This will:
# - Create directories with suitable permissions so the non-root use in the container can write to
#   them.
# 
# Needless to say, you only need to run this once.
# 


# Always work from the script's source directory
cd "$(dirname "$0")" || exit "$?"



##### CREATING DIRECTORIES #####
echo " > Creating directory 'config/'..."
mkdir -p config || exit "$?"
echo " > Creating directory 'data/'..."
mkdir -p data || exit "$?"



##### GENERATE ROOT PASSWORD #####
echo " > Generating root credentials to 'config/root.toml'"
pass=$(openssl rand -hex 32) || exit "$?"
bash -c "printf \"[credentials]\\nname=\\\"root\\\"\\npass=\\\"$pass\\\"\\n\" > config/root.toml" || exit "$?"



##### DONE #####
echo ""
echo "You're now ready to run:"
echo "   docker compose up -d --build"
echo ""
exit 0
