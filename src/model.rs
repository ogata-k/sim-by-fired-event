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
    fn start_frame(&mut self, recorder: &mut Rec);

    #[allow(unused_variables)]
    /// schedule event before first event in each frame
    fn before_first_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    ) {
        // usually not use
    }

    /// action when finish frame
    fn finish_frame(&mut self, recorder: &mut Rec);

    #[allow(unused_variables)]
    /// schedule event after last event in each frame
    fn after_last_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Rec,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    ) {
        // usually not use
    }
}

/// can calculate fired events in bulk
pub trait BulkEvents<Rec, E: Event>: Model<Rec, ModelEvent = E> {
    /// action for each one step
    fn step_in_bulk<R: Rng + ?Sized>(
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
        priority: Priority,
        fired_event: Self::ModelEvent,
    );
}
