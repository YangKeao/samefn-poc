//! Version 1 of sin must reproduce its frozen corpus bit for bit, through every entry point.

use samefn::sin;

fn corpus() -> Vec<(f64, f64)> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/sin_v1.tsv"))
        .expect("read corpus/sin_v1.tsv");
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let (x, y) = l.split_once('\t').expect("two columns");
            let p = |h: &str| f64::from_bits(u64::from_str_radix(h, 16).unwrap());
            (p(x), p(y))
        })
        .collect()
}

fn same(got: f64, want: f64) -> bool {
    (got.is_nan() && want.is_nan()) || got.to_bits() == want.to_bits()
}

#[test]
fn v1_matches_corpus() {
    let c = corpus();
    assert!(c.len() >= 10_000);
    for &(x, want) in &c {
        let got = sin::v1(x);
        assert!(same(got, want), "sin v1({x:e}) = {got:e}, frozen {want:e}");
    }
}

#[test]
fn batches_match_corpus() {
    let c = corpus();
    let input: Vec<f64> = c.iter().map(|p| p.0).collect();
    let mut out = vec![0.0; input.len()];
    sin::batch_v1(&input, &mut out);
    let mut in_place = input.clone();
    sin::batch_v1_in_place(&mut in_place);
    let mut c_abi = input.clone();
    unsafe { samefn::samefn_sin_batch_v1(c_abi.as_ptr(), c_abi.as_mut_ptr(), c_abi.len()) };
    for (i, &(_, want)) in c.iter().enumerate() {
        assert!(same(out[i], want));
        assert!(same(in_place[i], want));
        assert!(same(c_abi[i], want));
    }
}

#[test]
fn versions() {
    assert_eq!(samefn::max_version("sin"), 1);
    assert_eq!(samefn::max_version("cos"), 1);
    assert_eq!(unsafe { samefn::samefn_max_version(c"sin".as_ptr()) }, 1);
}
