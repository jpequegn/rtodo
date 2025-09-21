# RTodo Documentation

This directory contains documentation files for RTodo.

## Man Page

### Installing the Man Page

To install the man page system-wide (requires sudo):

```bash
sudo ./install-manpage.sh
```

To install for current user only:

```bash
./install-manpage.sh
```

### Viewing the Man Page

After installation:

```bash
man rtodo
```

### Manual Installation

If the script doesn't work, you can manually install:

```bash
# For system-wide installation (requires sudo)
sudo cp rtodo.1 /usr/share/man/man1/
sudo mandb

# For user installation
mkdir -p ~/.local/share/man/man1
cp rtodo.1 ~/.local/share/man/man1/
export MANPATH="$HOME/.local/share/man:$MANPATH"
```

### Man Page Sections

The man page includes:

- **Synopsis** - Command syntax
- **Description** - Overview of RTodo
- **Commands** - All available commands with options
- **Examples** - Common usage patterns
- **Files** - Data storage locations
- **Environment** - Environment variables
- **Exit Status** - Return codes
- **See Also** - Related commands

### Updating the Man Page

The man page source is `rtodo.1` in roff format. To modify:

1. Edit `rtodo.1`
2. Test with: `man ./rtodo.1`
3. Reinstall with: `./install-manpage.sh`

### Man Page Format

The man page follows standard Unix manual conventions:
- Section 1: User commands
- Roff markup for formatting
- Standard sections (NAME, SYNOPSIS, DESCRIPTION, etc.)

## Other Documentation

- **README.md** - Main project documentation
- **CONTRIBUTING.md** - Contribution guidelines
- **LICENSE** - MIT license terms