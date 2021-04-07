//! Simulator is discrete time simulator with event which fire at scheduled timing.

use crate::event::{Event, EventScheduler, Priority};
use crate::model::{BulkEvents, LastNoneEvent, Model, StepEachEvent};
use rand::Rng;
use std::mem;

pub mod event;
pub mod model;

/// TimeCounter for user
pub trait FrameCounter: Copy {
    /// start state. this value used always.
    fn start_index() -> Self;

    /// get next index
    fn next_index(&mut self);

    /// check can continue
    fn can_continue(&self, specified: &Self) -> bool;
}

macro_rules! impl_counter {
    ($t:ty, $i:ident) => {
        impl FrameCounter for $t {
            fn start_index() -> $t {
                $i::MIN
            }

            fn next_index(&mut self) {
                *self += 1;
            }

            fn can_continue(&self, specified: &$t) -> bool {
                self <= specified
            }
        }
    };
}
impl_counter!(u8, u8);
impl_counter!(u16, u16);
impl_counter!(u32, u32);
impl_counter!(u64, u64);
impl_counter!(u128, u128);
impl_counter!(usize, usize);

/// simulator
#[derive(Debug, Clone)]
pub struct Simulator<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E>,
    E: Event,
{
    model: M,
    recorder: Rec,
    scheduler: EventScheduler<E>,
}

impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E>,
    E: Event,
{
    /// create as default
    pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Self
    where
        Rec: Default,
        M: Default,
    {
        let mut sim = Self {
            model: Default::default(),
            recorder: Default::default(),
            scheduler: EventScheduler::new(),
        };
        sim.initialize(rng);
        sim
    }

    /// create simulator from model
    pub fn create_from<R: Rng + ?Sized>(rng: &mut R, model: M, recorder: Rec) -> Self {
        let mut sim = Self {
            model,
            recorder,
            scheduler: EventScheduler::new(),
        };
        sim.initialize(rng);
        sim
    }

    /// initialize simulator
    fn initialize<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.model
            .initialize(rng, &mut self.recorder, &mut self.scheduler);
    }

    /// getter for model
    pub fn get_model(&self) -> &M {
        &self.model
    }

    /// getter for scheduler
    pub fn get_scheduler(&self) -> &EventScheduler<E> {
        &self.scheduler
    }

    /// getter for recorder
    pub fn get_recorder(&self) -> &Rec {
        &self.recorder
    }

    /// getter for recorder
    pub fn get_recorder_as_mut(&mut self) -> &mut Rec {
        &mut self.recorder
    }

    /// swap new and old recorder with get old recorder.
    pub fn swap_recorder(&mut self, new_recorder: Rec) -> Rec {
        mem::replace(&mut self.recorder, new_recorder)
    }
}

impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: BulkEvents<Rec, E>,
    E: Event,
{
    /// run simulate for one frame with bulk events
    pub fn run_step_in_bulk<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.model.start_frame(rng, &mut self.recorder);
        let fired: Vec<(Priority, E)> = self.scheduler.next_time_and_fire(rng);
        self.model
            .step_in_bulk_event(rng, &mut self.recorder, &mut self.scheduler, fired);
        self.model.finish_frame(rng, &mut self.recorder);
    }

    /// run simulate for frames with bulk events
    pub fn run_n_count_in_bulk<R: Rng + ?Sized, FC: FrameCounter>(
        &mut self,
        rng: &mut R,
        counter: FC,
    ) {
        let mut index = FC::start_index();
        loop {
            index.next_index();
            if !index.can_continue(&counter) {
                break;
            }

            self.run_step_in_bulk(rng);
        }
    }

    /// run simulation until condition is true with bulk events
    pub fn run_until_in_bulk<R: Rng + ?Sized, F>(&mut self, rng: &mut R, can_continue: F)
    where
        F: Fn(&M) -> bool,
    {
        loop {
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_in_bulk(rng);
        }
    }

    /// run simulation with update model's state with bulk events
    pub fn run_with_state_in_bulk<R: Rng + ?Sized, S, F, P>(
        &mut self,
        rng: &mut R,
        update_state: F,
        can_continue: P,
    ) where
        F: Fn(&mut M),
        P: Fn(&M) -> bool,
    {
        loop {
            update_state(&mut self.model);
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_in_bulk(rng);
        }
    }
}

impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: StepEachEvent<Rec, E>,
    E: Event,
{
    /// run simulate for one frame with calculate each event
    pub fn run_step_each_event<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.model.start_frame(rng, &mut self.recorder);
        let fired: Vec<(Priority, E)> = self.scheduler.next_time_and_fire(rng);
        for fe in fired.iter() {
            self.model
                .step_each_event(rng, &mut self.recorder, &mut self.scheduler, fe);
        }
        self.model.finish_frame(rng, &mut self.recorder);
    }

    /// run simulate for frames with calculate each event
    pub fn run_n_count_each_event<R: Rng + ?Sized, FC: FrameCounter>(
        &mut self,
        rng: &mut R,
        counter: FC,
    ) {
        let mut index = FC::start_index();
        loop {
            index.next_index();
            if !index.can_continue(&counter) {
                break;
            }

            self.run_step_each_event(rng);
        }
    }

    /// run simulation until condition is true with calculate each event
    pub fn run_until_each_event<R: Rng + ?Sized, F>(&mut self, rng: &mut R, can_continue: F)
    where
        F: Fn(&M) -> bool,
    {
        loop {
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_each_event(rng);
        }
    }

    /// run simulation with update model's state with calculate each event
    pub fn run_with_state_each_event<R: Rng + ?Sized, S, F, P>(
        &mut self,
        rng: &mut R,
        update_state: F,
        can_continue: P,
    ) where
        F: Fn(&mut M),
        P: Fn(&M) -> bool,
    {
        loop {
            update_state(&mut self.model);
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_each_event(rng);
        }
    }
}

impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: LastNoneEvent<Rec, E>,
    E: Event,
{
    /// run simulate for one frame with calculate each event with get None event after all fired events
    pub fn run_step_optional<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.model.start_frame(rng, &mut self.recorder);
        let fired: Vec<(Priority, E)> = self.scheduler.next_time_and_fire(rng);
        let mut fired_iter = fired.iter();
        loop {
            let fe = fired_iter.next();
            self.model
                .step_optional(rng, &mut self.recorder, &mut self.scheduler, fe);
            if fe.is_none() {
                break;
            }
        }
        self.model.finish_frame(rng, &mut self.recorder);
    }

    /// run simulate for frames with calculate each event with get None event after all fired events
    pub fn run_n_count_optional<R: Rng + ?Sized, FC: FrameCounter>(
        &mut self,
        rng: &mut R,
        counter: FC,
    ) {
        let mut index = FC::start_index();
        loop {
            index.next_index();
            if !index.can_continue(&counter) {
                break;
            }

            self.run_step_optional(rng);
        }
    }

    /// run simulation until condition is true with calculate each event with get None event after all fired events
    pub fn run_until_optional<R: Rng + ?Sized, F>(&mut self, rng: &mut R, can_continue: F)
    where
        F: Fn(&M) -> bool,
    {
        loop {
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_optional(rng);
        }
    }

    /// run simulation with update model's state with calculate each event with get None event after all fired events
    pub fn run_with_state_optional<R: Rng + ?Sized, S, F, P>(
        &mut self,
        rng: &mut R,
        update_state: F,
        can_continue: P,
    ) where
        F: Fn(&mut M),
        P: Fn(&M) -> bool,
    {
        loop {
            update_state(&mut self.model);
            if !can_continue(&self.model) {
                break;
            }

            self.run_step_optional(rng);
        }
    }
}
