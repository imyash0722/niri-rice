# Neovim Setup Guide

Welcome to your customized Neovim configuration! This guide will help you navigate the core shortcuts, plugin managers, and powerful tools set up in your editor.

## The Leader Key

Your `<Leader>` key is mapped to the **Spacebar**. 
Whenever a shortcut requires `<Leader>`, simply tap **Space**, then tap the next key in the sequence.

---

## 🚀 The Basics

| Shortcut | Action |
|----------|--------|
| `<Leader>a` | Open the Neovim Home Page / Dashboard |
| `<C-s>` | Save the current file |
| `<Leader>q` | Quit the current window |
| `<Leader>wq` | Save all files and quit Neovim |
| `<C-j>` / `<C-k>` / `<C-l>` | Move cursor between split windows |
| `<A-1>`, `<A-2>` ... | Switch to tab/buffer 1, 2, 3, etc. |
| `<A-,>` / `<A-.>` | Cycle through tabs (Left/Right) |
| `<A-w>` | Close current tab |
| `<A-j>` / `<A-k>` | Move the current line up or down (VS Code style) |
| `<Tab>` / `<S-Tab>` | Indent / Un-indent line |

---

## 🔍 File Exploration & Searching

Your configuration uses **Neo-tree** for the file sidebar and **Telescope** for fuzzy finding.

| Shortcut | Action |
|----------|--------|
| `<Leader>e` | Toggle the file explorer sidebar (Neo-tree) |
| `<C-o>` | Find a file by name (Telescope) |
| `<C-f>` | Search for text inside all files (Live Grep) |
| `<Leader-o>` | Search through your currently open buffers |
| `<Leader>ts` | Switch Git branches |
| `<Leader>ge` | Open Git status explorer |

*Tip: While in Neo-tree, press `H` to toggle hidden files, and `o` to open a file with your system's default app (like an image).*

---

## 💻 Coding & LSP (Language Server Protocol)

Your setup comes fully loaded with intelligent code completion and diagnostics.

| Shortcut | Action |
|----------|--------|
| `gd` | Go to definition |
| `gD` | Go to declaration |
| `gi` | Go to implementation |
| `K` | Show hover documentation for the word under cursor |
| `<Leader>rn` | Rename symbol (variables, functions) across the project |
| `<Leader>k` / `<Leader>j` | Jump to previous / next error or warning |
| `<Leader>e` | Open error message details in a floating window |
| `<Leader>is` | Open Symbols Outline (inspect structure) |

---

## ⚙️ Plugin Management

This configuration is powered by **Lazy.nvim** (for downloading plugins) and **Mason.nvim** (for downloading language servers, linters, and formatters).

| Shortcut | Action |
|----------|--------|
| `<Leader>p` | Open **Lazy** (Update or manage plugins) |
| `<Leader>l` | Open **Mason** (Install formatters or LSPs like Python, JS, etc) |

### Installing Supermaven (AI Assistant)
Your config includes Supermaven for lightning-fast AI completions! 
To start using the free tier, simply type this command and press Enter:
`:SupermavenUseFree`

---

## 📂 Configuration Structure

If you ever want to tweak these settings, here is where everything lives:
- **`~/.config/nvim/init.lua`**: The main entry point
- **`~/.config/nvim/lua/max/core/`**: Core behavior, keymappings, options, and autocommands
- **`~/.config/nvim/lua/max/plugins/`**: Individual plugin configurations (e.g., `dashboard.lua`, `telescope.lua`, `neotree.lua`)
