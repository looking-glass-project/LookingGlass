use super::{Batch, Cue, Receiver};
use crate::{assets::Assets, audio::Audio, bsp::Bsp, collision::World, coop::{arena::Arena, world::Frame, PlayerId}, Options};
use anyhow::Result;
use macroquad::prelude::Vec3;
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Report {
    pub accepted: u64,
    pub recent: Vec<Cue>,
    pub voice_starts: u64,
    pub voice_path: Option<String>,
    pub voice_elapsed: f32,
    pub output_frames: u64,
    pub peak: f32,
    pub resolved: u64,
    pub missing: usize,
    pub listener_player: Option<PlayerId>,
    pub listener_health: f32,
    pub listener_swimming: bool,
    pub listener_submerged: bool,
}
#[derive(Default)]
struct Voice {
    key: Option<(String,usize,String)>,
    elapsed: f32,
}
impl Voice {
    fn changed(&mut self, now: Option<(&str,usize,&str,f32)>) -> bool {
        let key = now.map(|(id,index,path,_)|(id.to_owned(),index,path.to_owned()));
        let elapsed = now.map_or(0.,|v|v.3);
        let changed = key != self.key || elapsed + 0.25 < self.elapsed;
        self.key = key; self.elapsed = elapsed;
        changed
    }
}
pub struct Playback {
    audio: Audio,
    receiver: Receiver,
    voice: Voice,
    scope: Option<(u64,u64)>,
    capture_started: bool,
    pub report: Report,
}
impl Playback {
    pub fn new(options: &Options) -> Self {
        Self { audio:Audio::new(options.no_audio,options.audio_capture.clone()),
            receiver:Receiver::default(), voice:Voice::default(), scope:None, capture_started:false, report:Report::default() }
    }
    pub fn load(&mut self, assets:&mut Assets, name:&str, map:&Bsp) {
        self.audio.load(assets,name,map);
        self.receiver = Receiver::default(); self.voice = Voice::default(); self.scope = None;
        self.report.listener_player = None;
    }
    fn follow_player<'a>(&mut self, frame:&'a Frame, subject:PlayerId) -> &'a crate::coop::world::Body {
        if self.report.listener_player != Some(subject) { self.audio.reset_listener_player(); }
        let body = &frame.players[subject.index()];
        self.report.listener_player = Some(subject);
        self.report.listener_health = body.stats.sanity();
        self.report.listener_swimming = body.player.swimming;
        self.report.listener_submerged = body.player.immersion.level == 3;
        body
    }
    pub fn update(&mut self, assets:&mut Assets, arena:&Arena, frame:&Frame, world:&World,
        subject:PlayerId, eye:Vec3, yaw:f32, dt:f32, playing:bool) -> Result<()> {
        if playing && !self.capture_started { self.audio.begin_capture(); self.capture_started = true; }
        let scope = (frame.epoch,frame.checkpoint_generation);
        if self.scope != Some(scope) {
            self.audio.restore_world(); self.voice = Voice::default(); self.scope = Some(scope);
        }
        let body = self.follow_player(frame,subject);
        let empty = Batch::default();
        for cue in self.receiver.observe(scope,frame.tick,frame.sounds.as_ref().unwrap_or(&empty))? {
            if playing {
                if cue.animation {
                    self.audio.animation_at(crate::audio::events::Cue {path:cue.path.clone(),volume:cue.volume},cue.origin);
                } else { self.audio.effect(&cue.path,cue.origin,cue.volume); }
                self.report.accepted += 1;
                self.report.recent.push(cue);
                if self.report.recent.len() > 128 { self.report.recent.remove(0); }
            }
        }
        let current = playing.then(||arena.stage.story.voice_position()).flatten();
        if self.voice.changed(current) {
            if let Some((_,_,path,elapsed)) = current {
                self.audio.speak_from(assets,path,elapsed);
                self.report.voice_starts += 1;
            } else { self.audio.stop_voice(); }
        }
        self.report.voice_path = current.map(|v|v.2.to_owned());
        self.report.voice_elapsed = current.map_or(0.,|v|v.3);
        let loops = arena.characters.iter().enumerate().flat_map(|(index,c)|
            c.audio_loops().into_iter().filter_map(move |mut cue| {
                // Disjoint from level loops and the other Alice's projectile IDs.
                cue.id = cue.id.checked_mul(2)?.checked_add(index)? | (1usize << 63);
                Some(cue)
            })).collect();
        self.audio.world_replica(assets,&arena.stage.interactions,loops);
        self.audio.player_state(assets,body.stats.sanity(),body.player.swimming,
            body.player.immersion.level == 3, body.player.immersion.level == 3 && body.player.breath.remaining() <= 0.);
        self.audio.prepare(assets,world,eye,body.player.immersion.level == 3,dt);
        let world_dt = crate::coop::world::world_dt(&frame.players, dt);
        self.audio.update_clocks(dt,world_dt,eye,yaw,!playing,!playing || !arena.stage.voice_running);
        (self.report.output_frames,self.report.peak,self.report.resolved,self.report.missing) = self.audio.diagnostics();
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> { self.audio.finish() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn listener_follows_either_survivor_and_returns_to_the_revived_player() {
        use crate::{coop::world::{camera_player,Roster}, inventory::Stats};
        use macroquad::prelude::vec3;
        let world=World::fixture(&[(vec3(-1000.,-1000.,-100.),vec3(1000.,1000.,0.))]);
        for local in PlayerId::ALL {
            for survivor_underwater in [false,true] {
                let roster=Roster::new(&world,Vec3::Z*64.,0.,Stats::default()).unwrap();
                let mut frame=roster.frame(1,0);
                let mut playback=Playback {audio:Audio::new(true,None),receiver:Receiver::default(),
                    voice:Voice::default(),scope:None,capture_started:false,report:Report::default()};
                frame.players[local.index()].player.immersion.level=if survivor_underwater {0} else {3};
                frame.players[local.partner().index()].player.immersion.level=if survivor_underwater {3} else {0};
                frame.players[local.partner().index()].player.swimming=survivor_underwater;
                playback.follow_player(&frame,camera_player(&frame,local));
                assert_eq!(playback.report.listener_player,Some(local));
                frame.players[local.index()].stats.damage(1000.);
                playback.follow_player(&frame,camera_player(&frame,local));
                assert_eq!(playback.report.listener_player,Some(local.partner()));
                assert_eq!(playback.report.listener_submerged,survivor_underwater);
                assert_eq!(playback.report.listener_swimming,survivor_underwater);
                assert_eq!(playback.report.listener_health,100.);
                frame.players[local.index()].stats.set_health(25.).unwrap();
                playback.follow_player(&frame,camera_player(&frame,local));
                assert_eq!(playback.report.listener_player,Some(local));
                assert_eq!(playback.report.listener_submerged,!survivor_underwater);
                assert_eq!(playback.report.listener_health,25.);
            }
        }
    }
    #[test]
    fn voices_follow_lines_and_seek_only_on_join_skip_or_rewind() {
        let mut voice = Voice::default();
        assert!(!voice.changed(None));
        assert!(voice.changed(Some(("scene",0,"sound/line.wav",1.2))));
        for elapsed in [1.2,1.2,1.25,1.3] { assert!(!voice.changed(Some(("scene",0,"sound/line.wav",elapsed)))); }
        assert!(voice.changed(Some(("scene",1,"sound/line.wav",0.))));
        assert!(!voice.changed(Some(("scene",1,"sound/line.wav",3.))));
        assert!(voice.changed(Some(("scene",1,"sound/line.wav",0.5))));
        assert!(voice.changed(None)); assert!(!voice.changed(None));
    }
}
