//! Two-player co-op. The host owns gameplay decisions; guests supply input.
//!
//! This module is being integrated on the coop branch. Its policy tests are not
//! evidence that the single-player viewer or the campaign already supports co-op.
pub mod check;
pub mod arena;
pub mod audio;
pub mod campaign;
pub mod checkpoint;
pub mod solo;
pub mod solo_check;
mod clock;
pub(crate) mod placement;
pub(crate) mod separation;
mod effects;
mod diagnostics;
pub(crate) mod codec;
pub mod hosting;
pub mod invite;
pub mod menu;
pub mod palette;
pub mod preview;
pub mod protocol;
pub mod rules;
pub mod session;
pub mod transport;
pub mod tunnel;
pub mod world;
pub mod stage;
pub mod village_check;
pub mod pandemonium_check;
pub mod pandemonium_route;
pub mod route_driver;
pub(crate) mod native_route;
pub(crate) mod recovery_trace;
pub mod encounter_check;
mod school2_pickup_check;
mod school_return_check;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum PlayerId {
    Blue,
    Red,
}
impl PlayerId {
    pub const ALL: [Self; 2] = [Self::Blue, Self::Red];

    pub fn index(self) -> usize {
        match self {
            Self::Blue => 0,
            Self::Red => 1,
        }
    }

    pub fn partner(self) -> Self {
        match self {
            Self::Blue => Self::Red,
            Self::Red => Self::Blue,
        }
    }
}
