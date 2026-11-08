"""Traverse a fresh Funhouse in two real clients, fight normally and enter Hatter1."""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_funhouse_boss import BossPair, SCENE
from test_coop_play import ROOT


class ArenaEnding(Exception):
    """Actual victory interrupted an optional combat movement, not an arrival."""


class FullPair(BossPair):
    evidence_prefix = 'funhouse-full-'

    def __init__(self, args):
        self.arena_movement = False
        super().__init__(args)
        assert self.saved is None and self.args.start_step == 0
        self.deadline = time.monotonic() + 1700
        raw = Path(__file__).read_bytes()
        (self.folder / 'fixture-dependencies' / Path(__file__).name).write_bytes(raw)
        self.evidence['fixture_sha256'][Path(__file__).name] = hashlib.sha256(raw).hexdigest()

    def pump(self):
        super().pump()
        if self.arena_movement and any(r['route_driver']['scene'] == SCENE for r in self.reports.values()):
            raise ArenaEnding()

    def run(self):
        self.start_pair()
        assert not (self.folder / 'host/route-resume.json').exists(), 'Fresh run restored a checkpoint'
        self.scene_end()
        self.play_steps(0, 138)
        # All corridor, switch, machinery and pendulum arrivals are required for
        # both players before crediting the route. Arena healing circuits after
        # this point are optional combat tactics; actual victory can end them.
        self.verify(last_step=138)
        assert all(self.state(r)['gas'] and self.state(r)['cells'] == 255 for r in self.commands), 'Incomplete gas/cell progression'
        self.evidence['verified_pre_arena_steps'] = list(self.completed)
        self.evidence['pre_arena_end_ticks'] = {r: report['frame']['tick'] for r, report in self.reports.items()}
        self.retain('verified-pre-arena')
        self.arena_movement = True
        try:
            self.play_steps(139, 155)
        except ArenaEnding:
            self.evidence['interrupted_arena_step'] = self.step
            for role in self.commands:
                self.set(role, attack=False)
        finally:
            self.arena_movement = False
        if self.fighting():
            self.fight()
        else:
            self.until(lambda: all(r['route_driver']['scene'] == SCENE for r in self.reports.values()), 20)
        self.evidence['completed_steps'] = list(self.completed)
        self.finish_ending(full_route=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--skip-scenes', action='store_true')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.start_step, args.last_step, args.checkpoint = 0, 155, None
    args.save_before = [29, 35, 43, 89, 95, 103, 111, 121, 126, 135, 139, 144, 147, 151, 156]
    args.allow_revival, args.require_revival = True, False
    pair = FullPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, step=pair.step, completed=pair.completed)))
        raise
    finally:
        pair.close()
