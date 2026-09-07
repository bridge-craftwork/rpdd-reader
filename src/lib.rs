//! The deals of Richard Pavlicek's solved-deal library, from their index.
//!
//! His `rpdd.zip` ships the double-dummy results (`rpdd.zdd`) together with
//! `xxdd.exe`, a 2,560-byte program that recreates the deals those results
//! belong to. Only the results are data: the deals are a pure function of their
//! index, and his own documentation says as much — making them "was easy", and
//! the two years of computer time went on solving them.
//!
//! This crate is that function, ported from the program. It has no
//! dependencies and knows nothing about tables: an index goes in, a packed deal
//! comes out, and pairing it with a table is the caller's business.
//!
//! The tables themselves are not here and never will be — they are 100 MiB
//! that will never change again, published as fetchable chunks by
//! [rpdd-library](https://github.com/bridge-craftwork/rpdd-library).
//!
//! # Seeking is cheap
//!
//! The generator re-seeds every 16,384 deals from the group index, so deal N
//! costs at most 16,383 deals of catch-up rather than replaying from zero. At
//! roughly 640ns a deal that is about 10ms for the worst case, which is what
//! makes an arbitrary starting position practical. A wasm32 build costs about
//! the same: the unranking is deliberately written in arithmetic that target
//! has natively.
//!
//! ```
//! use rpdd_reader::Deals;
//!
//! // Anywhere in the library, without generating what comes before it.
//! let mut deals = Deals::from(4_096_000);
//! let packed = deals.next().expect("the library is 10,485,760 deals long");
//! assert_eq!(packed.len(), 13);
//! ```
//!
//! # The packed form
//!
//! Thirteen bytes, two bits per card, holding the seat: `00` West, `01` North,
//! `10` East, `11` South. Card order is SA, SK, ... S2, then the same
//! descending run for hearts, diamonds and clubs; bits are numbered least
//! significant first. That is the deal half of a `.zrd` record exactly, so it
//! can be handed straight to a decoder for that format.
//!
//! # Provenance
//!
//! Recovered from `xxdd.exe` by disassembly. `docs/xxdd-disassembly.asm` is the
//! annotated listing every constant below came from, and
//! `docs/reference-implementation.py` is a Python transcription kept beside it.
//! Neither is in the published crate — they are in the repository,
//! <https://github.com/bridge-craftwork/rpdd>. Do not "clean up" a constant
//! without checking them: the constants are not arbitrary, and nothing about a
//! wrong one fails loudly — it simply produces deals that are not his.

#![forbid(unsafe_code)]

/// How many deals the published library holds.
pub const LIBRARY_DEALS: u64 = 10_485_760;

/// How often the generator re-seeds, and therefore how far a seek must catch up.
pub const SEED_GROUP: u64 = 16_384;

/// Bytes in one packed deal.
pub const DEAL_LEN: usize = 13;

const TOTAL: u128 = 53_644_737_765_488_792_839_237_440_000; // 52!/(13!)^4

/// Where the unranking's tail fits in 64 bits.
///
/// Below this, `p * cnt[k]` cannot overflow a `u64` — `cnt[k]` is at most 13
/// and `13 * 2^60 < 2^64` — and `r < p` holds after every iteration, so `r`
/// comes below it too. Two thirds of the 52 iterations run under it.
const U64_TAIL: u128 = 1 << 60;

#[derive(Default)]
struct Rng {
    s: [u32; 4],
    c: u32,
}

impl Rng {
    /// Lag-4 multiply-with-carry step (xxdd.exe @0x4012F9).
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
    /// Seed from group index n = (deal_index >> 14) + 1 (xxdd.exe @0x4011BF).
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

/// `p * c / d`, in the 32-bit limbs xxdd.exe itself used (@0x40136E).
///
/// The original is a 32-bit program, so this *is* its arithmetic: three
/// `mull %ecx` with the high half carried into the next (@0x401370-0x40138D),
/// then three `divl %ebp` from the top limb down with each remainder feeding
/// the next through `edx` (@0x40138F-0x40139D). The `u128` spelling this
/// replaces was the reformulation; the constants are untouched either way.
///
/// Exact over the ranges the unranking uses, which are the ranges the original
/// relied on: `p < 2^96`, `c = cnt[k] <= 13`, `d = ebp <= 52`, and `c <= d`
/// always because the four counts sum to `ebp`. So `p * c` occupies four
/// limbs, the quotient fits back into three, and no division here can
/// overflow — as none did in the original, where it would have trapped.
///
/// It is also what wasm32 wants: there, `p as u128 * c as u128 / d as u128`
/// compiles to `__multi3` and `__udivti3` calls, while every operation below
/// is a single instruction.
#[inline]
fn mul_div(p: u128, c: u32, d: u32) -> u128 {
    debug_assert_eq!(p >> 96, 0, "p must fit the three limbs xxdd.exe gave it");
    debug_assert!(c <= d, "cnt[k] <= ebp, or the quotient would not fit");
    let (c, d) = (c as u64, d as u64);

    let mut prod = [0u32; 3];
    let mut carry = 0u64;
    for (out, limb) in prod
        .iter_mut()
        .zip([p as u32, (p >> 32) as u32, (p >> 64) as u32])
    {
        let t = limb as u64 * c + carry;
        *out = t as u32;
        carry = t >> 32;
    }

    // `carry` is the fourth limb of the product: the `edx` the last `mull`
    // left behind, which is where the original starts dividing.
    //
    // Indexed rather than zipped deliberately. `q.iter_mut().zip(prod).rev()`
    // reads better and measures 20% slower on aarch64 — it gives back the
    // whole of this change. Leave it indexed.
    let mut rem = carry;
    let mut q = [0u32; 3];
    for i in (0..3).rev() {
        let cur = (rem << 32) | prod[i] as u64;
        q[i] = (cur / d) as u32;
        rem = cur % d;
    }
    q[0] as u128 | ((q[1] as u128) << 32) | ((q[2] as u128) << 64)
}

/// The 13-byte packed deal at absolute index `i`.
///
/// `rng` must have been stepped over every deal since the last 16,384
/// boundary; [`Deals`] is what guarantees that.
fn deal(rng: &mut Rng, i: u64) -> [u8; 13] {
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
    let mut cnt = [13u32; 4]; // index i -> seat 3-i (0=W,1=N,2=E,3=S)
    let mut cards = [0u8; 52];
    for ebp in (1..=52u32).rev() {
        let mut k = 3usize;
        if p < U64_TAIL && r < U64_TAIL {
            // The tail: p has shrunk far enough that the whole step is
            // ordinary 64-bit arithmetic. Same sequence, narrower registers.
            let pu = p as u64;
            let d = ebp as u64;
            let mut ru = r as u64;
            let mut q = pu * cnt[k] as u64 / d;
            while q <= ru && k > 0 {
                ru -= q;
                k -= 1;
                q = pu * cnt[k] as u64 / d;
            }
            r = ru as u128;
            p = q as u128;
        } else {
            let mut q = mul_div(p, cnt[k], ebp);
            while q <= r && k > 0 {
                r -= q;
                k -= 1;
                q = mul_div(p, cnt[k], ebp);
            }
            p = q;
        }
        cnt[k] -= 1;
        cards[52 - ebp as usize] = 3 - k as u8;
    }
    let mut out = [0u8; 13];
    for (j, &s) in cards.iter().enumerate() {
        out[j >> 2] |= s << (2 * (j & 3));
    }
    out
}

/// Deals from a starting index onwards.
///
/// Created by [`Deals::from`], which winds the generator forward from the seed
/// boundary at or below `start` — so the first deal it yields is `start`, and
/// nothing before it is observable.
pub struct Deals {
    rng: Rng,
    next: u64,
}

impl Deals {
    /// Deals from `start` onwards.
    ///
    /// Catching up costs at most `SEED_GROUP - 1` deals, whatever `start` is,
    /// because the generator re-seeds on every group boundary.
    pub fn from(start: u64) -> Self {
        let mut rng = Rng::default();
        let boundary = start - start % SEED_GROUP;
        for index in boundary..start {
            deal(&mut rng, index);
        }
        Deals { rng, next: start }
    }

    /// The index of the deal the next call will produce.
    pub fn position(&self) -> u64 {
        self.next
    }
}

impl Iterator for Deals {
    type Item = [u8; DEAL_LEN];

    fn next(&mut self) -> Option<Self::Item> {
        let packed = deal(&mut self.rng, self.next);
        self.next += 1;
        Some(packed)
    }
}

/// One deal, when only one is wanted.
///
/// Pays the full catch-up to `index`'s seed boundary every time, so a loop
/// wants [`Deals`] instead.
pub fn deal_at(index: u64) -> [u8; DEAL_LEN] {
    Deals::from(index).next().expect("Deals never ends")
}
