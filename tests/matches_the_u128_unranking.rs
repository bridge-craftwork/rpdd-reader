//! The narrow arithmetic must produce exactly the deals the wide arithmetic did.
//!
//! `reproduces_the_library.rs` is the check that matters, but it can only see
//! the 81,920 deals the fixture has digests for. This one holds a second,
//! deliberately naive generator — the `u128` unranking the 32-bit limbs and the
//! `u64` tail replaced — and compares it deal for deal over a million or so,
//! from starts scattered through the library including its last group.
//!
//! It is `#[ignore]`d because it takes about two seconds in release and
//! considerably longer in debug, and because it is a regression guard rather
//! than a fact about the library:
//!
//! ```text
//! cargo test --release -- --ignored
//! ```
//!
//! Keep it if the unranking is touched again. The reference below is the
//! shipping code of bridge-craftwork/rpdd-library@74d2407, transcribed
//! unchanged — the crate lived there until it was split out; both it and the
//! crate came from `docs/xxdd-disassembly.asm`, so agreeing with it is evidence
//! the narrowing preserved the algorithm, not evidence about the constants.

/// The generator as it stood before the 32-bit limbs: every step in `u128`.
mod reference {
    const TOTAL: u128 = 53_644_737_765_488_792_839_237_440_000; // 52!/(13!)^4

    #[derive(Default)]
    pub struct Rng {
        s: [u32; 4],
        c: u32,
    }

    impl Rng {
        fn next(&mut self) -> u32 {
            let p = 0x13FBu64 * self.s[0] as u64
                + 0x6F0u64 * self.s[1] as u64
                + 0x5D4u64 * self.s[2] as u64
                + 0x7DD4FFC7u64 * self.s[3] as u64
                + self.c as u64;
            self.s[3] = self.s[2];
            self.s[2] = self.s[1];
            self.s[1] = self.s[0];
            self.s[0] = p as u32;
            self.c = (p >> 32) as u32;
            self.s[0]
        }

        fn seed(&mut self, n: u32) {
            let mut a = (n ^ 0xFFFF).rotate_right(10);
            a = (a & 0xFFFF_0000) | (n & 0xFFFF);
            a = a.rotate_right(10);
            let d = n << 2;
            a |= d & 0xFF00;
            a |= d & 0x00FF;
            a |= 3;
            let mut w = [0u32; 5];
            for x in w.iter_mut() {
                a = a.wrapping_mul(0x01C8_E815).wrapping_sub(1);
                *x = a;
            }
            self.s = [w[0], w[1], w[2], w[3]];
            self.c = w[4];
            for _ in 0..12 {
                self.next();
            }
        }
    }

    pub fn deal(rng: &mut Rng, i: u64) -> [u8; 13] {
        if i & 0x3FFF == 0 {
            rng.seed((i >> 14) as u32 + 1);
        }
        let (r0, r1, r2) = (rng.next() as u128, rng.next() as u128, rng.next() as u128);
        let lo64 = (r1 << 32) | r0;
        let m: u128 = if lo64 >= 0x634D_DA65_8BF4_9200 {
            0xAD55_E315
        } else {
            0xAD55_E316
        };
        let mut r = (((r2 * m) >> 32) << 64) | lo64;

        let mut p = TOTAL;
        let mut cnt = [13u128; 4];
        let mut cards = [0u8; 52];
        for ebp in (1..=52u128).rev() {
            let mut k = 3usize;
            let mut q = p * cnt[k] / ebp;
            while q <= r && k > 0 {
                r -= q;
                k -= 1;
                q = p * cnt[k] / ebp;
            }
            cnt[k] -= 1;
            cards[52 - ebp as usize] = 3 - k as u8;
            p = q;
        }
        let mut out = [0u8; 13];
        for (j, &s) in cards.iter().enumerate() {
            out[j >> 2] |= s << (2 * (j & 3));
        }
        out
    }

    /// Deals from `start`, winding forward from the seed boundary below it.
    pub fn deals_from(start: u64) -> impl Iterator<Item = [u8; 13]> {
        let mut rng = Rng::default();
        for index in (start - start % 16_384)..start {
            deal(&mut rng, index);
        }
        let mut next = start;
        std::iter::from_fn(move || {
            let packed = deal(&mut rng, next);
            next += 1;
            Some(packed)
        })
    }
}

/// Scattered starts: a group boundary, one deal past it, the last deal of a
/// group, an unaligned start, the middle of the library and its final group.
const STARTS: [(u64, usize); 8] = [
    (0, 300_000),
    (1, 150_000),
    (16_383, 150_000),
    (99_999, 150_000),
    (1_048_576, 150_000),
    (4_096_000, 150_000),
    (8_192_000, 150_000),
    (rpdd::LIBRARY_DEALS - 200_000, 150_000),
];

#[test]
#[ignore = "about two seconds in release; run with --ignored when the unranking changes"]
fn the_narrow_arithmetic_deals_what_the_wide_arithmetic_did() {
    let mut checked = 0usize;
    for (start, count) in STARTS {
        let wide = reference::deals_from(start).take(count);
        let narrow = rpdd::Deals::from(start).take(count);
        for (i, (a, b)) in wide.zip(narrow).enumerate() {
            assert_eq!(a, b, "deal {} differs", start + i as u64);
            checked += 1;
        }
    }

    // And the last deal in the library, reached by seeking rather than walking.
    let last = rpdd::LIBRARY_DEALS - 1;
    let mut rng = reference::Rng::default();
    for index in (last - last % 16_384)..last {
        reference::deal(&mut rng, index);
    }
    assert_eq!(reference::deal(&mut rng, last), rpdd::deal_at(last));
    checked += 1;

    assert_eq!(checked, 1_350_001, "fewer deals compared than intended");
}
