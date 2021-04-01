//! SimRs's model

use crate::event::{Event, EventScheduler};
use rand::Rng;

/// can store model as SimRs's model
pub trait Model {
    /// usable event's type
    type ModelEvent: Event;

    /// model initializer.
    /// This initializer is `not used` when creating model.
    /// This is `used` by simulator's initializer.
    fn initialize<R: Rng + ?Sized>(&mut self, rng: &mut R);

    /// action after initialize when initialize simulator
    fn at_first_frame<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    );

    /// action for each one step
    fn step<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_events: Vec<Self::ModelEvent>,
    );
}
