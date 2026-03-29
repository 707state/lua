#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
export ANDROID_LINKER_WRAPPER_NAME="$(basename "$0")"
exec "${script_dir}/android-linker.sh" "$@"
