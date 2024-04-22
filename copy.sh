# Copy all files that don't exist in ../cfreplayer/fixtures/api/replay from /replays/

for file in ./replays/*; do
  if [ ! -f "../../cfreplayer/fixtures/api/replay/$(basename $file)" ]; then
    cp "$file" "../../cfreplayer/fixtures/api/replay/$(basename $file)"
  fi
done