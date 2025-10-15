#!/usr/bin/env bash
set -euo pipefail

profile="${1:-dev}"
boot_crate="${2:-rtos-bootloader}"
boot_bin="${3:-bootx64}"
boot_efi_name="${4:-BOOTX64.EFI}"
debug_mode="${5:-off}"
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

# ---- build KERNEL (ELF) ------------------------------------------------------
KERNEL_CRATE="rtos-kernel"
KERNEL_BIN="rtos-kernel"
KERNEL_TARGET="x86_64-unknown-none"

KERNEL_BUILD_DIR="${BUILD_ROOT}/${KERNEL_CRATE}"
export CARGO_TARGET_DIR="${KERNEL_BUILD_DIR}/target"
mkdir -p "${CARGO_TARGET_DIR}"

if [[ "$profile" == "release" ]]; then
  cargo +nightly build -p "$KERNEL_CRATE" --release --target "$KERNEL_TARGET" -Z build-std=core,compiler_builtins
  k_out="release"
else
  cargo +nightly build -p "$KERNEL_CRATE" --target "$KERNEL_TARGET" -Z build-std=core,compiler_builtins
  k_out="debug"
fi

kernel_elf="${CARGO_TARGET_DIR}/${KERNEL_TARGET}/${k_out}/${KERNEL_BIN}"

# ---- fresh ESP staging -------------------------------------------------------
ESP_DIR="${BOOT_BUILD_DIR}/esp"
rm -rf "${ESP_DIR}"
mkdir -p "${ESP_DIR}/EFI/BOOT"

# re-export the bootloader target dir temporarily to copy from it
BOOT_TARGET_DIR="${BUILD_ROOT}/${boot_crate}/target"
cp -f "${BOOT_TARGET_DIR}/${target}/${out_dir}/${boot_bin}.efi" "${ESP_DIR}/EFI/BOOT/${boot_efi_name}"

# ---- RTOSK pack (ELF -> RTOSK) ----------------------------------------------
KERNEL_RK_NAME="KERNEL.RTOSK"
kernel_rk="${ESP_DIR}/EFI/BOOT/${KERNEL_RK_NAME}"
cargo run --quiet --release -p rtkgen -- "${kernel_elf}" "${kernel_rk}"

# sanity check
if [[ ! -f "${kernel_rk}" ]]; then
  echo "ERROR: ${kernel_rk} not found after packing!" >&2
  echo "ESP content:" >&2
  find "${ESP_DIR}" -maxdepth 3 -type f -printf '%P\n' | sed 's/^/  /' >&2 || true
  exit 1
fi
echo "ESP ready:"
find "${ESP_DIR}/EFI/BOOT" -maxdepth 1 -type f -printf '  %f\n' | sort

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
if [[ "$debug_mode" == "on" ]]; then
  # Start paused so GDB can attach before the INT3 fires or early code runs
  extra_qemu_args+=(-s -S)
  echo "debug: QEMU started paused; attach with 'gdb -ex \"target remote :1234\"' then 'c'"
fi

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
