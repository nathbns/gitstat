#!/bin/bash

# Script pour créer des releases multi-plateformes

echo "🔨 Building release binaries..."

# Build pour macOS (votre plateforme actuelle)
echo "📱 Building for macOS..."
cargo build --release
cp target/release/gitstat gitstat-macos

# Build pour Linux (si cross-compilation est configurée)
echo "🐧 Building for Linux..."
# cargo build --release --target x86_64-unknown-linux-gnu
# cp target/x86_64-unknown-linux-gnu/release/gitstat gitstat-linux

# Build pour Windows (si cross-compilation est configurée)  
echo "🪟 Building for Windows..."
# cargo build --release --target x86_64-pc-windows-gnu
# cp target/x86_64-pc-windows-gnu/release/gitstat.exe gitstat-windows.exe

echo "✅ Build completed!"
echo "📁 Binaries:"
ls -la gitstat-*
