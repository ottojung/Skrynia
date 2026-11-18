use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::cap::Eval;
use skrynia_program0::reg::Registry;
use std::sync::Arc;

mod eval;
use eval::{CycleEval, SelfEval};
mod case0;
mod case1;
