pub trait KpopMultiturnPrompts {
    /// Builds the next KPOP block prompt text.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the implementor cannot produce prompt text.
    fn kpop_block(
        &mut self,
        want: usize,
        remaining_after_this_turn: usize,
    ) -> Result<String, String>;

    /// Builds the next MBC2-only prompt text.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the implementor cannot produce prompt text.
    fn mbc2_pure(&mut self) -> Result<String, String>;
}
