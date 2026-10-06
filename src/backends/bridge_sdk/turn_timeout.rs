pub(crate) trait TurnTimeoutExtension: Sync {
    fn tools_in_flight(&self) -> bool;
}
