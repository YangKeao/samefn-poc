use std::{hint::black_box, time::Instant};
fn main() {
    let input: Vec<f64> = (0..1024)
        .map(|i| ((i * 15427 + 431) % 200001) as f64 / 10000. - 10.)
        .collect();
    for (op, native, shared) in [
        (
            "sin",
            f64::sin as fn(f64) -> f64,
            samefn::sin::v1 as fn(f64) -> f64,
        ),
        ("cos", f64::cos, samefn::cos::v1),
        ("tan", f64::tan, samefn::tan::v1),
        ("exp", f64::exp, samefn::exp::v1),
    ] {
        for (mode, f) in [("native", native), ("shared", shared)] {
            for rep in 0..7 {
                let start = Instant::now();
                let mut sum = 0.;
                for _ in 0..20000 {
                    for &x in &input {
                        sum += black_box(f(black_box(x)));
                    }
                }
                let ns = start.elapsed().as_nanos() as f64 / (20000. * 1024.);
                black_box(sum);
                println!("{op},{mode},{rep},{ns:.5}");
            }
        }
    }
}
