use super::{Reader, Run, Writer};
use std::{io, path::Path};
impl Run {
    pub fn merge(dir: &Path, a: &Run, b: &Run) -> io::Result<Self> {
        let mut left = Reader::new(a)?;
        let mut right = Reader::new(b)?;
        let mut x = left.next()?;
        let mut y = right.next()?;
        let mut output = Writer::new(dir)?;
        while x.is_some() || y.is_some() {
            match (x.as_ref(), y.as_ref()) {
                (Some((kx, _)), Some((ky, _))) if kx == ky => {
                    let (key, dx) = x.take().unwrap();
                    let (_, dy) = y.take().unwrap();
                    output.push(&key, dx.plus(dy)?)?;
                    x = left.next()?;
                    y = right.next()?;
                }
                (Some((kx, _)), Some((ky, _))) if kx > ky => {
                    let (key, delta) = y.take().unwrap();
                    output.push(&key, delta)?;
                    y = right.next()?;
                }
                (Some(_), _) => {
                    let (key, delta) = x.take().unwrap();
                    output.push(&key, delta)?;
                    x = left.next()?;
                }
                (None, Some(_)) => {
                    let (key, delta) = y.take().unwrap();
                    output.push(&key, delta)?;
                    y = right.next()?;
                }
                (None, None) => break,
            }
        }
        output.finish()
    }
}
