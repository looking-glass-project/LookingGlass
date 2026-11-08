"""Check audio listener selection through real deaths, revivals and guest rejoins.

Runs the ordinary underwater recovery fixture in two independent clients.
Checks the production audio inputs, not speaker output or acoustic fidelity.
No audio endpoint, screenshots, edited bodies or artificial deaths are needed.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_play import ROOT
from test_coop_utemple_countdown import CountdownTemple


PLAYERS=('Blue','Red')
LISTENER_FIELDS=('listener_player','listener_health','listener_swimming','listener_submerged')


def verify_listener(row):
    assert row['role'] in ('host','guest') and row['camera_player'] in PLAYERS, 'Missing listener identity'
    audio=row['audio'];subject=PLAYERS.index(row['camera_player']);body=row['players'][subject]
    assert audio['listener_player']==row['camera_player'], 'Audio follows a different player from the camera'
    assert audio['listener_health']==body['health'], 'Audio uses another player\'s health'
    assert audio['listener_swimming']==body['swimming'], 'Audio uses another player\'s swimming ambience'
    assert audio['listener_submerged']==body['submerged'], 'Audio uses another player\'s underwater filter'


def verify_listener_history(history):
    assert set(history)=={'host','guest'}, 'Missing independent listener client'
    evidence={}
    for role,rows in history.items():
        local=('host','guest').index(role)
        counts=dict(normal=0,spectating=0,revived=0,different_immersion=0)
        spectated_at=None;scope=None
        for row in rows:
            assert row['role']==role, 'Listener row belongs to another client'
            verify_listener(row)
            current_scope=(row['epoch'],row['generation'])
            if scope!=current_scope:
                spectated_at=None;scope=current_scope
            own=row['players'][local];subject=PLAYERS.index(row['camera_player'])
            if own['health']>0 and subject==local:
                counts['normal']+=1
                counts['revived']+=int(spectated_at is not None and own['spawned_at']>spectated_at)
            if own['health']<=0 and row['players'][1-local]['health']>0:
                assert subject==1-local, 'Dead player is not listening with the survivor'
                counts['spectating']+=1
                spectated_at=row['tick']
                counts['different_immersion']+=int(own['submerged']!=row['players'][subject]['submerged'])
        assert all(counts[k]>0 for k in ('normal','spectating','revived')), 'Missing live, spectator or revived listener coverage'
        evidence[role]=dict(rows=len(rows),processes=len({r['pid'] for r in rows}),**counts)
    return evidence


class ListenerPair(CountdownTemple):
    evidence_prefix='audio-listener-'

    def __init__(self,args):
        self.listener_rows={role:[] for role in ('host','guest')}
        self.listener_seen=set()
        super().__init__(args)
        source=Path(__file__).read_bytes()
        (self.folder/Path(__file__).name).write_bytes(source)
        self.evidence['sources'][Path(__file__).name]=hashlib.sha256(source).hexdigest()

    def pump(self):
        super().pump()
        for role,report in self.reports.items():
            assert report['build']==self.args.build, 'Wrong audio listener runtime build'
            frame=report['frame'];key=(role,report['pid'],frame['epoch'],frame['checkpoint_generation'],frame['tick'])
            if key in self.listener_seen:continue
            row=dict(role=role,pid=report['pid'],epoch=frame['epoch'],tick=frame['tick'],
                     generation=frame['checkpoint_generation'],camera_player=report['camera_player'],
                     audio={k:report['audio'][k] for k in LISTENER_FIELDS},
                     players=[dict(health=b['stats']['sanity'],swimming=b['player']['swimming'],
                                   submerged=b['player']['immersion']['level']==3,spawned_at=b['spawned_at'])
                              for b in frame['players']])
            self.listener_rows[role].append(row)
            self.listener_seen.add(key)
            verify_listener(row)

    def run(self):
        super().run()
        # Keep the recovery result distinct if this additional verification fails.
        result_path=self.folder/'result.json'
        result_path.replace(self.folder/'recovery-result.json')
        proof=verify_listener_history(self.listener_rows)
        result=json.loads((self.folder/'recovery-result.json').read_text())
        result.update(scope='Both real underwater spectator/revival listener changes and fresh guest rejoins; production mixer inputs only',
                      speaker_output_verified=False,full_route_passed=False)
        result['evidence']['audio_listeners']=proof
        result_path.write_text(json.dumps(result,indent=2))
        print('PASS audio listener follows both survivor cameras and returns on revival:',proof,flush=True)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,required=True)
    parser.add_argument('--build',required=True)
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args=parser.parse_args();args.first='blue'
    if not all(0<=n<=500 for n in (args.delay_ms,args.jitter_ms)):parser.error('Delay/jitter must be 0..500 ms')
    pair=ListenerPair(args)
    print('Evidence:',pair.folder,flush=True)
    try:pair.run()
    except BaseException as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error),build=pair.build),indent=2))
        raise
    finally:
        (pair.folder/'audio-listeners.json').write_text(json.dumps(pair.listener_rows))
        pair.close()


if __name__=='__main__':main()
