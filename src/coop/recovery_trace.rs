//! Optional exact host-tick evidence. Never changes game state or network frames.
use super::{rules::Life,world::Roster,PlayerId};
use crate::collision::World;
use anyhow::{ensure,Result};
use macroquad::prelude::*;
use std::{fs::File,io::{BufWriter,Write},path::PathBuf};

struct Previous {
    epoch:u64,tick:u64,generation:u64,spawned:[u64;2],life:[Life;2],
}
pub struct Trace {
    file:BufWriter<File>,previous:Option<Previous>,serial:u64,
}
fn kind(life:Life)->u8 {match life {Life::Alive=>0,Life::Spectating{..}=>1,Life::AwaitingSafeSpawn=>2}}
impl Trace {
    pub fn new(path:PathBuf)->Result<Self> {
        // A new isolated fixture owns this path. Refuse to erase older evidence.
        let file=File::options().write(true).create_new(true).open(path)?;
        Ok(Self{file:BufWriter::new(file),previous:None,serial:0})
    }
    pub fn record(&mut self,roster:&Roster,world:&World,epoch:u64,map:&str,cinematic:bool,
        spawn_held:bool,safe:impl Fn(Vec3)->bool)->Result<()> {
        if let Some(p)=&self.previous {
            if (p.epoch,p.tick)==(epoch,roster.tick) {return Ok(());}
            ensure!(epoch>p.epoch || (epoch==p.epoch && roster.tick==p.tick+1),"Recovery trace missed a host tick");
        }
        let life=PlayerId::ALL.map(|id|roster.lives.state(id));
        let spawned=roster.players.each_ref().map(|p|p.spawned_at);
        let full=self.previous.as_ref().is_none_or(|p|p.epoch!=epoch || p.generation!=roster.checkpoint_generation
            || p.spawned!=spawned || p.life.map(kind)!=life.map(kind));
        let dead=life.iter().any(|l|*l!=Life::Alive);
        let was_dead=self.previous.as_ref().is_some_and(|p|p.life.iter().any(|l|*l!=Life::Alive));
        if full || dead || was_dead || roster.tick%120==0 {
            self.serial+=1;
            let players=roster.players.iter().enumerate().map(|(i,b)|serde_json::json!({
                "life":life[i],"health":b.stats.sanity(),"feet":b.player.feet,"spawned_at":b.spawned_at,
                "warped_at":b.warped_at,"god":b.stats.god,"notarget":b.stats.notarget,
                "stats":full.then_some(&b.stats),
                "geometry":full.then(||serde_json::json!({"clear":world.body_clear(b.player.feet),
                    "safe":safe(b.player.feet),"valid":b.player.validate_world(world).is_ok(),
                    "supported":super::world::revival_supported(world,&b.player),
                    "swimming":b.player.immersion.level>=2,"rope":b.player.rope.is_some()}))
            })).collect::<Vec<_>>();
            serde_json::to_writer(&mut self.file,&serde_json::json!({"version":1,"serial":self.serial,
                "build":env!("LOOKING_GLASS_BUILD"),"map":map,"epoch":epoch,"tick":roster.tick,
                "checkpoint_generation":roster.checkpoint_generation,"cinematic":cinematic,
                "spawn_held":spawn_held,"players":players}))?;
            self.file.write_all(b"\n")?;
            if full || roster.tick%24==0 {self.file.flush()?;}
        }
        self.previous=Some(Previous{epoch,tick:roster.tick,generation:roster.checkpoint_generation,spawned,life});
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Stats,movement::EYE_HEIGHT};
    #[test]
    fn records_exact_revive_between_report_frames_without_changing_the_roster() {
        let path=std::env::temp_dir().join(format!("looking-glass-recovery-{}-{}.jsonl",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let world=World::fixture(&[(vec3(-1000.,-1000.,-100.),vec3(1000.,1000.,0.))]);
        let mut roster=Roster::new(&world,Vec3::Z*(EYE_HEIGHT+0.04),0.,Stats::default()).unwrap();
        let mut trace=Trace::new(path.clone()).unwrap();
        assert!(Trace::new(path.clone()).is_err(),"Recorder overwrote existing evidence");
        trace.record(&roster,&world,1,"fixture",false,false,|_|true).unwrap();
        roster.players[0].stats.damage(1000.);
        for _ in 0..1205 {
            roster.step(&world,Default::default()).unwrap();
            let before=serde_json::to_value(roster.frame(1,0)).unwrap();
            trace.record(&roster,&world,1,"fixture",false,false,|_|true).unwrap();
            trace.record(&roster,&world,1,"fixture",false,false,|_|true).unwrap();
            assert_eq!(serde_json::to_value(roster.frame(1,0)).unwrap(),before);
        }
        drop(trace);
        let data=std::fs::read_to_string(&path).unwrap();std::fs::remove_file(path).unwrap();
        let rows:Vec<serde_json::Value>=data.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
        assert_eq!(rows.len(),1202);
        assert_eq!(rows[1]["players"][0]["life"]["Spectating"]["ticks_left"],1200);
        let last=rows.last().unwrap();assert_eq!(last["tick"],1201);
        assert_eq!(last["players"][0]["spawned_at"],1201);assert_eq!(last["players"][0]["health"],25.);
        assert_eq!(last["players"][0]["geometry"]["clear"],true);
        assert_eq!(last["players"][0]["geometry"]["supported"],true);
        for pair in rows.windows(2) {assert_eq!(pair[1]["serial"].as_u64().unwrap(),pair[0]["serial"].as_u64().unwrap()+1);}
    }
    #[test]
    fn recovery_support_requires_ground_contact_and_a_full_footprint() {
        let world=World::fixture(&[(vec3(-100.,-100.,-20.),vec3(100.,100.,0.))]);
        let mut player=crate::movement::Player::spawn(&world,Vec3::Z*(EYE_HEIGHT+0.03125)).unwrap();
        assert!(super::super::world::revival_supported(&world,&player));
        player.feet.z+=8.;
        assert!(!super::super::world::revival_supported(&world,&player));
        player.feet.z-=8.;player.grounded=false;
        assert!(!super::super::world::revival_supported(&world,&player));
        player.grounded=true;player.feet.x=90.;
        assert!(!super::super::world::revival_supported(&world,&player));
    }
}
