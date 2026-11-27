use super::Ram;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 256 * 1024;
const ROUNDS: u32 = 32;

#[test]
#[ignore = "throughput comparison: cargo test --lib ram::tests::bench -- --ignored --nocapture"]
fn bulk_outperforms_byte_loops() {
    let r = Ram::new();
    r.truncate(N as u64).unwrap();
    let src = vec![0x5a; N];
    let mut dst = vec![0u8; N];
    for i in 0..N {
        r.write_byte(i as u64, 0xa5).unwrap();
    }
    let bw = time(|| {
        for (i, b) in src.iter().enumerate() {
            r.write_byte(i as u64, *b).unwrap();
        }
    });
    let pw = time(|| {
        r.write(0, black_box(&src)).unwrap();
    });
    let br = time(|| {
        for (i, b) in dst.iter_mut().enumerate() {
            *b = r.read_byte(i as u64).unwrap();
        }
    });
    let pr = time(|| {
        r.read(0, black_box(&mut dst)).unwrap();
    });
    assert_eq!(dst, src);
    println!("write byte {:.1} MB/s page {:.1} MB/s", mb(bw), mb(pw));
    println!("read  byte {:.1} MB/s page {:.1} MB/s", mb(br), mb(pr));
}

fn time(mut f: impl FnMut()) -> f64 {
    let start = Instant::now();
    for _ in 0..ROUNDS {
        f();
    }
    start.elapsed().as_secs_f64() / f64::from(ROUNDS)
}
fn mb(s: f64) -> f64 {
    N as f64 / s / 1e6
}
