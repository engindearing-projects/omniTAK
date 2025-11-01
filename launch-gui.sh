#!/bin/bash
# OmniTAK GUI Launcher for WSL2
# Automatically sets the correct display backend

echo "🚀 Launching omniTAK GUI..."
echo "Setting X11 backend for WSL2 compatibility..."

export WINIT_UNIX_BACKEND=x11
export WAYLAND_DISPLAY=""

# Build if needed
if [ ! -f "./target/release/omnitak-gui" ]; then
    echo "Building GUI..."
    cargo build --bin omnitak-gui --release
fi

# Launch the GUI
./target/release/omnitak-gui

echo "✅ GUI launched successfully!"
