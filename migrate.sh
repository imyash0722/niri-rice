#!/usr/bin/env bash
# migrate.sh — backs up a user's existing dotfiles and replaces them with this rice
# Usage: bash migrate.sh
# Safe to re-run — won't overwrite existing backups.

set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_DIR="$HOME/.dotfiles-backup-$(date +%Y%m%d-%H%M%S)"

echo "=========================================="
echo "  Niri Rice — Dotfile Migration"
echo "=========================================="
echo "  Repo   : $REPO_DIR"
echo "  Backup : $BACKUP_DIR"
echo ""
echo "This will:"
echo "  1. Move your current dotfiles to $BACKUP_DIR"
echo "  2. Symlink everything from this repo in their place"
echo ""
read -rp "Continue? [y/N]: " confirm
[[ "$confirm" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 0; }

mkdir -p "$BACKUP_DIR"

# ---------------------------------------------------------------------------
# Helper: move existing file/dir to backup, then symlink repo version in
# ---------------------------------------------------------------------------
migrate_link() {
    local src="$1"   # file/dir inside repo
    local dst="$2"   # target path in home

    # Already correctly symlinked — nothing to do
    if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
        echo "  Already linked: $dst"
        return 0
    fi

    # Back up whatever is there
    if [ -e "$dst" ] || [ -L "$dst" ]; then
        echo "  Backing up : $dst"
        mv "$dst" "$BACKUP_DIR/$(basename "$dst")"
    fi

    mkdir -p "$(dirname "$dst")"
    ln -sf "$src" "$dst"
    echo "  Linked     : $dst -> $src"
}

# ---------------------------------------------------------------------------
# Migrate .zshrc
# ---------------------------------------------------------------------------
echo ""
echo "[1/3] Migrating shell config..."
migrate_link "$REPO_DIR/.zshrc" "$HOME/.zshrc"

# ---------------------------------------------------------------------------
# Migrate every entry in .config/
# ---------------------------------------------------------------------------
echo ""
echo "[2/3] Migrating .config/ entries..."
mkdir -p "$HOME/.config"
for src_item in "$REPO_DIR/.config"/*; do
    [ -e "$src_item" ] || continue
    name="$(basename "$src_item")"
    migrate_link "$src_item" "$HOME/.config/$name"
done

# ---------------------------------------------------------------------------
# Migrate .local/share/ entries
# ---------------------------------------------------------------------------
echo ""
echo "[3/3] Migrating .local/share/ entries..."
if [ -d "$REPO_DIR/.local" ]; then
    for subdir in "$REPO_DIR/.local"/*/; do
        [ -d "$subdir" ] || continue
        local_sub="$(basename "$subdir")"
        mkdir -p "$HOME/.local/$local_sub"
        for src_item in "$subdir"*; do
            [ -e "$src_item" ] || continue
            name="$(basename "$src_item")"
            migrate_link "$src_item" "$HOME/.local/$local_sub/$name"
        done
    done
fi

# ---------------------------------------------------------------------------
# Reload shell config
# ---------------------------------------------------------------------------
echo ""
echo "=========================================="
echo "  Migration complete!"
echo "  Old dotfiles backed up to: $BACKUP_DIR"
echo ""
echo "  Run: source ~/.zshrc"
echo "  Or open a new terminal."
echo "=========================================="
