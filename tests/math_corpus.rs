#[test]
fn additional_math_matches_independent_mpfr_bits() {
    for name in ["cos", "tan", "exp"] {
        let text = std::fs::read_to_string(format!(
            "{}/corpus/{name}_v1.tsv",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let pairs: Vec<(f64, f64)> = text
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let (a, b) = l.split_once('\t').unwrap();
                (
                    f64::from_bits(u64::from_str_radix(a, 16).unwrap()),
                    f64::from_bits(u64::from_str_radix(b, 16).unwrap()),
                )
            })
            .collect();
        let scalar: fn(f64) -> f64 = match name {
            "cos" => samefn::cos::v1,
            "tan" => samefn::tan::v1,
            _ => samefn::exp::v1,
        };
        let mut batch: Vec<f64> = pairs.iter().map(|p| p.0).collect();
        match name {
            "cos" => samefn::cos::batch_v1_in_place(&mut batch),
            "tan" => samefn::tan::batch_v1_in_place(&mut batch),
            _ => samefn::exp::batch_v1_in_place(&mut batch),
        };
        for (i, &(x, want)) in pairs.iter().enumerate() {
            for got in [scalar(x), batch[i]] {
                assert!(
                    (want.is_nan() && got.is_nan()) || got.to_bits() == want.to_bits(),
                    "{name} input={x:?} want={want:?} got={got:?}"
                );
            }
        }
    }
}
