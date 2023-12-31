use bench_hash_2023::{crit_group, RustC, SeaHash};
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

use fnv::FnvBuildHasher;
use paste::paste;
use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash},
};

fn name_fn(name: &str, inner: &str, kt: &str, vt: &str) -> String {
    format!("HashMap<{},{},{}>::{}", kt, vt, name, inner)
}

macro_rules! bench_for {
    ($typ: ident, $k: expr, $v: expr) => {
        bench_for! {$typ, $typ, $k, $v}
    };
    ($kt: ident, $vt: ident, $k: expr, $v: expr) => {
        paste! {
            fn [<gen_bench_ $kt:snake _ $vt:snake>]<H>(name: &str, c: &mut Criterion)
            where
                H: BuildHasher + Default,
            {

                let mut group = crit_group(c, name);

                let new =  || {
                    HashMap::<$kt, $vt, H>::default()
                };
                let insert = |h: &mut HashMap<$kt, $vt, H>| {
                    h.insert($k, $v);
                };
                let remove = |h: &mut HashMap<$kt, $vt, H>| {
                    h.remove(&$k)
                };
                let get = |h: &HashMap<$kt, $vt, H>| {
                    h.get(&$k);
                };
                let kt = stringify!{$kt};
                let vt = stringify!{$vt};
                let name_fn = |inner: &str| name_fn(name, inner, kt, vt);
                group.bench_function(
                    &name_fn("insert[!exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            remove(&mut h);
                            h
                        },
                        |mut this| {
                            insert(black_box(&mut this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
                group.bench_function(
                    &name_fn("insert[exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            insert(&mut h);
                            h
                        },
                        |mut this| {
                            insert(black_box(&mut this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
                group.bench_function(
                    &name_fn("get[!exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            remove(&mut h);
                            h
                        },
                        |this| {
                            get(black_box(&this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
                group.bench_function(
                    &name_fn("get[exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            insert(&mut h);
                            h
                        },
                        |this| {
                            get(black_box(&this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
                group.bench_function(
                    &name_fn("remove[!exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            remove(&mut h);
                            h
                        },
                        |mut this| {
                            remove(black_box(&mut this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
                group.bench_function(
                    &name_fn("remove[exists]"),
                    |b| {
                    b.iter_batched_ref(
                        || {
                            let mut h = new();
                            insert(&mut h);
                            h
                        },
                        |mut this| {
                            remove(black_box(&mut this))
                        },
                        codspeed_criterion_compat::BatchSize::SmallInput,
                    )
                });
            }

            fn [<$kt:snake _ $vt:snake _bench_std_hashmap>](b: &mut Criterion) {
                [<gen_bench_ $kt:snake _ $vt:snake>]::<RandomState>("std::collections::hash_map::RandomState", b)
            }

            fn [<$kt:snake _ $vt:snake _bench_seahash_hashmap>](b: &mut Criterion) {
                [<gen_bench_ $kt:snake _ $vt:snake>]::<SeaHash>("seahash::SeaHasher", b)
            }

            fn [<$kt:snake _ $vt:snake _bench_ahash_hashmap>](b: &mut Criterion) {
                [<gen_bench_ $kt:snake _ $vt:snake>]::<ahash::RandomState>("ahash::RandomState", b)
            }

            fn [<$kt:snake _ $vt:snake _bench_fnv_hashmap>](b: &mut Criterion) {
                [<gen_bench_ $kt:snake _ $vt:snake>]::<FnvBuildHasher>("fnv::FnvBuildHasher", b)
            }

            fn [<$kt:snake _ $vt:snake _bench_rustc_hash_hashmap>](b: &mut Criterion) {
                [<gen_bench_ $kt:snake _ $vt:snake>]::<RustC>("rustc_hash::FxHasher", b)
            }

            criterion_group!(
                [<benches_ $kt:snake _ $vt:snake>],
                [<$kt:snake _ $vt:snake _bench_std_hashmap>],
                [<$kt:snake _ $vt:snake _bench_seahash_hashmap>],
                [<$kt:snake _ $vt:snake _bench_ahash_hashmap>],
                [<$kt:snake _ $vt:snake _bench_fnv_hashmap>],
                [<$kt:snake _ $vt:snake _bench_rustc_hash_hashmap>]
            );
        }
    };
}

macro_rules! bench_table {
    ( $({ $kt: ident, $vt: ident, $k: expr, $v: expr }), + $(,)?) => {
        $(
            bench_for!{
                $kt,
                $vt,
                $k,
                $v
            }
        )*

        paste::paste!{
            criterion_main!(
                $(
                [<benches_ $kt:snake _ $vt:snake>],
                )*
            );
        }

    };
}

type RawStr = &'static str;

#[derive(PartialEq, Eq, Clone)]
enum MyKeyValue {
    Key(String),
    Value(String),
}

impl Hash for MyKeyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Key(s) => state.write(s.as_bytes()),
            Self::Value(s) => state.write(s.as_bytes()),
        }
    }
    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for datum in data {
            datum.hash(state)
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum StaticAb {
    Abc,
    Def,
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash, Clone)]
enum StaticAbReprC {
    Abc,
    Def,
}

const COMPARABLE_KEY: &str = "k";
const COMPARABLE_SIZED: &str = "12345";

bench_table! {
    { RawStr, RawStr, COMPARABLE_KEY, COMPARABLE_SIZED},
    { RawStr, String, COMPARABLE_KEY, COMPARABLE_SIZED.to_string() },
    { RawStr, usize, COMPARABLE_KEY, 1 },
    { usize, usize, 1, 42 },
    { usize, RawStr, 1, COMPARABLE_SIZED },
    { usize, String, 1, COMPARABLE_SIZED.to_string() },
    { i32, i32, 1, 42 },
    { i32, usize, 1, 42 },
    { i32, RawStr, 1, COMPARABLE_SIZED },
    { String, String, COMPARABLE_KEY.to_string(), COMPARABLE_SIZED.to_string() },
    { String, RawStr, COMPARABLE_KEY.to_string(), COMPARABLE_SIZED },
    {
        MyKeyValue,
        MyKeyValue,
        MyKeyValue::Key(COMPARABLE_KEY.to_string()),
        MyKeyValue::Value(COMPARABLE_SIZED.to_string())
    },
    {
        MyKeyValue,
        RawStr,
        MyKeyValue::Key(COMPARABLE_KEY.to_string()),
        COMPARABLE_SIZED
    },
    {
        StaticAb,
        StaticAb,
        StaticAb::Abc,
        StaticAb::Def
    },
    {
        StaticAb,
        RawStr,
        StaticAb::Abc,
        COMPARABLE_SIZED
    },
    {
        StaticAb,
        String,
        StaticAb::Abc,
        COMPARABLE_SIZED.to_string()
    },
    {
        StaticAbReprC,
        StaticAbReprC,
        StaticAbReprC::Abc,
        StaticAbReprC::Def
    }
}
