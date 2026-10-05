use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttemptCeiling {
    Unlimited,
    AtMost(NonZeroU32),
}

impl AttemptCeiling {
    #[must_use]
    pub(crate) const fn unlimited() -> Self {
        Self::Unlimited
    }

    #[must_use]
    pub(crate) const fn one() -> Self {
        Self::AtMost(NonZeroU32::MIN)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn at_most_u32(n: u32) -> Self {
        Self::AtMost(NonZeroU32::new(n).expect("attempt ceiling is at least 1"))
    }

    #[must_use]
    pub(crate) const fn allows_attempt(self, attempt: u32) -> bool {
        match self {
            Self::Unlimited => true,
            Self::AtMost(max) => attempt < max.get(),
        }
    }
}
