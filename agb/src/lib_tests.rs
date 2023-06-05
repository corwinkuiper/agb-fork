#[cfg(test)]
mod tests {
    use agb_fixnum::Num;

    #[test_case]
    fn benchmark_fixnum_long_multiply(_: &mut crate::Gba) {
        let mask = (!0) >> 10;

        for _ in 0..10000 {
            let x = Num::<i32, 16>::from_raw(crate::rng::gen() & mask);
            let y = Num::<i32, 16>::from_raw(crate::rng::gen() & mask);

            let r = x * y;
            core::hint::black_box(r);
        }
    }
}
