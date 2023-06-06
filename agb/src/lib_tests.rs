#[cfg(test)]
mod tests {
    use agb_fixnum::{num, Num};

    #[test_case]
    fn benchmark_fixnum_long_multiply(_: &mut crate::Gba) {
        let x: Num<i32, 16> = num!(2893.5588682686);
        let y: Num<i32, 16> = num!(26596.529373983);

        for _ in 0..10000 {
            let r = core::hint::black_box(x) * core::hint::black_box(y);
            core::hint::black_box(r);
        }
    }
}
