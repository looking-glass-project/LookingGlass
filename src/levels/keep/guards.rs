use super::*;
#[derive(Clone, Serialize, Deserialize)]
pub(super) enum Body {
    Basic(Guard),
    Card(crate::cards::Guard),
}
impl Body {
    pub fn identity(&self) -> (&'static str, f32) {
        match self {
            Self::Basic(g) => (if g.ranged { "diamond" } else { "club" }, g.scale),
            Self::Card(g) => (g.kind.model(), g.scale),
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Basic(g) => g.validate_save(),
            Self::Card(g) => g.validate(),
        }
    }
    pub fn health(&self) -> f32 {
        match self {
            Self::Basic(g) => g.health,
            Self::Card(g) => g.health,
        }
    }
    pub fn feet(&self) -> Vec3 {
        match self {
            Self::Basic(g) => g.feet,
            Self::Card(g) => g.feet,
        }
    }
    pub fn hit(&mut self, h: Hit) -> Option<&'static str> {
        match self {
            Self::Basic(g) => g.hit(h),
            Self::Card(g) => g.hit(h),
        }
    }
    pub fn target(&self, id: usize) -> Target {
        match self {
            Self::Basic(g) => g.target(id),
            Self::Card(g) => g.target(id),
        }
    }
}
impl Keep {
    pub(super) fn combat_guards(&mut self, c: &mut crate::level::Combat<'_>) -> crate::combat::Feedback {
        self.combat_guards_with(c.dt, c.world, c.player.eye(), c.notarget, c.summon, None)
    }
    pub(super) fn combat_guards_with(&mut self, dt: f32, world: &World, eye: Vec3,
        notarget: bool, summon: Option<Target>, players: Option<[crate::combat::PlayerTarget; 2]>)
        -> crate::combat::Feedback {
        let mut out = crate::combat::Feedback::default();
        if self.scripted() || !dt.is_finite() || dt <= 0. {
            return out;
        }
        for a in &mut self.saved.spawned {
            a.electric = (a.electric - dt).max(0.);
            let mut f = crate::combat::Feedback::default();
            match &mut a.guard {
                Body::Basic(g) => {
                    g.notarget = notarget;
                    g.opponents.summon = summon;
                    g.opponents.players = players;
                    f = g.advance(
                        dt,
                        world,
                        eye,
                        if g.ranged {
                            self.data.diamond
                        } else {
                            self.data.guard
                        },
                    );
                }
                Body::Card(g) => {
                    g.notarget = notarget;
                    g.opponents.summon = summon;
                    g.opponents.players = players;
                    a.accumulator += dt.min(0.1) as f64;
                    while a.accumulator >= crate::cards::STEP as f64 {
                        a.accumulator -= crate::cards::STEP as f64;
                        self.data.cards[usize::from(g.kind == crate::cards::Kind::Spade)].step(
                            g,
                            world,
                            eye,
                            &mut f,
                        );
                    }
                }
            }
            f.spatial_sounds.extend(f.sounds.drain(..).map(|sound| (sound, a.guard.feet())));
            out.append(f);
        }
        out
    }
}
