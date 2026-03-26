# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri v2 desktop application with a Vue 3 frontend. The project uses:
- **Frontend**: Vue 3 with Composition API (`<script setup>` syntax), Vite for bundling
- **Backend**: Rust with Tauri v2 framework
- **Package Manager**: pnpm

## Architecture

### Frontend (src/)
- `src/main.js` - Vue app entry point
- `src/App.vue` - Root Vue component
- Vue components communicate with Rust backend via `@tauri-apps/api/core` invoke() function

### Backend (src-tauri/)
- `src-tauri/src/main.rs` - Application entry point, calls the library
- `src-tauri/src/lib.rs` - Core Tauri application logic, command handlers, and plugins
- `src-tauri/tauri.conf.json` - Tauri configuration (window settings, build commands, bundle config)
- `src-tauri/Cargo.toml` - Rust dependencies and package configuration
- Library name is `hooks_lib` (with `_lib` suffix to avoid Windows naming conflicts)

### Tauri Commands
Rust functions exposed to frontend are marked with `#[tauri::command]` and registered in `invoke_handler`. Frontend calls them using `invoke("command_name", { args })`.

## Development Commands

```bash
# Install dependencies
pnpm install

# Run development server (starts both Vite dev server and Tauri)
pnpm tauri dev

# Build frontend only
pnpm build

# Build production application
pnpm tauri build

# Preview production build
pnpm preview
```

## Configuration

- **Dev server**: Runs on port 1420 (configured in vite.config.js and tauri.conf.json)
- **HMR port**: 1421
- **Build output**: Frontend builds to `dist/`, final app bundles to `src-tauri/target/release/`

## Adding Tauri Commands

1. Add command function in `src-tauri/src/lib.rs` with `#[tauri::command]` attribute
2. Register in `.invoke_handler(tauri::generate_handler![command1, command2, ...])`
3. Call from Vue using `invoke("command_name", { param: value })`

## Adding Tauri Plugins

1. Add dependency to `src-tauri/Cargo.toml`
2. Register plugin in `lib.rs` using `.plugin(plugin_name::init())`
