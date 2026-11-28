use std::num::NonZeroUsize;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvalidIteration {
    pub(crate) current: usize,
    pub(crate) max: usize,
}
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
    pub(crate) fn new(current: usize, max: usize) -> Result<Self, InvalidIteration> {
        let invalid = InvalidIteration { current, max };
        let (Some(current), Some(max)) = (NonZeroUsize::new(current), NonZeroUsize::new(max))
        else {
            return Err(invalid);
        };
        if current > max {
            return Err(invalid);
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
