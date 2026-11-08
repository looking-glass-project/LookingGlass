# Co-op fixes and latency update

**v0.33.0-preview.4**, 8 November 2026, is the current
[experimental prerelease](https://github.com/looking-glass-project/LookingGlass/releases/tag/v0.33.0-preview.4).
The repository, installation guide and website download links select this build.
Both co-op players need the same version.

It fixes setup persistence, adds 2560 x 1600, restores each player's aim glow,
and provides difficulty selection, Tab and supported console commands in co-op.
Snapshot deltas, bounded flow control and stable update schedules reduce
avoidable network delay. It passed 985 Rust tests (six ignored), Windows CI,
native public-invite menu, save/rejoin, particle and stall-recovery checks.
Remote guest stutter and post-cutscene lag remain unverified on the affected
laptop; local performance results do not establish that those reports are fixed.
See [the evidence and remaining limits](COOP_PLAYTEST_FIXES.md).

The release tag and tested executable correspond to source commit
`fe4f38171bb5f465cd4fda1145d0f6e963502289`, now merged into `main`.
The package, source manifest and checksums are unchanged by the merge.

## First co-op preview (historical record)

**v0.33.0-preview.1**, 8 November 2026, brings two-player internet co-op to the
regular **Launch.cmd** and main/ESC menus. Host shares an invite link; Join uses
it. Blue and red Alice have separate cameras, health and consumables.

Existing solo saves can become co-op sessions. Shared checkpoints support
resuming and rejoining; a fallen player spectates their partner for ten seconds
before reviving nearby, and a shared death restores the last checkpoint.
Normal paired progression has been accepted across all 39 campaign visits,
including leaf rides, swimming and the ending. See [playable evidence and
remaining polish](COOP_PLAYABLE.md) and [core save/rejoin checks](COOP_CORE_TESTS.md).

The release incorporates the current main branch's localization and explicit
download opt-in setup, shares its ordinary-save reader with co-op import, and
prevents the source launcher from selecting an obsolete private playtest build.
The Windows package includes a static C runtime, dependency notices, source
manifest and checksums. It remains unsigned and experimental; Latest makes the
download discoverable and is not a stability certification.

## Walking cadence update (historical record)

**v0.32.0-preview.2**, 9 October 2026, follows the first Windows preview below.
[Download and release notes](https://github.com/skulitom/LookingGlass/releases/tag/v0.32.0-preview.2).

- Merges [PR #3](https://github.com/skulitom/LookingGlass/pull/3), addressing
  [issue #2](https://github.com/skulitom/LookingGlass/issues/2).
- Walking targets 104 units/s for a brisk pace, with animation still driven by
  actual movement. Gentle stick input animates below the earlier threshold.
  Running stays at 320 units/s; walking jumps cover less horizontal distance.
- Save format 12 and original asset requirements are unchanged.
- The Windows package is built from the tagged public source with a statically
  linked C runtime. BUILD_INFO.json, MANIFEST.json and the attached source manifest
  identify its commit, compiler, build inputs and hashes.
- Validation covers unit tests, original animation data, native render/save
  checks, the school physics check, village traversal and extracted-package
  launch. This update has no new complete campaign replay, Steam Deck/Proton,
  clean-account installation or audible-playback certification.
- The release remains unsigned and experimental. As with preview.1, it is listed
  as GitHub's Latest for download discoverability, not as a claim of stability.

## First public preview (historical record)

**v0.32.0-preview.1**, 1 October 2026, is the first experimental Windows x64 preview.
[Repository](https://github.com/skulitom/LookingGlass) Â· [Release and download](https://github.com/skulitom/LookingGlass/releases/tag/v0.32.0-preview.1)

## Publication boundary

The public repository begins with a reviewed snapshot of current source. The older
local development history, private research, saves, game packs and raw captures
remain private. The six hash-checked README media files are the requested publicity
exception; the underlying game artwork is excluded from the MIT source license.

The Windows ZIP includes the new engine, launch/setup helpers, player instructions,
component notices, an extractor and required third-party source packages. It does
not contain the original game executable, DLLs or game packs. Setup fetches game
data separately only when the player selects Download.

## Build identity

All 479 engine build inputs (Rust source, compiled route JSON, Cargo manifests and
build.rs) match the previously tested static-runtime candidate, allowing only Git
line-ending normalization (152 files change from CRLF to LF). No code or data
content changes were made to the engine.
The public commit adds the current installer, player documentation, source audit
and curated gameplay previews. BUILD_INFO.json identifies both the original build
snapshot and the matching public source commit; it does not claim a rebuild from
that commit. MANIFEST.json hashes every other packaged file. The release also
provides a ZIP checksum and an engine-input manifest.

The executable uses x86_64-pc-windows-msvc and a statically linked C runtime. It is
unsigned. The release notes identify the compiler and verification evidence.

## Verification and remaining limits

- 665 Rust unit tests passed, one ignored, on the matching engine source.
- Installer tests cover synthetic inputs, invalid hashes, cancellation and retained
  configuration. Package tests check the source boundary and archive contents.
- The exact candidate passed campaign graph, registry and items checks. Its campaign
  legs run exceeded the earlier 180-second harness limit; that run is incomplete.
- Guided first launch, actual compatible-data download/import and second launch
  were checked on the development machine. A clean Windows account and audible
  playback have not been verified.
- Five earlier automated campaign lineages completed all 39 visits, with one fresh
  uninterrupted run. See [the campaign audit](CAMPAIGN_RUN_AUDIT.md). This does not
  certify every cinematic, sound, original behavior or hardware configuration.
- Windows CI runs source-boundary, synthetic setup/package and Rust unit checks.
  It does not download original game data or publish releases automatically.

[Known issues](KNOWN_ISSUES.md) accompany this preview. At the owner's request,
the Windows preview is listed as GitHub's **Latest** release so it appears in the
repository's Releases sidebar and at `/releases/latest`. GitHub requires clearing
its prerelease flag for this placement. The tag, title and player documentation
continue to identify it as an **experimental preview**; broader player/device and
fidelity validation are still needed before describing it as stable. This listing
change does not replace the tested Windows package or change its checksum.
Steam publicity is prepared separately as a draft for the owner to review.

## Future release checks

Freeze the intended source, preserve versioned save migrations, verify the build
against its commit, run relevant gameplay checks and repeat ZIP-only installation
on a clean account. Review dependency notices and exact public files. Record any
uncompleted checks honestly in release notes. Never upload private test evidence,
original assets or old development history as an incidental release artifact.
