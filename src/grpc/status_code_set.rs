use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::{fmt, iter, ops};

type BitSet = u32;

/// A set of [`tonic::Code`] which can be efficiently checked for the presence or absence of
/// certain code values.
///
/// ```
/// use ya_gcp::grpc::StatusCodeSet;
/// use tonic::Code;
///
/// const BAD_CODES: StatusCodeSet = StatusCodeSet::new(&[
///     Code::DataLoss,
///     Code::FailedPrecondition,
///     Code::Internal,
///     Code::Aborted,
/// ]);
///
/// assert!(BAD_CODES.contains(Code::DataLoss));
/// assert!(!BAD_CODES.contains(Code::Ok));
/// ```
// Implemented manually because common enumset crates require derives, and I can't do that on a
// foreign type :(
#[derive(
    Clone, Copy, Eq, PartialEq, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign,
)]
pub struct StatusCodeSet {
    set: BitSet,
}

impl StatusCodeSet {
    const BITS: usize = std::mem::size_of::<BitSet>() * 8;

    /// Create a new set with no elements
    pub const fn empty() -> Self {
        Self { set: 0 }
    }

    /// Create a new set containing the given status codes
    pub const fn new(codes: &[tonic::Code]) -> Self {
        let mut set = 0;

        // loop manually to stay const
        let mut i = 0;
        while i < codes.len() {
            set |= StatusCodeSet::code_to_bit(codes[i]);
            i += 1;
        }
        Self { set }
    }

    /// Check whether this set contains the given code
    pub const fn contains(&self, code: tonic::Code) -> bool {
        (self.set & StatusCodeSet::code_to_bit(code)) != 0
    }

    const fn code_to_bit(code: tonic::Code) -> BitSet {
        #[allow(clippy::manual_unwrap_or)] // must be manual because unwrap_or is not const
        match u32::checked_shl(1, code as u32) {
            Some(bit) => bit,
            None => {
                // At the time of writing, tonic defines 17 Code variants mapping to the 17 status
                // codes in the gRPC spec. This number could increase in the future, and tonic
                // marked the enum as non-exhaustive so it could increase without a major version
                // change. Increasing past 31 seems unlikely, but we need to handle that case
                // somehow. Ideally we'd panic, but until const panics are stable
                // (https://github.com/rust-lang/rust/issues/51999) that's not an option. The least
                // bad alternative is probably just not setting any bit for such variants;
                // `contains` will always return false
                0
            }
        }
    }

    /// Get an iterator over the codes contained in this set
    pub fn iter(&self) -> impl Iterator<Item = tonic::Code> {
        let this = *self;

        // create codes from all the possibly set bits in this set
        (0..(Self::BITS - (self.set.leading_zeros() as usize)) as i32)
            .into_iter()
            .map(|integer_code| {
                code_from_i32(integer_code).expect("set bits should all be valid codes")
            })
            // only return codes actually in this set
            .filter(move |code| this.contains(*code))
    }
}

/// Perform a checked equivalent to [`tonic::Code::from_i32`]
fn code_from_i32(integer_code: i32) -> Option<tonic::Code> {
    // tonic's `from_i32` documents returning `Unknown` for any value outside of its known range.
    // We can thus check if a given integer corresponds to a real code by comparing the returned
    // code against the canonical Unknown integer value
    use tonic::Code::Unknown;

    let code = tonic::Code::from_i32(integer_code);
    if code == Unknown && integer_code != Unknown as i32 {
        None
    } else {
        Some(code)
    }
}

impl fmt::Debug for StatusCodeSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_set().entries(self.iter()).finish()
        } else {
            fmt.debug_tuple("StatusCodeSet")
                .field(&format_args!("{:#b}", self.set))
                .finish()
        }
    }
}

impl Default for StatusCodeSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<tonic::Code> for StatusCodeSet {
    fn from(code: tonic::Code) -> Self {
        Self {
            set: StatusCodeSet::code_to_bit(code),
        }
    }
}

impl iter::FromIterator<tonic::Code> for StatusCodeSet {
    fn from_iter<I>(codes: I) -> Self
    where
        I: IntoIterator<Item = tonic::Code>,
    {
        codes
            .into_iter()
            .map(StatusCodeSet::from)
            .fold(StatusCodeSet::empty(), <StatusCodeSet as ops::BitOr>::bitor)
    }
}

impl<'de> serde::Deserialize<'de> for StatusCodeSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<i32>::deserialize(deserializer)?
            .into_iter()
            .map(|code| {
                code_from_i32(code).ok_or_else(|| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(i64::from(code)),
                        &"a known tonic::Code value",
                    )
                })
            })
            .collect::<Result<StatusCodeSet, _>>()
    }
}

impl crate::retry_policy::RetryPredicate<tonic::Status> for StatusCodeSet {
    fn is_retriable(&self, error: &tonic::Status) -> bool {
        self.contains(error.code())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for StatusCodeSet {
        fn arbitrary(gen: &mut Gen) -> Self {
            StatusCodeSet {
                set: BitSet::arbitrary(gen),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.set.shrink().map(|set| StatusCodeSet { set }))
        }
    }

    const KNOWN_VARIANTS: &[tonic::Code] = &[
        tonic::Code::Ok,
        tonic::Code::Cancelled,
        tonic::Code::Unknown,
        tonic::Code::InvalidArgument,
        tonic::Code::DeadlineExceeded,
        tonic::Code::NotFound,
        tonic::Code::AlreadyExists,
        tonic::Code::PermissionDenied,
        tonic::Code::ResourceExhausted,
        tonic::Code::FailedPrecondition,
        tonic::Code::Aborted,
        tonic::Code::OutOfRange,
        tonic::Code::Unimplemented,
        tonic::Code::Internal,
        tonic::Code::Unavailable,
        tonic::Code::DataLoss,
        tonic::Code::Unauthenticated,
    ];

    // wrap Code in order to implement quickcheck::Arbitrary
    #[derive(Debug, Copy, Clone)]
    struct ArbitraryCode(tonic::Code);

    impl Arbitrary for ArbitraryCode {
        fn arbitrary(gen: &mut Gen) -> Self {
            ArbitraryCode(
                *gen.choose(KNOWN_VARIANTS)
                    .expect("Gen guarantees non-None value for non-empty slice"),
            )
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                (0..(self.0 as i32))
                    .map(tonic::Code::from)
                    .map(ArbitraryCode),
            )
        }
    }

    fn dewrap(wrapped: Vec<ArbitraryCode>) -> Vec<tonic::Code> {
        wrapped.into_iter().map(|wrapped| wrapped.0).collect()
    }

    /// All codes inserted at construction should return true from `contains`
    #[quickcheck_macros::quickcheck]
    fn construct_contains(codes: Vec<ArbitraryCode>) {
        let codes = dewrap(codes);
        let set = StatusCodeSet::new(&codes);

        for code in codes {
            assert!(set.contains(code));
        }
    }

    /// Constructing from `new` and `from_iter` should result in identical sets
    #[quickcheck_macros::quickcheck]
    fn construct_from_iter(codes: Vec<ArbitraryCode>) {
        let codes = dewrap(codes);

        let from_iter = codes.iter().copied().collect::<StatusCodeSet>();
        let constructed = StatusCodeSet::new(&codes);

        assert_eq!(constructed, from_iter);
    }

    /// Constructing from `new` and deserializing (e.g. from json) should result in identical sets
    #[quickcheck_macros::quickcheck]
    fn deserialize(codes: Vec<ArbitraryCode>) {
        let codes = dewrap(codes);
        let json = serde_json::json! {
            codes
            .iter()
            .copied()
            .map(|code| code as i32)
            .collect::<Vec<i32>>()
        };
        assert_eq!(
            StatusCodeSet::new(&codes),
            serde_json::from_value(json).unwrap()
        );
    }
}
