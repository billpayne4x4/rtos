#!/usr/bin/env bash
set -euo pipefail

# Inputs (all optional):
#   $1 = profile: dev|release  (default: release)
# Env:
#   BUILD_ROOT overrides output root directory (default: build)

selected_profile="${1:-release}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd -- "${script_dir}/.." && pwd)"
build_root="${BUILD_ROOT:-build}"

inspect_crate_rel="tools/rtosk-inspect"
inspect_build_dir="${workspace_root}/${build_root}/rtosk-inspect"
inspect_target_dir="${inspect_build_dir}/target"
inspect_manifest_path="${workspace_root}/${inspect_crate_rel}/Cargo.toml"

mkdir -p "${inspect_target_dir}"

cargo_profile_args=()
case "${selected_profile}" in
  release) cargo_profile_args+=(--release) ;;
  dev)     : ;;
  *) echo "unknown profile: ${selected_profile} (use: dev|release)"; exit 2 ;;
esac

export CARGO_TARGET_DIR="${inspect_target_dir}"

cd "${workspace_root}"
cargo build --manifest-path "${inspect_manifest_path}" -p rtosk-inspect "${cargo_profile_args[@]}"

binary_dir="${inspect_target_dir}/${selected_profile}"
binary_path="${binary_dir}/rtosk-inspect"

if [[ ! -x "${binary_path}" ]]; then
  # Fallback for toolchains that always use 'release' dir when --release is set
  if [[ -x "${inspect_target_dir}/release/rtosk-inspect" ]]; then
    binary_path="${inspect_target_dir}/release/rtosk-inspect"
  elif [[ -x "${inspect_target_dir}/debug/rtosk-inspect" ]]; then
    binary_path="${inspect_target_dir}/debug/rtosk-inspect"
  fi
fi

echo "rtosk-inspect built:"
echo "  ${binary_path}"
