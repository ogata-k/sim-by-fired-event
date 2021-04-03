//! SimRs's model

use crate::event::{Event, EventScheduler};
use rand::Rng;

/// can store model as SimRs's model
pub trait Model<Rec> {
    /// usable event's type
    type ModelEvent: Event;

    /// model initializer.
    /// This initializer is `not used` when creating model.
    /// This is `used` by simulator's initializer.
    fn initialize<R: Rng + ?Sized>(&mut self, rng: &mut R, recorder: &mut Rec);

    /// action after initialize when initialize simulator
    fn initialize_frame<R: Rng + ?Sized>(
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
