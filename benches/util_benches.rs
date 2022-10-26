use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_prodcheck::ml_prodcheck::{compute_f, compute_xy_vectors_from_x};
use ark_std::test_rng;
use ark_test_curves::bls12_381::{Bls12_381, Fr};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_xy(c: &mut Criterion) {
    let mut rng = test_rng();
    let mut group = c.benchmark_group("compute xy");
    let x = vec![0u8, 0u8, 0u8, 0u8, 0u8];

    for m in [5usize, 10usize, 20usize, 25usize].iter() {
        let v = DenseMultilinearExtension::<Fr>::rand(*m, &mut rng);
        group.bench_with_input(BenchmarkId::from_parameter(m), m, |b, &m| {
            b.iter(|| compute_xy_vectors_from_x::<Bls12_381>(x.clone(), m - x.len(), &v));
        });
    }
    group.finish();
}

fn bench_f(c: &mut Criterion) {
    let mut rng = test_rng();
    let mut group = c.benchmark_group("compute f");

    for m in [5usize, 10usize, 15usize].iter() {
        let v = DenseMultilinearExtension::<Fr>::rand(*m, &mut rng);
        group.bench_with_input(BenchmarkId::from_parameter(m), m, |b, _| {
            b.iter(|| compute_f::<Bls12_381>(&v));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_xy, bench_f);
criterion_main!(benches);
