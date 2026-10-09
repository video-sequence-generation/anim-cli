#!/bin/sh
# Render an animation via the anim-cli Docker image.
#
# Usage: ./generate-video.sh ["prompt"] [duration-seconds]
# Example: ./generate-video.sh "neon radar sweep" 6
#
# Credentials and endpoints are read from .env in this directory:
#   GEMINI_API_KEY    key for the hosted Gemini API
#   LOCAL_API_URL     base URL of an OpenAI-compatible local server.
#                     When set, the render is sent to that server via
#                     --local-api and GEMINI_API_KEY is not required.
#                     When unset/empty, Gemini is used instead.
#   LOCAL_API_MODEL   optional model name override for the local server
set -eu

# Resolve paths relative to the script, not the caller's cwd.
cd "$(dirname "$0")"

prompt="${1:-an ocean wave}"
duration="${2:-6}"

# Read KEY from .env. Tolerates optional surrounding quotes and blank lines.
read_env() {
  value=$(sed -n "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*//p" .env 2>/dev/null | head -n 1)
  printf '%s' "$value" | sed -e 's/^["'\'']//' -e 's/["'\'']$//'
}

gemini_key=$(read_env GEMINI_API_KEY)
local_url=$(read_env LOCAL_API_URL)
local_model=$(read_env LOCAL_API_MODEL)

if [ -z "$gemini_key" ] && [ -z "$local_url" ]; then
  echo "error: set GEMINI_API_KEY and/or LOCAL_API_URL in .env" >&2
  exit 1
fi

if [ ! -f .env ]; then
  echo "error: .env not found in $(pwd)" >&2
  exit 1
fi

# Build the docker argument list in "$@". $1/$2 are already captured above.
set -- -v "$(pwd)/output:/home/appuser"
if [ -n "$gemini_key" ]; then
  set -- "$@" -e "GEMINI_API_KEY=$gemini_key"
fi
if [ -n "$local_url" ]; then
  set -- "$@" -e "LOCAL_API_URL=$local_url"
fi
if [ -n "$local_model" ]; then
  set -- "$@" -e "LOCAL_API_MODEL=$local_model"
fi

mkdir -p output

# --local-api is an anim-cli flag, not a docker flag, so it is added here
# rather than to the "$@" list above.
if [ -n "$local_url" ]; then
  docker run "$@" anim-cli render "$prompt" -d "$duration" --local-api -o out.mp4
else
  docker run "$@" anim-cli render "$prompt" -d "$duration" -o out.mp4
fi

echo "wrote $(pwd)/output/out.mp4"