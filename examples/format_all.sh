#!/usr/bin/sh

# Get the current directory
current_dir=$(pwd)

# Loop through each subdirectory
for dir in */; do
    if [ -d "$dir" ]; then
        echo "Formatting in $dir"
        (cd "$dir" && cargo +nightly fmt)
    fi
done
