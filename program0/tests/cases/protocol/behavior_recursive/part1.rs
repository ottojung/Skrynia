impl Eval for SelfEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        if w == b"self" && !i.is_empty() {
            self.reg
                .invoke(&*self.disk, self, &self.me, &i[1..])
                .map_err(|_| ())
        } else {
            Ok(vec![i.len() as u8])
        }
    }
}

#[test]
fn recursive_self_lookup() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(1), b"self".to_vec()).unwrap();
    let eval = SelfEval {
        reg: reg.clone(),
        disk: disk.clone(),
        me: id.clone(),
    };
    assert_eq!(reg.invoke(&*disk, &eval, &id, b"abc").unwrap(), vec![0]);
}

struct CycleEval {
    reg: Arc<Registry>,
    disk: Arc<MemDisk>,
    a: Vec<u8>,
    b: Vec<u8>,
}
impl Eval for CycleEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        match w {
            b"A" => self
                .reg
                .invoke(&*self.disk, self, &self.b, i)
                .map_err(|_| ()),
            b"B" if !i.is_empty() => self
                .reg
                .invoke(&*self.disk, self, &self.a, &i[1..])
                .map_err(|_| ()),
            _ => Ok(b"done".to_vec()),
        }
    }
}
