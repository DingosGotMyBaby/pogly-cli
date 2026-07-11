#!/usr/bin/env bash
# Spawns a looping rick roll, teleports it around a 1920x1080 canvas,
# then deletes it. Usage: ./rickroll.sh [duration-seconds]
set -euo pipefail

WIDTH=480
HEIGHT=270
DURATION=${1:-10}

created=$(pogly --json elements add media \
  --source "https://www.youtube.com/watch?v=dQw4w9WgXcQ" \
  --width $WIDTH --height $HEIGHT --autoplay --loop --x 720 --y 405)
id=$(printf '%s' "$created" | grep -m1 '"id"' | grep -o '[0-9]\+')
echo "Spawned media element $id - never gonna give you up (for ${DURATION}s)"

cleanup() {
  pogly elements delete "$id" >/dev/null
  echo "Deleted element $id - rick has left the building"
}
trap cleanup EXIT

end=$((SECONDS + DURATION))
while ((SECONDS < end)); do
  x=$((RANDOM % (1920 - WIDTH)))
  y=$((RANDOM % (1080 - HEIGHT)))
  pogly elements update "$id" --x "$x" --y "$y" >/dev/null
  sleep 0.15
done
