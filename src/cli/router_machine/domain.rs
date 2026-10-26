use super::ProtocolError;
use std::num::NonZeroUsize;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Completion {
    Impasse,
    Satisfied,
    StillOpen,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Iteration {
    current: NonZeroUsize,
    max: NonZeroUsize,
}
impl Iteration {
    pub(crate) fn new(current: usize, max: usize) -> Result<Self, ProtocolError> {
        let current =
            NonZeroUsize::new(current).ok_or(ProtocolError::InvalidIteration { current, max })?;
        let max = NonZeroUsize::new(max).ok_or_else(|| ProtocolError::InvalidIteration {
            current: current.get(),
            max,
        })?;
        if current > max {
            return Err(ProtocolError::InvalidIteration {
                current: current.get(),
                max: max.get(),
            });
        }
        Ok(Self { current, max })
    }
    pub(crate) const fn current(self) -> usize {
        self.current.get()
    }
    pub(crate) const fn is_last(self) -> bool {
        self.current.get() == self.max.get()
    }
    pub(super) fn next(self) -> Self {
        Self::new(self.current() + 1, self.max.get()).expect("valid next iteration")
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RunOutcome {
    Succeeded,
    GatesFailed,
    AgentFailed,
}
