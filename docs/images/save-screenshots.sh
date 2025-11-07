#!/bin/bash
# Script to help organize GUI screenshots

echo "OmniTAK GUI Screenshot Organizer"
echo "================================="
echo ""
echo "This script will help you copy your GUI screenshots to the docs/images directory."
echo ""

# Check if screenshots exist on Desktop
if ls ~/Desktop/Screenshot*.png 1> /dev/null 2>&1; then
    echo "Found screenshots on Desktop. Recent ones:"
    ls -lt ~/Desktop/Screenshot*.png | head -4
    echo ""
    echo "Please manually copy the 4 GUI screenshots to:"
    echo "  - docs/images/gui-dashboard.png"
    echo "  - docs/images/gui-connections.png"  
    echo "  - docs/images/gui-messages.png"
    echo "  - docs/images/gui-settings.png"
else
    echo "No screenshots found on Desktop."
    echo "Please take screenshots and save them to docs/images/"
fi

echo ""
echo "After copying, run: git add docs/images/*.png"
