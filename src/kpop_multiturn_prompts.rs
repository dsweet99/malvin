pub trait KpopMultiturnPrompts {
    fn kpop_block(
        &mut self,
        want: usize,
        remaining_after_this_turn: usize,
    ) -> Result<String, String>;
    fn mbc2_pure(&mut self) -> Result<String, String>;
}
