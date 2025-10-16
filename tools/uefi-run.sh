#!/usr/bin/env bash
set -euo pipefail

profile="${1:-dev}"
boot_crate="${2:-rtos-bootloader}"
boot_bin="${3:-bootx64}"
boot_efi_name="${4:-BOOTX64.EFI}"
debug_mode="${5:-off}"

target="x86_64-unknown-uefi"
KERNEL_CRATE="rtos-kernel"
KERNEL_BIN="rtos-kernel"
KERNEL_TARGET="x86_64-unknown-none"

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
echo "Kernel ELF: ${kernel_elf}"
[[ -f "${kernel_elf}" ]] || { echo "ERROR: kernel ELF not found at ${kernel_elf}" >&2; exit 2; }
echo "Kernel ELF size: $(stat -c%s "${kernel_elf}") bytes"

# ---- make a FLAT BIN from sections we actually need -------------------------
KERNEL_BIN_DIR="${BUILD_ROOT}/${KERNEL_CRATE}"
mkdir -p "${KERNEL_BIN_DIR}"
kernel_bin="${KERNEL_BIN_DIR}/kernel.bin"

# Extract .text + .rodata + .data as a contiguous flat image.
# (Avoids weird PHDR/SHDR edge cases in tiny tests; matches future non-ELF world.)
objcopy -O binary -j .text -j .rodata -j .data "${kernel_elf}" "${kernel_bin}"
echo "Kernel BIN: ${kernel_bin} ($(stat -c%s "${kernel_bin}") bytes)"

# ---- fresh ESP staging -------------------------------------------------------
ESP_DIR="${BOOT_BUILD_DIR}/esp"
rm -rf "${ESP_DIR}"
mkdir -p "${ESP_DIR}/EFI/BOOT"
cp -f "${boot_efi}" "${ESP_DIR}/EFI/BOOT/${boot_efi_name}"

# ---- build PACKER and pack BIN -> RTOSK -------------------------------------
PACKER_BUILD_DIR="${BUILD_ROOT}/rtosk-gen"
export CARGO_TARGET_DIR="${PACKER_BUILD_DIR}/target"
cargo +nightly build --release -p rtosk-gen
packer_bin="${PACKER_BUILD_DIR}/target/release/rtosk-gen"
[[ -x "${packer_bin}" ]] || { echo "ERROR: packer not built: ${packer_bin}" >&2; exit 3; }

KERNEL_RK_CANON="${BUILD_ROOT}/${KERNEL_CRATE}/KERNEL.RTOSK"
rm -f "${KERNEL_RK_CANON}"

# Args: <input> <output.rtosk> [entry_va] [page_size]
# We pass entry 0x200000 to match your linker & bootloader jump.
"${packer_bin}" "${kernel_bin}" "${KERNEL_RK_CANON}" 0x200000 0x1000

[[ -f "${KERNEL_RK_CANON}" ]] || { echo "ERROR: ${KERNEL_RK_CANON} not produced by packer" >&2; exit 4; }
rk_size=$(stat -c%s "${KERNEL_RK_CANON}")
echo "Packed: ${KERNEL_RK_CANON} (size ${rk_size} bytes)"

cp -f "${KERNEL_RK_CANON}" "${ESP_DIR}/EFI/BOOT/KERNEL.RTOSK"

echo "ESP ready:"
find "${ESP_DIR}/EFI/BOOT" -maxdepth 1 -type f -printf '  %f (%s bytes)\n' | sort

# ---- optional: rtosk-inspect (best-effort) ----------------------------------
INSPECT_BUILD_DIR="${BUILD_ROOT}/rtosk-inspect"
export CARGO_TARGET_DIR="${INSPECT_BUILD_DIR}/target"
cargo +nightly build --release -p rtosk-inspect || true
echo "Inspector (if built): ${INSPECT_BUILD_DIR}/target/release/rtosk-inspect"

# ---- OVMF auto-detect and QEMU ----------------------------------------------
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
[[ -n "$ovmf_code" ]] || { echo "OVMF CODE not found. Install 'ovmf' or set OVMF_CODE=/path/to/OVMF_CODE*.fd"; exit 7; }

extra_qemu_args=()
if [[ "$debug_mode" == "on" ]]; then
  extra_qemu_args+=(-s -S)
  echo "debug: QEMU started paused; attach with 'gdb -ex \"target remote :1234\"' then 'c'"
fi

if [[ -n "$ovmf_vars" ]] then
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
