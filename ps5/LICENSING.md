# Licensing of the PS5 port

| Part | Licence |
| --- | --- |
| Looking Glass engine and the changes in this repository | MIT (`LICENSE-MIT`) |
| `vendor/miniquad` (incl. the new Vulkan backend and PS5 platform), `vendor/macroquad`, `vendor/cpal` | MIT OR Apache-2.0, as upstream |
| `vendor/miniquad/src/native/ps5c/platform.{c,h}` (PS5_VulkanTemplate, Mihawk) | MIT |
| `vendor/ps5-std-rt` (neokuze/ps5-rust) and `ps5/targets/x86_64-sie-ps5-std.json` | GPL-3.0-or-later |
| `ps5/link-title.sh` (follows PS5_VulkanTemplate's MIT `link-title.sh`) | MIT |
| `ps5/sce_sys/icon0.png` (PS5_VulkanTemplate placeholder icon) | as in PS5_VulkanTemplate |

A built title (`eboot.bin`) combines these with GPL-3.0 parts of the build stack (ps5-std-rt, the PS5
payload SDK's startup code, PS5_Vulkan's RADV archive and tools). **Anyone distributing a build must
offer its complete corresponding source under the GPL-3.0**, including the exact revisions of
PS5_Vulkan, PS5_Mesa and PS5_PayloadSDK it was built from.

Never commit or distribute game data (pk3 files, extracted assets) or build output.
