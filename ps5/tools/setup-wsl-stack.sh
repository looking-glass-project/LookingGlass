#!/usr/bin/env bash
# Set up the PS5 build stack on Arch Linux (native or WSL2: `wsl --install archlinux`). Idempotent.
#
#   bash ps5/tools/setup-wsl-stack.sh [PS5_IP]
#
# Installs the host packages PS5_Vulkan's author uses (Arch/CachyOS), Rust nightly + rust-src,
# clones mihawk-99's PS5 stack side by side in ~/ps5stack (PS5_VulkanTemplate, PS5_Vulkan,
# PS5_Mesa, PS5_PayloadSDK) and runs the template's bootstrap, which builds the payload SDK fork,
# the native tool, libc.prx and the RADV release archive (~10 min on a fast PC).
set -euo pipefail
STACK=${PS5_STACK:-$HOME/ps5stack}
SUDO=""; [[ $(id -u) -ne 0 ]] && SUDO=sudo
command -v pacman >/dev/null || { echo "expects Arch Linux (WSL: wsl --install archlinux)"; exit 2; }
echo "==> packages"
$SUDO pacman -Syu --needed --noconfirm \
  base-devel clang llvm lld make python python-pip git curl wget unzip tar pkgconf \
  cmake ninja meson rsync zstd glslang spirv-tools libclc spirv-llvm-translator \
  python-mako python-markupsafe python-yaml python-packaging python-numpy python-pillow \
  bison flex rustup openbsd-netcat
echo "==> rust"
rustup toolchain list | grep -q nightly || rustup toolchain install nightly --profile minimal -c rust-src
rustup default nightly >/dev/null
echo "==> mihawk-99 PS5 stack in $STACK"
mkdir -p "$STACK"
[[ -d $STACK/PS5_VulkanTemplate/.git ]] || git clone https://github.com/mihawk-99/PS5_VulkanTemplate.git "$STACK/PS5_VulkanTemplate"
if [[ -n ${1:-} ]]; then
  [[ -d $STACK/PS5_Vulkan/.git ]] || git clone https://github.com/mihawk-99/PS5_Vulkan.git "$STACK/PS5_Vulkan"
  grep -q "^PS5_HOST=" "$STACK/PS5_Vulkan/.env" 2>/dev/null || echo "PS5_HOST=$1" >> "$STACK/PS5_Vulkan/.env"
fi
bash "$STACK/PS5_VulkanTemplate/ps5/tools/bootstrap.sh" || true   # reports the console address as missing until set
if [[ ! -f $STACK/PS5_Vulkan/build/ps5vkctl/ps5vkctl.elf ]]; then
  (cd "$STACK/PS5_Vulkan" && PS5_PAYLOAD_SDK=$STACK/PS5_Vulkan/.deps/native/ps5-payload-sdk bash tools/build-ps5vkctl.sh) || true
fi
A=$STACK/PS5_Vulkan/.deps/native/radv-release/lib/libvulkan_radeon.ps5.a
[[ -f $A ]] && echo "==> ready: RADV archive $A" || { echo "==> RADV archive missing; see the bootstrap output above" >&2; exit 1; }
