//! Simulator's model

use crate::event::{Event, EventScheduler};
use rand::Rng;

/// can store model as Simulator's model
pub trait Model<Rec> {
    /// usable event's type
    type ModelEvent: Event;

    /// initialize model and schedule when create simulator
    fn initialize<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    );

    /// action for each one step
    fn step<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_events: &mut Vec<Self::ModelEvent>,
    );
}
