# ps5-std-rt (vendored)

From [neokuze/ps5-rust](https://github.com/neokuze/ps5-rust) at `19a6ac0`, `crates/ps5-std-rt`,
GPL-3.0-or-later (see `LICENSE`). It makes Rust's FreeBSD std link and run on the PS5.

Change for the Looking Glass PS5 title: with `PS5_STD_RT_TITLE=1` (set by `ps5/build.sh`) the build
script no longer bundles its patched copy of the payload SDK's static `libc.a`. A title takes libc
from the console's modules (`libc.prx`, `libSceLibcInternal`) and the payload SDK fork's platform
layer, as PS5_VulkanTemplate's titles do; a second, static libc clashes with RADV's C11 threads.
Its Rust shims stay; names that also exist on the platform side are renamed in the Rust objects by
`ps5/link-title.sh`.
