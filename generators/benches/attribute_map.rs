//! Benchmark whether a HashMap or BTreeMap is faster for `attribute` fields in protobuf
//! definitions.
//!
//! This context includes certain constraints:
//!
//! * The HashMap set by prost's generator uses the default hasher from the std lib
//! * The BTreeMap set by prost's generator uses the default parameters from the std lib
//! * Attribute maps tend to be small. For example, PubsubMessage.attribute is limited to a maximum
//!   of 100 keys and 256 bytes per key
//!
//! Results:
//! The BTreeMap tends to be faster for all operations on longer keys (30 to 100 chars) and on
//! smaller sets (5 to 20 keys) than the HashMap. The HashMap surpasses it typically for sets of
//! many short keys (100 keys of 4 to 30 chars).
//!
//! Conclusion:
//! Anticipated workloads would not generally use upwards of 100 attributes. The key sizes may vary
//! (though also unlikely to reach ~256 chars). Given such a distribution, the BTreeMap would
//! generally be more performant

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::{
    distributions::{Alphanumeric, Distribution},
    thread_rng as rng,
    Rng,
};

macro_rules! make_fns {
    ($($type_name:ident),+) => {
        $(
            // create a module for each map type, and put all the functionality within that module.
            // Then this macro will build benchmarks for each module

            #[allow(non_snake_case)]
            mod $type_name {
                pub(crate) use std::collections::$type_name as Map;

                pub(crate) const IMPL: &'static str = stringify!($type_name);

                pub(crate) fn init(
                    input: impl IntoIterator<Item = (String, String)>,
                ) -> Map<String, String> {
                    input.into_iter().collect()
                }
            }
        )+

        const MAX_KEYS: usize = 100;
        const MAX_LEN: usize = 256;

        pub(crate) fn bench(c: &mut Criterion) {
            // Actual keys won't be uniformly random alphanums (more likely human readable
            // snake_case) so this is not exactly a reflection of real-world performance, but it
            // should be at least ballpark-informative
            let random_letters = &mut Alphanumeric.sample_iter(rng()).map(char::from);

            // a set of random keys that are relatively short, maybe a few words worth.
            let short = &std::iter::repeat_with(|| {
                random_letters
                    .take(rng().gen_range(4..30))
                    .collect::<String>()
            })
            .take(MAX_KEYS)
            .collect::<Vec<_>>();

            // a set of random keys that are relatively long, up to the rpc max.
            let long = &std::iter::repeat_with(|| {
                random_letters
                    .take(rng().gen_range(30..MAX_LEN))
                    .collect::<String>()
            })
            .take(MAX_KEYS)
            .collect::<Vec<_>>();

            // a set of random keys of all lengths
            let mixed = &std::iter::repeat_with(|| {
                random_letters
                    .take(rng().gen_range(4..MAX_LEN))
                    .collect::<String>()
            })
            .take(MAX_KEYS)
            .collect::<Vec<_>>();

            // a set of keys used by a real application
            let hedwig_keys = &[
                String::from("hedwig_id"),
                String::from("hedwig_publisher"),
                String::from("hedwig_message_timestamp"),
                String::from("hedwig_format_version"),
                String::from("hedwig_schema")
            ];

            let test_runs = &[
                (&[][..], "empty"),
                (&short[0..5], "short few"),
                (&short[0..20], "short some"),
                (&short[0..MAX_KEYS], "short many"),
                (&long[0..5], "long few"),
                (&long[0..20], "long some"),
                (&long[0..MAX_KEYS], "long many"),
                (&mixed[0..5], "mixed few"),
                (&mixed[0..20], "mixed some"),
                (&mixed[0..MAX_KEYS], "mixed many"),
                (hedwig_keys, "hedwig")
            ];

            for (keys, description) in test_runs {
                let mut group = c.benchmark_group(format!("Init {}", description));

                $({
                    group.bench_function($type_name::IMPL, |b| {
                        b.iter_batched(
                            || keys.iter().map(|s| (s.clone(), String::new())).collect::<Vec<_>>(),
                            $type_name::init,
                            BatchSize::SmallInput
                        )
                    });
                })+

                group.finish();

                let mut group = c.benchmark_group(format!("Get {}", description));

                $({
                    let map = keys.iter().map(|s| (s.clone(), String::new())).collect::<$type_name::Map<_, _>>();
                    group.bench_function($type_name::IMPL, |b| {
                        b.iter(|| {
                            for k in keys.iter() {
                                black_box(map.get(k));
                            }
                        })
                    });
                })+

                group.finish();

                let mut group = c.benchmark_group(format!("Remove {}", description));

                $({
                    group.bench_function($type_name::IMPL, |b| {
                        b.iter_batched(
                            || keys.iter().map(|s| (s.clone(), String::new())).collect::<$type_name::Map<_, _>>(),
                            |mut map| {
                                for k in keys.iter() {
                                    black_box(map.remove(k));
                                }
                            },
                            BatchSize::SmallInput
                        )
                    });
                })+
            }
        }
    };
}

make_fns!(BTreeMap, HashMap);

criterion_group!(benches, bench);
criterion_main!(benches);
