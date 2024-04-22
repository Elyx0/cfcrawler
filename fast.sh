#!/bin/bash
# Write a shell script that removes the extension of all files in the /api folder
for file in ./replays/*; do
mv "$file" "${file%.*}"
done