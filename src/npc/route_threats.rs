//! Input-driver choices only; these never change an NPC's AI, damage or vulnerability.
use super::*;

impl Npcs {
    /// Read-only local test inventory; keep resident enemies visible to diagnostics.
    pub fn route_boojums(&self) -> impl Iterator<Item=(usize,&crate::boojum::Boojum,bool)> {
        self.actors.iter().enumerate().filter_map(|(id, actor)| {
            let resident=actor.resident.as_ref()?;
            let resident::Body::Boojum(boojum)=&resident.body else {return None;};
            Some((id,boojum,resident.active && resident.awake && boojum.active))
        })
    }

    pub fn route_engaged(&self, id: usize, eye: Vec3) -> bool {
        let Some(actor) = self.actors.get(id) else {
            return false;
        };
        if let Some(resident) = &actor.resident {
            if let resident::Body::Snark(snark) = &resident.body {
                if snark.kind == crate::snark::Kind::BiteOnly {
                    // A player on a leaf need not spend Cards on distant submerged
                    // biters. Respond before bite range when one can reach Alice.
                    return snark.phase != crate::snark::Phase::Idle
                        && snark.target(id).center.distance(eye) < 180. * snark.scale;
                }
            }
        }
        true
    }
}
