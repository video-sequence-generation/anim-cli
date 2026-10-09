#!/bin/sh
# Render an animation via the anim-cli Docker image.
#
# Usage: ./generate-video.sh ["prompt"] [duration-seconds]
# Example: ./generate-video.sh "neon radar sweep" 6
set -eu

mkdir -p output
docker run -e GEMINI_API_KEY=$(cat .env | cut -d'=' -f2 | tr -d '"') \
  -v $(pwd)/output:/home/appuser \
  anim-cli render "${1:-an ocean wave}" -d "${2:-6}" -o out.mp4