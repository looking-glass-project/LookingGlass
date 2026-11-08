# Playing co-op

Extract the Windows release and open the regular **Launch.cmd**. Co-op is part
of the main game; choose **Co-op** from the main menu or ESC. Both players need
this release and their own compatible game files.

When building from source, use `cargo build --release --locked`, then open
**Launch.cmd**. The source launcher uses the current release build.
Current acceptance and deferred polish are tracked in [playable status](COOP_PLAYABLE.md).

## Host and join

1. Open **Co-op** from the main menu or ESC, then choose **Host**.
2. Choose the game to host and copy the invite link.
3. The other player opens **Co-op > Join**, pastes the link and joins.

**Start chapter** opens the chapter and difficulty chooser. Click the difficulty
or press D to cycle Easy, Normal, Hard and Nightmare before starting. Hosting a
loaded game or continuing a checkpoint keeps that game's saved difficulty.

During play, **Tab** opens the chapter browser on either client; the host can
start a fresh shared visit with the selected difficulty. The **` / ~** console
is available to both players. Type `help` for supported co-op commands; the host
controls shared map changes, restart and checkpoint saving. These overlays do
not pause the other player.

**F11** toggles a frame-rate/active-GPU overlay and a small `private/coop-performance.json`
report (inside the selected profile when using `--profile`). It records every
frame in each reporting interval, including stalls. Detailed world reports are
disabled in normal play; automated tests can opt in with
`LOOKING_GLASS_COOP_DIAGNOSTICS=1`. Both report writers avoid disk waits on the
render thread. Diagnostics contain no invite token.

The invite works over the internet without configuring router ports. The host
owns the shared campaign save; both players keep their own health and resources.
To resume later, the host chooses **Continue co-op** and shares the new invite.

To add a second player to an earlier solo game, load that save first, then open
**ESC > Co-op > Host > Host this game**. Co-op uses separate checkpoint files;
the original single-player slot is preserved.

## Two clients on one PC

From the repository, run:

```powershell
.\tools\run_coop_pair.ps1 -Data 'C:\Games\Alice\base' -Menu
```

Replace the data path with your compatible installation. The launcher opens
two independent profiles and main menus using the release build. Use Host and
Join as above, then focus either window to control its Alice. Profiles and
logs are retained in the printed `private/coop/manual-...` folder.
Supply `-Exe` to select another built executable.
To resume one of these test sessions later, launch each executable with
`--profile` pointing to its retained `host` or `guest` directory. The pair
launcher creates fresh profiles on each run.

For an automatic connection at the first campaign visit, omit `-Menu`. Add
`-Local` only when deliberately testing a local connection instead of a public
invite. The optional staged combat fixture uses `-Map skool1 -Combat`.

## Repeating the final handoff check

`tools/test_coop_finale.py` accepts `--exe`, `--data` and an unchanged, normally
earned Queen2 co-op `--checkpoint`. It restores that save, joins two public
clients with 40 ms delay and 20 ms jitter, and drives ordinary combat inputs.
It compares sent/restored state and requires each client's ending movie and
credits to render. Focus each film window, press P if paused, and hold Enter
to skip to credits. The runner closes its own clients after both pass.

Original game data and personal checkpoints remain local and are not included.
