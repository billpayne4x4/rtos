#!/usr/bin/env bash
set -euo pipefail

# Usage: uefi-run.sh [dev|release] [boot_crate] [boot_bin] [boot_efi_name] [debug:on|off]
profile="${1:-dev}"                       # dev | release
boot_crate="${2:-rtos-bootloader}"        # bootloader workspace member
boot_bin="${3:-bootx64}"                  # [[bin]].name in bootloader
boot_efi_name="${4:-BOOTX64.EFI}"         # filename on ESP
debug_mode="${5:-off}"                    # on | off
target="x86_64-unknown-uefi"

BUILD_ROOT="${BUILD_ROOT:-build}"

# ---- build BOOTLOADER --------------------------------------------------------
BOOT_BUILD_DIR="${BUILD_ROOT}/${boot_crate}"
export CARGO_TARGET_DIR="${BOOT_BUILD_DIR}/target"
mkdir -p "${CARGO_TARGET_DIR}"

if [[ "$profile" == "release" ]]; then
  cargo +nightly build -p "$boot_crate" --release --target "$target" -Z build-std=core,compiler_builtins
  out_dir="release"
else
  cargo +nightly build -p "$boot_crate" --target "$target" -Z build-std=core,compiler_builtins
  out_dir="debug"
fi

boot_efi="${CARGO_TARGET_DIR}/${target}/${out_dir}/${boot_bin}.efi"

# ---- build KERNEL (UEFI app for now) ----------------------------------------
KERNEL_CRATE="rtos-kernel"
KERNEL_BIN="kernelx64"
KERNEL_BUILD_DIR="${BUILD_ROOT}/${KERNEL_CRATE}"
export CARGO_TARGET_DIR="${KERNEL_BUILD_DIR}/target"
mkdir -p "${CARGO_TARGET_DIR}"

if [[ "$profile" == "release" ]]; then
  cargo +nightly build -p "$KERNEL_CRATE" --release --target "$target" -Z build-std=core,compiler_builtins
  k_out="release"
else
  cargo +nightly build -p "$KERNEL_CRATE" --target "$target" -Z build-std=core,compiler_builtins
  k_out="debug"
fi

kernel_efi="${CARGO_TARGET_DIR}/${target}/${k_out}/${KERNEL_BIN}.efi"

# ---- stage ESP ---------------------------------------------------------------
ESP_DIR="${BOOT_BUILD_DIR}/esp"
mkdir -p "${ESP_DIR}/EFI/BOOT"
cp -f "${boot_efi}"   "${ESP_DIR}/EFI/BOOT/${boot_efi_name}"
cp -f "${kernel_efi}" "${ESP_DIR}/EFI/BOOT/KERNELX64.EFI"   # bootloader expects this path

# ---- OVMF auto-detect (prefer 4M pflash) ------------------------------------
ovmf_code_candidates=(
  "${OVMF_CODE:-}"
  /usr/share/OVMF/OVMF_CODE_4M.fd
  /usr/share/OVMF/OVMF_CODE_4M.secboot.fd
  /usr/share/OVMF/OVMF_CODE_4M.snakeoil.fd
  /usr/share/OVMF/OVMF_CODE.fd
  /usr/share/edk2-ovmf/x64/OVMF_CODE_4M.fd
  /usr/share/edk2/ovmf/OVMF_CODE_4M.fd
  /usr/share/ovmf/x64/OVMF_CODE_4M.fd
  /usr/share/edk2-ovmf/x64/OVMF_CODE.fd
  /usr/share/edk2/ovmf/OVMF_CODE.fd
  /usr/share/ovmf/x64/OVMF_CODE.fd
)
ovmf_vars_candidates=(
  "${OVMF_VARS:-}"
  /usr/share/OVMF/OVMF_VARS_4M.fd
  /usr/share/OVMF/OVMF_VARS_4M.ms.fd
  /usr/share/OVMF/OVMF_VARS_4M.snakeoil.fd
  /usr/share/OVMF/OVMF_VARS.fd
  /usr/share/edk2-ovmf/x64/OVMF_VARS_4M.fd
  /usr/share/edk2/ovmf/OVMF_VARS_4M.fd
  /usr/share/ovmf/x64/OVMF_VARS_4M.fd
  /usr/share/edk2-ovmf/x64/OVMF_VARS.fd
  /usr/share/edk2/ovmf/OVMF_VARS.fd
  /usr/share/ovmf/x64/OVMF_VARS.fd
)

ovmf_code=""; ovmf_vars=""
for f in "${ovmf_code_candidates[@]}";  do [[ -n "$f" && -f "$f" ]] && { ovmf_code="$f"; break; };  done
for f in "${ovmf_vars_candidates[@]}";  do [[ -n "$f" && -f "$f" ]] && { ovmf_vars="$f"; break; };  done
[[ -n "$ovmf_code" ]] || { echo "OVMF CODE not found. Install 'ovmf' or set OVMF_CODE=/path/to/OVMF_CODE*.fd"; exit 1; }

extra_qemu_args=()
[[ "$debug_mode" == "on" ]] && extra_qemu_args+=(-s -S)

if [[ -n "$ovmf_vars" ]]; then
  mkdir -p "${BOOT_BUILD_DIR}/ovmf"
  cp -f "$ovmf_vars" "${BOOT_BUILD_DIR}/ovmf/OVMF_VARS.fd"
  ovmf_vars_rw="${BOOT_BUILD_DIR}/ovmf/OVMF_VARS.fd"

  exec qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
    -drive if=pflash,format=raw,file="$ovmf_vars_rw" \
    -drive file=fat:rw:"${ESP_DIR}",format=raw \
    -serial stdio \
    "${extra_qemu_args[@]}"
else
  exec qemu-system-x86_64 \
    -bios "${ovmf_code}" \
    -drive file=fat:rw:"${ESP_DIR}",format=raw \
    -serial stdio \
    "${extra_qemu_args[@]}"
fi
