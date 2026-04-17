pub mod state_0;

pub mod state_1;
pub use state_1::StateOne;

pub mod state_2;
pub use state_2::StateTwo;

#[derive(Debug, Default, Clone)]
pub enum State {
    #[default]
    Zero,
    One(StateOne),
    Two(StateTwo),
}

impl State {
    pub fn index(&self) -> u32 {
        match self {
            State::Zero   => 0,
            State::One(_) => 1,
            State::Two(_) => 2,
        }
    }
}
