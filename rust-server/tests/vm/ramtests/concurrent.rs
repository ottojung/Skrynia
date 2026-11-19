use super::mutation::kernel;
use std::io::{Result as IoResult, Write};
use std::sync::{Arc, mpsc};

struct Hold {
    entered: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
}

impl Write for Hold {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        self.entered.send(()).unwrap();
        self.resume.recv().unwrap();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

#[test]
fn two_requests_never_own_the_mutation_lock_simultaneously() {
    let kernel = Arc::new(kernel());
    let (entered, seen) = mpsc::channel();
    let (resume, go) = mpsc::channel();
    let worker = {
        let kernel = kernel.clone();
        std::thread::spawn(move || {
            kernel.invoke_stream(
                Box::new(std::io::Cursor::new(b"H")),
                16,
                Box::new(Hold {
                    entered,
                    resume: go,
                }),
            )
        })
    };
    seen.recv_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    assert_eq!(kernel.invoke(b"B"), Err(-10));
    resume.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    assert_eq!(kernel.invoke(b"Y").unwrap(), b"Y");
}
