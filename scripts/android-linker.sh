#!/usr/bin/env bash

set -euo pipefail

ndk_home="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "${ndk_home}" ]]; then
  echo "ANDROID_NDK_HOME or ANDROID_NDK_ROOT must be set" >&2
  exit 1
fi

host_os="$(uname -s)"
host_arch="$(uname -m)"
case "${host_os}:${host_arch}" in
  Linux:x86_64) host_tag="linux-x86_64" ;;
  Darwin:x86_64) host_tag="darwin-x86_64" ;;
  Darwin:arm64) host_tag="darwin-x86_64" ;;
  *) echo "unsupported host for Android NDK: ${host_os} ${host_arch}" >&2; exit 1 ;;
esac

clang="${ndk_home}/toolchains/llvm/prebuilt/${host_tag}/bin/clang"
if [[ ! -x "${clang}" ]]; then
  echo "missing NDK clang: ${clang}" >&2
  exit 1
fi

api_level="${ANDROID_API_LEVEL:-${ANDROID_PLATFORM:-21}}"
api_level="${api_level#android-}"

script_name="${ANDROID_LINKER_WRAPPER_NAME:-$(basename "$0")}"
case "${script_name}" in
  android-linker-aarch64-linux-android.sh)
    target="aarch64-linux-android${api_level}"
    ;;
  android-linker-armv7-linux-androideabi.sh)
    target="armv7a-linux-androideabi${api_level}"
    ;;
  android-linker-x86_64-linux-android.sh)
    target="x86_64-linux-android${api_level}"
    ;;
  android-linker-i686-linux-android.sh)
    target="i686-linux-android${api_level}"
    ;;
  *)
    echo "unsupported Android linker wrapper: ${script_name}" >&2
    exit 1
    ;;
esac

exec "${clang}" "--target=${target}" "$@"
