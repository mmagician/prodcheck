use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_prodcheck::ml_prodcheck::{compute_f, compute_xy_vectors_from_x};
use ark_std::test_rng;
use ark_test_curves::bls12_381::{Bls12_381, Fr};
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = test_rng();
    let v = DenseMultilinearExtension::<Fr>::rand(10, &mut rng);
    let x = vec![0u8, 0u8, 0u8, 0u8, 0u8];

    c.bench_function("compute xy, l: 5", |b| {
        b.iter(|| compute_xy_vectors_from_x::<Bls12_381>(x.clone(), 5, &v))
    });
    c.bench_function("compute xy, l: 10", |b| {
        b.iter(|| compute_xy_vectors_from_x::<Bls12_381>(vec![], 10, &v))
    });
    c.bench_function("compute f", |b| b.iter(|| compute_f::<Bls12_381>(&v)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
