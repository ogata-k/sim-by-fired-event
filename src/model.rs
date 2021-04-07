//! Simulator's model

use crate::event::{Event, EventScheduler, Priority};
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

    /// action when start frame
    fn start_frame<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    );

    /// action when finish frame
    fn finish_frame<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    );
}

/// can calculate fired events in bulk
pub trait BulkEvents<Rec, E: Event>: Model<Rec, ModelEvent = E> {
    /// action for each one step
    fn step_in_bulk_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_events: Vec<(Priority, Self::ModelEvent)>,
    );
}

/// can calculate fired each event
pub trait StepEachEvent<Rec, E: Event>: Model<Rec, ModelEvent = E> {
    /// action for each one step for one event
    fn step_each_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_event: &(Priority, Self::ModelEvent),
    );
}

/// can calculate fired each event with get None event after all fired events
pub trait LastNoneEvent<Rec, E: Event>: Model<Rec, ModelEvent = E> {
    /// action for each one step for one event with get None event after all fired events
    fn step_optional<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_event: Option<&(Priority, Self::ModelEvent)>,
    );
}
