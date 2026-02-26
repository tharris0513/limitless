use serde::Serialize;

pub mod abilities;
pub mod ability_effects;
pub mod adventures;
pub mod classes;
pub mod creatures;
pub mod locations;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Statistic {
    Might,
    Defense,
    Magic,
    Resistance,
    Agility,
}
