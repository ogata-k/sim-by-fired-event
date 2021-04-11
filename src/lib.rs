//! Simulator is discrete time simulator with event which fire at scheduled timing.

use crate::event::{Event, EventScheduler, Priority};
use crate::model::{BulkEvents, Model, NothingEventModel, StepEachEvent};
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

/// simulator for Nothing event
#[derive(Debug, Clone)]
pub struct NothingEventSimulator<M, Rec>
where
    M: NothingEventModel<Rec>,
{
    model: M,
    recorder: Rec,
}

impl<M, Rec> NothingEventSimulator<M, Rec>
where
    M: NothingEventModel<Rec>,
{
    /// create as default
    pub fn new() -> Self
    where
        M: Default,
        Rec: Default,
    {
        let mut sim = Self {
            model: Default::default(),
            recorder: Default::default(),
        };
        sim.initialize();
        sim
    }

    /// create simulator from model
    pub fn create_from(model: M, recorder: Rec) -> Self {
        let mut sim = Self { model, recorder };
        sim.initialize();
        sim
    }

    /// initialize simulator
    fn initialize(&mut self) {
        self.model.initialize(&mut self.recorder);
    }

    /// getter for model
    pub fn get_model(&self) -> &M {
        &self.model
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

    //
    // run simulation
    //

    /// run simulate for one frame
    pub fn run_step(&mut self) {
        self.model.start_frame(&mut self.recorder);
        self.model.step(&mut self.recorder);
        self.model.finish_frame(&mut self.recorder);
    }

    /// run simulate for frames
    pub fn run_n<FC: FrameCounter>(&mut self, counter: FC) {
        let mut index = FC::start_index();
        loop {
            index.next_index();
            if !index.can_continue(&counter) {
                break;
            }

            self.run_step();
        }
    }

    /// run simulation until condition is true
    pub fn run_until<F>(&mut self, can_continue: F)
    where
        F: Fn(&M) -> bool,
    {
        loop {
            if !can_continue(&self.model) {
                break;
            }

            self.run_step();
        }
    }

    /// run simulation with update model's state
    pub fn run_with_state<F, P>(&mut self, update_state: F, can_continue: P)
    where
        F: Fn(&mut M),
        P: Fn(&M) -> bool,
    {
        loop {
            update_state(&mut self.model);
            if !can_continue(&self.model) {
                break;
            }

            self.run_step();
        }
    }
}

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

    //
    // run simulation
    //

    /// run simulate for one frame
    pub fn run_step<R: Rng + ?Sized, H>(&mut self, rng: &mut R, mut handler: H)
    where
        H: FnMut(&mut R, &mut M, &mut Rec, &mut EventScheduler<E>, Vec<(Priority, E)>),
    {
        self.model.start_frame(&mut self.recorder);
        let fired_events: Vec<(Priority, E)> = self.scheduler.next_time_and_fire(rng);
        self.model
            .before_first_event(rng, &mut self.recorder, &mut self.scheduler);
        handler(
            rng,
            &mut self.model,
            &mut self.recorder,
            &mut self.scheduler,
            fired_events,
        );
        self.model
            .after_last_event(rng, &mut self.recorder, &mut self.scheduler);

        self.model.finish_frame(&mut self.recorder);
    }

    /// run simulate for frames
    pub fn run_n<R: Rng + ?Sized, FC: FrameCounter, H>(
        &mut self,
        rng: &mut R,
        counter: FC,
        mut handler: H,
    ) where
        H: FnMut(&mut R, &mut M, &mut Rec, &mut EventScheduler<E>, Vec<(Priority, E)>),
    {
        let mut index = FC::start_index();
        loop {
            index.next_index();
            if !index.can_continue(&counter) {
                break;
            }

            self.run_step(rng, |rng, model, recorder, scheduler, events| {
                handler(rng, model, recorder, scheduler, events)
            });
        }
    }

    /// run simulation until condition is true
    pub fn run_until<R: Rng + ?Sized, F, H>(&mut self, rng: &mut R, can_continue: F, mut handler: H)
    where
        F: Fn(&M) -> bool,
        H: FnMut(&mut R, &mut M, &mut Rec, &mut EventScheduler<E>, Vec<(Priority, E)>),
    {
        loop {
            if !can_continue(&self.model) {
                break;
            }

            self.run_step(rng, |rng, model, recorder, scheduler, events| {
                handler(rng, model, recorder, scheduler, events)
            });
        }
    }

    /// run simulation with update model's state
    pub fn run_with_state<R: Rng + ?Sized, F, P, H>(
        &mut self,
        rng: &mut R,
        update_state: F,
        can_continue: P,
        mut handler: H,
    ) where
        F: Fn(&mut M),
        P: Fn(&M) -> bool,
        H: FnMut(&mut R, &mut M, &mut Rec, &mut EventScheduler<E>, Vec<(Priority, E)>),
    {
        loop {
            update_state(&mut self.model);
            if !can_continue(&self.model) {
                break;
            }

            self.run_step(rng, |rng, model, recorder, scheduler, events| {
                handler(rng, model, recorder, scheduler, events)
            });
        }
    }
}

// TODO If concat_idents macro is to be stable, then replace $suffix:ident and concat_idents!.
macro_rules! impl_base_set {
    ($handler:ident, [$run_step:ident,$run_n:ident,$run_until:ident,$run_with_state:ident]) => {
        /// run simulate for one frame
        pub fn $run_step<R: Rng + ?Sized>(&mut self, rng: &mut R) {
            self.model.start_frame(&mut self.recorder);
            let fired_events: Vec<(Priority, E)> = self.scheduler.next_time_and_fire(rng);
            self.model
                .before_first_event(rng, &mut self.recorder, &mut self.scheduler);
            self.$handler(rng, fired_events);
            self.model
                .after_last_event(rng, &mut self.recorder, &mut self.scheduler);

            self.model.finish_frame(&mut self.recorder);
        }

        /// run simulate for frames
        pub fn $run_n<R: Rng + ?Sized, FC: FrameCounter>(&mut self, rng: &mut R, counter: FC) {
            let mut index = FC::start_index();
            loop {
                index.next_index();
                if !index.can_continue(&counter) {
                    break;
                }
                self.$run_step(rng);
            }
        }

        /// run simulation until condition is true
        pub fn $run_until<R: Rng + ?Sized, F>(&mut self, rng: &mut R, can_continue: F)
        where
            F: Fn(&M) -> bool,
        {
            loop {
                if !can_continue(&self.model) {
                    break;
                }
                self.$run_step(rng);
            }
        }

        /// run simulation with update model's state
        pub fn $run_with_state<R: Rng + ?Sized, S, F, P>(
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
                self.$run_step(rng);
            }
        }
    };
}

/// simulate for fired event with calculate in bulk
impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: BulkEvents<Rec, E>,
    E: Event,
{
    fn handler_in_bulk_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        fired_events: Vec<(Priority, E)>,
    ) {
        self.model
            .step_in_bulk(rng, &mut self.recorder, &mut self.scheduler, fired_events);
    }

    impl_base_set!(
        handler_in_bulk_event,
        [
            run_step_in_bulk_event,
            run_n_in_bulk_event,
            run_until_in_bulk_event,
            run_with_state_in_bulk_event
        ]
    );
}

/// simulate for fired event with calculate each event
impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: StepEachEvent<Rec, E>,
    E: Event,
{
    fn handler_each_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        fired_events: Vec<(Priority, E)>,
    ) {
        for (p, e) in fired_events.into_iter() {
            self.model
                .step_each_event(rng, &mut self.recorder, &mut self.scheduler, p, e);
        }
    }

    impl_base_set!(
        handler_each_event,
        [
            run_step_each_event,
            run_n_each_event,
            run_until_each_event,
            run_with_state_each_event
        ]
    );
}
