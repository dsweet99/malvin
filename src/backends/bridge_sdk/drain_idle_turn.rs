use crate::sdk_drain_timeout::sdk_drain_idle_timeout_from_env;

use super::bridge_sdk::drain_idle::DrainIdleClock;

pub(crate) struct DrainIdleTurn {
    pub(crate) clock: DrainIdleClock,
}

impl DrainIdleTurn {
    pub(crate) fn new() -> Self {
        Self {
            clock: DrainIdleClock::new(sdk_drain_idle_timeout_from_env()),
        }
    }
}
