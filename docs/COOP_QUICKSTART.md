# Trying the co-op branch

Build `coop` with `cargo build --release --locked`, then run
`target/release/looking-glass.exe` with your compatible local game data.
Current acceptance and deferred polish are tracked in [playable status](COOP_PLAYABLE.md).

## Host and join

1. Open **Co-op** from the main menu or ESC, then choose **Host**.
2. Choose the game to host and copy the invite link.
3. The other player opens **Co-op > Join**, pastes the link and joins.

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
