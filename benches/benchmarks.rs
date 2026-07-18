use core::cell::RefCell;
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::ChaCha20Rng;
use rand::{Rng, SeedableRng};
use rust_map::ByteArrayTreeMap;

thread_local! {
    static THREAD_RNG: RefCell<ChaCha20Rng> = RefCell::new(ChaCha20Rng::from_seed(rand::random()));
}

pub fn get_random_bytes<const N: usize>() -> [u8;N]{
    let mut bytes = [0; N];
    THREAD_RNG.with_borrow_mut(|x| x.fill_bytes(&mut bytes));
    bytes
}

const fn get_new_map() -> ByteArrayTreeMap<32, u64> {ByteArrayTreeMap::new()}

fn benchmark_inserts(c: &mut Criterion) {

    // Generate data outside the timed loop so generation time isn't counted
    let mut keys = Vec::new();
    for _ in 0..100_000 {keys.push(get_random_bytes())}

    c.bench_function("map_inserts", |b| {
        b.iter(|| {
            // If your map persists across iterations, you can clone or reset it here.
            // For a clean slate every iteration:
            let mut map = get_new_map();

            for key in keys.iter() {
                // black_box ensures the compiler actually performs the insert
                black_box(map.insert(key, 0));
            }
        })
    });
}

fn benchmark_removes(c: &mut Criterion) {

    let mut keys = Vec::new();
    for _ in 0..100_000 {keys.push(get_random_bytes())}
    let mut map = get_new_map();
    for key in keys.iter() {map.insert(key, 0)}

    c.bench_function("map_removes", |b| {
        b.iter(|| {
            for key in keys.iter() {
                black_box(_ = map.remove(key));
            }
        })
    });
}

criterion_group!(benches, benchmark_inserts);
criterion_main!(benches);