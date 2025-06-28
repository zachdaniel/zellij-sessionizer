# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

zellij-sessionizer is a Rust-based Zellij plugin that provides a session management interface inspired by ThePrimeagen's tmux sessionizer. It allows users to quickly navigate and create Zellij sessions based on project directories.

## Key Commands

### Build
```bash
cargo build --target wasm32-wasi --release
```

### Development
```bash
make dev  # Opens development workspace with hot reload
```

### Deploy
```bash
make deploy  # Builds and copies to ~/.config/zellij/plugins/
```

### Build/Install for User
When asked to build or install this plugin, the wasm file should be copied to:
```bash
~/.dotfiles/zellij/plugins/zellij-sessionizer.wasm
```

### Testing
```bash
cargo test
```

## Architecture

The plugin follows a component-based architecture with clear separation of concerns:

1. **State Management** (`src/main.rs`): The main plugin entry point manages two screens:
   - `SearchDirs`: Directory browser with fuzzy search
   - `SearchSessions`: Active/resurrectable session manager
   
2. **Components**:
   - `DirList` (`src/dirlist.rs`): Manages project directory listing with scrollable navigation
   - `SessList` (`src/sesslist.rs`): Manages Zellij sessions with visual state indicators
   - `TextInput` (`src/textinput.rs`): Search bar component with cursor handling
   - `Filter` (`src/filter.rs`): Fuzzy search implementation using nucleo-matcher

3. **Configuration** (`src/config.rs`): Handles KDL-based plugin configuration including:
   - Root directories for project search
   - Custom session layouts
   - Built-in vs custom layout parsing

## Important Implementation Details

### Session Icons
Sessions use specific Nerd Font icons to indicate state:
- ` ` (f2d0) - Active session
- ` ` (f2d2) - Current session  
- `󰤄` (f0904) - Resurrectable (dead) session

### Target Platform
The plugin must be compiled for `wasm32-wasi` (or `wasm32-wasip1`) target. The project uses Rust 2021 edition.

### Key Bindings in Plugin
- `Tab`: Switch between directory and session search
- `Enter`: Create/attach to session
- `Ctrl+N/P` or `Down/Up`: Navigate list
- `Ctrl+X`: Kill selected session (in session mode)
- `Esc`: Close plugin

### File System Interaction
- The plugin's `cwd` should be set to `/` for proper absolute path handling
- Hidden directories are filtered except for `.config`
- Directory search is one level deep from configured root directories

## Development Workflow

1. Run `make dev` to open the development workspace
2. The plugin will auto-reload on changes (via watchexec in the dev workspace)
3. Test changes in the floating pane that opens
4. Use `make deploy` to install the plugin locally

## Recent Features

- Resurrectable session support with visual indicators
- Session deletion via Ctrl+X
- Improved session name/icon separation to fix duplication bugs
- Proper session killing that matches Zellij's built-in behavior