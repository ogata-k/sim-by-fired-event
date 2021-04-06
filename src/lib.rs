//! Simulator is discrete time simulator with event which fire at scheduled timing.

use crate::event::{Event, EventScheduler};
use crate::model::Model;
use rand::Rng;
use std::mem;

pub mod event;
pub mod model;

/// TimeCounter for user
pub trait FrameCounter: Copy {
    /// start state. this value used always.
    fn start_state() -> Self;

    /// get (can continue flag, next state)
    fn next_state(&self, specified: &Self) -> (bool, Self);
}

macro_rules! impl_counter {
    ($t:ty, $i:ident) => {
        impl FrameCounter for $t {
            fn start_state() -> $t {
                $i::MIN
            }

            fn next_state(&self, specified: &$t) -> (bool, $t) {
                let next = self + 1;
                (&next <= specified, next)
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

impl<M, E, Rec> Default for Simulator<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E> + Default,
    E: Event,
    Rec: Default,
{
    fn default() -> Self {
        Self {
            model: Default::default(),
            recorder: Default::default(),
            scheduler: EventScheduler::new(),
        }
    }
}

impl<M, E, Rec> Simulator<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E>,
    E: Event,
{
    /// create simulator from model
    pub fn create_from(model: M, recorder: Rec) -> Self {
        Self {
            model,
            recorder,
            scheduler: EventScheduler::new(),
        }
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

    // -----------
    // methods for run simulation
    // -----------

    /// initialize simulator
    pub fn initialize<R: Rng + ?Sized>(&mut self, rng: &mut R) -> &mut Self {
        self.scheduler.clear();
        self.model.initialize(rng, &mut self.recorder);
        self.model
            .initialize_frame(rng, &mut self.recorder, &mut self.scheduler);
        self
    }

    /// run simulate for frames
    pub fn run<R: Rng + ?Sized, FC: FrameCounter>(&mut self, rng: &mut R, counter: FC) {
        let mut index = FC::start_state();
        loop {
            let (can_continue, next) = index.next_state(&counter);
            if !can_continue {
                break;
            }
            index = next;

            self.step(rng);
        }
    }

    /// run simulate for one frame
    pub fn step<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let mut fired: Vec<E> = self.scheduler.next_time_and_fire(rng);
        self.model
            .step(rng, &mut self.recorder, &mut self.scheduler, &mut fired);
    }

    /// run simulation until condition is true
    pub fn run_until<R: Rng + ?Sized, F>(&mut self, rng: &mut R, judge: F)
    where
        F: Fn(&mut Rec, &M) -> bool,
    {
        while judge(&mut self.recorder, &self.model) {
            self.step(rng);
        }
    }
}
