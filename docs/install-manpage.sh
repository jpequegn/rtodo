#!/bin/bash
# Install rtodo man page

set -e

# Check if running as root or with sudo
if [[ $EUID -eq 0 ]]; then
    MAN_DIR="/usr/share/man/man1"
else
    # Install to user's local man directory
    MAN_DIR="$HOME/.local/share/man/man1"
    mkdir -p "$MAN_DIR"
fi

echo "Installing rtodo man page to $MAN_DIR..."

# Copy the man page
cp "$(dirname "$0")/rtodo.1" "$MAN_DIR/"

# Update man database if available
if command -v mandb >/dev/null 2>&1; then
    if [[ $EUID -eq 0 ]]; then
        mandb
    else
        mandb -u "$HOME/.local/share/man"
    fi
    echo "Updated man database"
elif command -v makewhatis >/dev/null 2>&1; then
    makewhatis "$MAN_DIR"
    echo "Updated whatis database"
fi

echo "Man page installed successfully!"
echo "You can now run: man rtodo"

# Add instructions for PATH if needed
if [[ $EUID -ne 0 ]]; then
    echo ""
    echo "Note: If 'man rtodo' doesn't work, you may need to add to your shell profile:"
    echo "export MANPATH=\"\$HOME/.local/share/man:\$MANPATH\""
fi