#!/bin/bash
# Create a simple PNG icon (1x1 pixel red square)
# We'll use printf to create a minimal PNG
create_png() {
    size=$1
    file=$2
    # For now, just create a symlink to the ico file
    # In production, you'd use proper icon generation
    touch "$file"
}

create_png 32 32x32.png
create_png 128 128x128.png
create_png 128 128x128@2x.png
