use skrynia_program0::cap::Random;
pub struct Rng(pub u8);
impl Random for Rng {
    type Err = ();
    fn fill(&self, o: &mut [u8]) -> Result<(), ()> {
        for b in o {
            *b = self.0;
        }
        Ok(())
    }
}
