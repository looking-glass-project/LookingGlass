#!/usr/bin/env bash
# Looking Glass for PS5 - the linker cargo calls (CARGO_TARGET_X86_64_SIE_PS5_STD_LINKER).
#
# rustc hands over its objects and rlibs; this links them the way PS5_VulkanTemplate links a
# title (its ps5/tools/link-title.sh, MIT, Mihawk): PS5_Vulkan's RADV release archive and link
# recipe (tools/radv-link.sh: the SDK fork's platform layer, heap/thread wraps, libc names bound
# to ps5_*), the app CRT, AGC stubs, then the native tool's ELF -> signed eboot.bin, and the
# title folder ps5/dist/<TITLE_ID>/ with sce_sys/ and sce_module/libc.prx.
#
# Environment (set by ps5/build.sh): PS5_VULKAN (PS5_Vulkan checkout), PS5_SDK (the template's
# payload SDK fork, .deps/native/ps5-payload-sdk).
set -euo pipefail
here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
vulkan=${PS5_VULKAN:?set PS5_VULKAN}
sdk=${PS5_SDK:?set PS5_SDK}
work=$here/build/link
mkdir -p "$work/obj" "$work/stubs"
printf '%q ' "$@" > "$work/rustc-args.txt"

# What rustc passes: keep its objects and archives, take -o; the rest (crt, -l, -L, -Wl,..)
# is replaced by the recipe below.
inputs=() out=""
while (($#)); do
    case $1 in
        -o) out=$2; shift ;;
        -o*) out=${1#-o} ;;
        *.o|*.rlib|*.a) [[ -f $1 ]] && inputs+=("$1") ;;
    esac
    shift
done
[[ -n $out ]] || { echo "link-title.sh: no -o" >&2; exit 2; }

archive=${RADV_ARCHIVE:-$vulkan/.deps/native/radv-release/lib/libvulkan_radeon.ps5.a}
tool="$vulkan/build/host/ps5-native-tool"
native="$vulkan/tooling/native"
export PS5_CLANG=${PS5_CLANG:-$(command -v clang)}
for file in "$archive" "$tool" "$sdk/bin/prospero-lld" "$vulkan/tools/radv-link.sh" "$vulkan/runtime/libc.prx"; do
    [[ -e $file ]] || { echo "missing: $file" >&2; exit 2; }
done
cc() { PS5_PAYLOAD_SDK="$sdk" sh "$vulkan/tooling/prospero-clang18" "$@"; }
cc -std=c++20 -O2 -fno-exceptions -fno-rtti -c "$native/app_crt.cpp" -o "$work/obj/app_crt.o"
stub() {
    local library=$1 source=$2
    cc -std=c11 -O2 -fPIC -c "$vulkan/$source" -o "$work/obj/${library}_stub.o"
    "$sdk/bin/prospero-lld" --shared -soname "${library}.prx" -o "$work/stubs/${library}.so" "$work/obj/${library}_stub.o"
}
stub libSceAgc vendor/ps5/sdk/stubs/agc_canary_link_stub.c
stub libSceAgcDriver vendor/ps5/sdk/stubs/agc_driver_canary_link_stub.c
# shellcheck source=/dev/null
source "$vulkan/tools/radv-link.sh"
radv_link_recipe "$vulkan" "$sdk" "$archive" || exit 2
# Names both sides define. ps5-std-rt implements some libc calls (opendir, readdir, fstatat,
# clock_nanosleep, ...) with the FreeBSD 12 layouts Rust's std expects; the recipe binds the same
# names to the platform layer's ps5_* for RADV, which was built against the SDK's own layouts.
# Each side must keep its own, so in the Rust objects only, those names become __lg_<name>
# (definitions and references alike); RADV and the platform layer keep theirs.
nm="$sdk/bin/llvm-nm" objcopy="$sdk/bin/llvm-objcopy" ar="$sdk/bin/llvm-ar"
owned=$( { printf '%s\n' "${radv_link_flags[@]}" | sed -n 's/^--defsym=\([^=]*\)=.*/\1/p'
    for lib in "$archive" "$sdk/target/lib/libps5platform.a" "$sdk/target/lib/libc++.a" \
            "$sdk/target/lib/libc++abi.a" "$sdk/target/lib/libunwind.a"; do
        "$nm" --defined-only --extern-only "$lib" 2>/dev/null | awk 'NF == 3 { print $3 }'
    done; } | sort -u)
rust_defined=$("$nm" --defined-only --extern-only "${inputs[@]}" 2>/dev/null | awk 'NF == 3 && $2 != "W" && $2 != "V" { print $3 }' | sort -u)
conflicts=$(comm -12 <(echo "$rust_defined") <(echo "$owned"))
if [[ -n $conflicts ]]; then
    echo "link-title.sh: renaming in the Rust objects: $(echo $conflicts)"
    map="$work/rename.map"
    awk '{ print $1, "__lg_" $1 }' <<<"$conflicts" > "$map"
    pattern=$(paste -sd'|' <<<"$conflicts")
    renamed=()
    rm -rf "$work/renamed"; mkdir -p "$work/renamed"
    n=0
    for input in "${inputs[@]}"; do
        n=$((n + 1))
        # read the listing whole before searching it: piped into grep -q under pipefail, llvm-nm's
        # SIGPIPE turns a hit into a miss (the template's link-title.sh notes the same)
        symbols=$("$nm" "$input" 2>/dev/null || true)
        if ! grep -qE " ($pattern)\$" <<<"$symbols"; then renamed+=("$input"); continue; fi
        case $input in
            *.o) "$objcopy" --redefine-syms="$map" "$input" "$work/renamed/$n.o"; renamed+=("$work/renamed/$n.o") ;;
            *) dir="$work/renamed/$n"; mkdir -p "$dir"
               (cd "$dir" && "$ar" x "$input")
               rm -f "$dir/lib.rmeta"
               for member in "$dir"/*; do "$objcopy" --redefine-syms="$map" "$member" 2>/dev/null || rm -f "$member"; done
               "$ar" rcs "$work/renamed/$n.a" "$dir"/*
               renamed+=("$work/renamed/$n.a") ;;
        esac
    done
    inputs=("${renamed[@]}")
fi

# A title exports nothing (the native converter refuses application exports); Rust's
# #[no_mangle] items would otherwise land in .dynsym. Imports are not affected.
printf '{\n    local: *;\n};\n' > "$work/all-local.map"

# Optional (weak) imports of the Rust side that no console module exports, e.g. std's
# posix_spawn_file_actions_addchdir_np: the native tool refuses an import it cannot bind, and
# the code using them already checks for null, so bind them to address 0 ("not present").
exports=$(for library in "$sdk"/target/lib/*.so "$work/stubs/"*.so; do
        case ${library##*/} in libkernel_sys.so | libScePosixForWebKit.so) continue ;; esac
        "$nm" -D --defined-only "$library" 2>/dev/null | awk '{ print $NF }'
    done; for lib in "$archive" "$sdk/target/lib/libps5platform.a" "$sdk/target/lib/libc++.a" \
            "$sdk/target/lib/libc++abi.a" "$sdk/target/lib/libunwind.a"; do
        "$nm" --defined-only --extern-only "$lib" 2>/dev/null | awk 'NF == 3 { print $3 }'
    done)
rust_weak=$("$nm" --undefined-only "${inputs[@]}" 2>/dev/null | awk '$1 == "w" { print $2 }' | sort -u)
rust_def=$("$nm" --defined-only --extern-only "${inputs[@]}" 2>/dev/null | awk 'NF == 3 { print $3 }' | sort -u)
absent=$(comm -23 <(echo "$rust_weak") <( { echo "$exports"; echo "$rust_def"; } | sort -u) | grep -v '^$' || true)
weak_flags=()
for name in $absent; do weak_flags+=("--defsym=$name=0"); done
[[ -n $absent ]] && echo "link-title.sh: optional imports bound to null: $(echo $absent)"

"$sdk/bin/prospero-lld" "${radv_linker_script[@]}" --eh-frame-hdr "${radv_link_flags[@]}" "${weak_flags[@]}" \
    --version-script "$native/app-symbols.map" --version-script "$work/all-local.map" --exclude-libs=ALL \
    -e _start -o "$work/llvm-pie.elf" \
    "$work/obj/app_crt.o" --start-group "${inputs[@]}" --end-group \
    "$work/stubs/libSceAgc.so" "$work/stubs/libSceAgcDriver.so" \
    "${radv_link_inputs[@]}" \
    --as-needed "$sdk"/target/lib/*.so

# Imports no module a title loads exports are null at run time: refuse them here.
null_imports=$(comm -23 \
    <("$sdk/bin/llvm-nm" -D --undefined-only "$work/llvm-pie.elf" | awk '$1 == "U" { sub(/@.*/, "", $2); print $2 }' | sort -u) \
    <(for library in "$sdk"/target/lib/*.so "$work/stubs/"*.so; do
        case ${library##*/} in libkernel_sys.so | libScePosixForWebKit.so) continue ;; esac
        "$sdk/bin/llvm-nm" -D --defined-only "$library" 2>/dev/null | awk '{ print $NF }'
    done | sort -u))
if [[ -n $null_imports ]]; then
    echo "link-title.sh: imports that would be null at run time: ${null_imports//$'\n'/ }" >&2
    exit 1
fi

"$tool" link --in "$work/llvm-pie.elf" --out "$work/eboot.elf" \
    --stub-dir "$sdk/target/lib" --stub "$work/stubs/libSceAgc.so" --stub "$work/stubs/libSceAgcDriver.so" \
    --module-sdk 0x02000009 --companion-sdk 0x08050001 --file-name eboot.elf
param="$here/sce_sys/param.json"
title_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["titleId"])' "$param")
app="$here/dist/$title_id"
mkdir -p "$app/sce_sys" "$app/sce_module"
"$tool" self --sign --in "$work/eboot.elf" --out "$app/eboot.bin" --magic 0x1D3D154F
"$tool" self --inspect --file "$app/eboot.bin" > /dev/null
cp "$param" "$here/sce_sys/icon0.png" "$app/sce_sys/"
(cd "$vulkan/runtime" && sha256sum --check --strict --quiet libc.prx.sha256)
cp "$vulkan/runtime/libc.prx" "$app/sce_module/libc.prx"
cp "$work/llvm-pie.elf" "$out"   # what cargo expects at -o (unsigned, for symbols)
printf '==> %s: %s/eboot.bin (%s bytes)\n' "$title_id" "$app" "$(stat -c %s "$app/eboot.bin")"
