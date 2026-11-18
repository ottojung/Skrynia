impl Eval for CatEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        Ok([w, b":", i].concat())
    }
}
