#!/usr/bin/env bash
# Build Looking Glass as a PS5 title: ps5/dist/PPSA99641/ (eboot.bin, sce_sys, sce_module/libc.prx).
#   ps5/build.sh                 release build
# Needs the stack from ps5/tools/setup-wsl-stack.sh (default ~/ps5stack): PS5_Vulkan with its RADV
# release archive, and PS5_VulkanTemplate's payload SDK fork.
set -euo pipefail
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
src=$(dirname "$here")
STACK=${PS5_STACK:-$HOME/ps5stack}
export PS5_VULKAN=${PS5_VULKAN:-$STACK/PS5_Vulkan}
export PS5_SDK=${PS5_SDK:-$STACK/PS5_VulkanTemplate/.deps/native/ps5-payload-sdk}
[[ -d $PS5_SDK ]] || { echo "no SDK at $PS5_SDK (run ps5/tools/setup-wsl-stack.sh)" >&2; exit 2; }
[[ -f $PS5_VULKAN/.deps/native/radv-release/lib/libvulkan_radeon.ps5.a ]] ||
    { echo "no RADV release archive in $PS5_VULKAN (run ps5/tools/setup-wsl-stack.sh)" >&2; exit 2; }
export PS5_PAYLOAD_SDK=$PS5_SDK PS5_PLATFORM_SDK=$PS5_SDK
export CC_x86_64_sie_ps5_std=$PS5_SDK/bin/prospero-clang AR_x86_64_sie_ps5_std=$PS5_SDK/bin/prospero-ar
export CARGO_TARGET_X86_64_SIE_PS5_STD_LINKER=$here/link-title.sh
export RUSTC_BOOTSTRAP=1
export PS5_STD_RT_TITLE=1   # libc from the console modules, not ps5-std-rt's bundled copy
cd "$src"
cargo build --release "$@" \
  --target "$here/targets/x86_64-sie-ps5-std.json" \
  -Zjson-target-spec -Zbuild-std=std,panic_abort
ls -la "$here/dist/PPSA99641/"
