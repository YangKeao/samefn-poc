//! Wrote corpus/sin_v1.tsv once, when version 1 was frozen. Kept to document how the corpus
//! was made; do not regenerate it. A changed line means a changed result, which needs a new
//! version instead.
//!
//!     cargo run --release --example gen_corpus -- <CORE-MATH sin.wc> > corpus/sin_v1.tsv

fn main() {
    let mut inputs: Vec<f64> = Vec::new();

    // Special and boundary values.
    let specials = [
        0.0, f64::MIN_POSITIVE, f64::from_bits(1), f64::from_bits(0x000f_ffff_ffff_ffff),
        f64::EPSILON, 1e-300, 2f64.powi(-26), 2f64.powi(-27), 0.5, 1.0, 2.0, 3.0,
        std::f64::consts::PI, std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_4,
        std::f64::consts::FRAC_PI_6, 2.0 * std::f64::consts::PI, 1e6, 1e15, 1e22, 2f64.powi(52),
        2f64.powi(1023), 1674610777.0, 279935.5950394922, 393038.1174444391, f64::MAX,
        f64::INFINITY,
    ];
    for x in specials {
        inputs.push(x);
        inputs.push(-x);
    }
    inputs.push(f64::NAN);

    // Every 1000th upstream hard-to-round case, and its negation.
    if let Some(path) = std::env::args().nth(1) {
        let text = std::fs::read_to_string(path).expect("read sin.wc");
        let hard = text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
        for line in hard.step_by(1000) {
            // First token only: some lines carry a trailing "# comment".
            let x = parse_hex_float(line.split_whitespace().next().unwrap());
            inputs.push(x);
            inputs.push(-x);
        }
    }

    // Seeded random inputs (xorshift64*), as in ctest/crosscheck_mpfr.c.
    let mut s: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        s.wrapping_mul(0x2545_f491_4f6c_dd1d)
    };
    let mut i = 0u64;
    while inputs.len() < 10_000 {
        let r = next();
        let x = match i & 3 {
            0 | 1 => f64::from_bits(r),
            2 => ((r >> 11) as f64 * 2f64.powi(-53) * 2.0 - 1.0) * 10.0,
            _ => ((r >> 11) as f64 * 2f64.powi(-53) * 2.0 - 1.0) * 1e6,
        };
        i += 1;
        if !x.is_nan() {
            inputs.push(x);
        }
    }

    println!("# samefn sin version 1, frozen. Columns: input, output, as IEEE 754 binary64 bit");
    println!("# patterns in hex. A NaN output matches any NaN. Never regenerate: a changed");
    println!("# line is a changed result, which needs a new version.");
    for x in inputs {
        println!("{:016x}\t{:016x}", x.to_bits(), samefn::sin::v1(x).to_bits());
    }
}

/// Parses `[-]0x1.<hex>p<exp>` as written by C's `%a` (finite, normal or subnormal).
fn parse_hex_float(s: &str) -> f64 {
    let (neg, s) = s.strip_prefix('-').map_or((false, s), |r| (true, r));
    let s = s.strip_prefix("0x").expect("0x prefix");
    let (mant, exp) = s.split_once('p').expect("p exponent");
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let mut m = u64::from_str_radix(int, 16).unwrap() as f64;
    let mut scale = 1.0 / 16.0;
    for c in frac.chars() {
        m += c.to_digit(16).unwrap() as f64 * scale; // exact: at most 13 hex digits
        scale /= 16.0;
    }
    let v = m * 2f64.powi(exp.parse::<i32>().unwrap());
    if neg { -v } else { v }
}
