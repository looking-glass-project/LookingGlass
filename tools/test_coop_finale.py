"""Resume the unchanged earned Queen checkpoint in two public clients."""
import argparse,hashlib,json,shutil,sys,time,traceback
from pathlib import Path
root=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(root/'tools'))
from test_coop_village import Pair

class Finale(Pair):
    map_name='qlair'
    next_map='qlair'
    evidence_prefix='finale-public-'
    capture_scenes=False
    resume_checkpoint=True
    def validate_report(self,role,report):
        for b in report['frame']['players']:
            assert not b['stats']['god'] and not b['stats']['notarget']
    def validate_clear(self,role,report):
        # Normal deaths and the original ending choreography remain possible.
        if report['combat']['stage']['owner'] is None:
            assert all(report['route_driver']['clear']),f'{role}: obstructed body'
    def run(self):
        source=self.args.checkpoint
        raw=source.read_bytes()
        shutil.copyfile(source,self.folder/'host/coop-checkpoint.json')
        self.evidence['checkpoint_sha256']=hashlib.sha256(raw).hexdigest()
        for role in self.commands:
            self.set(role,attack=True,navigate=True,run=True)
            self.commands[role].update(queen_finale=True)
        self.start_pair()
        last=-1
        while True:
            self.pump()
            if self.reports:
                tick=self.reports['host']['frame']['tick']
                if tick//1200!=last:
                    last=tick//1200
                    stage=self.reports['host']['combat']['stage']['world']
                    queen=stage['levels']['qlair']
                    print('FINAL',tick,{k:queen[k] for k in ('phase','queen1','queen2')},flush=True)
            logs={r:p.read_text(errors='replace') for r,p in self.log_paths.items()}
            for r,text in logs.items():
                assert 'Ending film failed:' not in text,r+' movie failed'
            if len(logs)==2 and all('Finale credits: first frame presented' in text for text in logs.values()):break
        for r,text in logs.items():
            assert 'Co-op campaign complete; playing ending and credits' in text
            assert 'Story movie ending: first frame presented' in text
            saved=json.loads((self.folder/'host/coop-checkpoint.json').read_text())
            assert saved['payload']['frame']['combat']['stage']['world']['levels']['qlair']['phase']=='Queen2'
        self.verify_network_states()
        self.verify_network_states('world_sha256','matching_sent_and_restored_world_states')
        self.verify_network_states('native_sha256','matching_sent_and_restored_npc_states')
        self.evidence['presentation']={r:{'movie':True,'credits':True} for r in logs}
        result={'passed':True,'build':self.build,'internet':True,'delay_ms':self.args.delay_ms,
            'jitter_ms':self.args.jitter_ms,'evidence':self.evidence,
            'scope':'unchanged earned Queen2 checkpoint, ordinary public paired combat, ending movie and credits presented in both clients'}
        (self.folder/'result.json').write_text(json.dumps(result,indent=2))
        print('PASS public two-client finale handoff',self.folder,flush=True)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=root/'target/release/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--checkpoint',type=Path,required=True,help='Unchanged earned Queen2 co-op checkpoint')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args=parser.parse_args()
    args.local=False;args.first='blue';args.rejoin=False
    p=Finale(args)
    try:p.run()
    finally:p.close()
