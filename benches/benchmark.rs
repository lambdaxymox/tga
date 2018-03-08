#[macro_use]
extern crate criterion;

extern crate tga;

mod sample;

use criterion::{Criterion, Fun};
use tga::TgaImage;
use std::fs::File;
use std::io::Read;


fn config() -> Criterion {
    Criterion::default().sample_size(20)
}

fn benchmark(c: &mut Criterion) {
    let control = Fun::new("Control", |b, filename| b.iter(|| {
        let mut file = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        buffer
    }));
    
    let parse_from_file = Fun::new("ParseFromFile", |b, filename| b.iter(|| {
        let mut file = File::open(filename).unwrap();
        TgaImage::parse_from_file(&mut file)
    }));

    let parse_from_buffer = Fun::new("ParseFromBuffer", |b, filename| b.iter(|| { 
        let mut file = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        TgaImage::parse_from_buffer(&buffer)
    }));

    let functions = vec![control, parse_from_file, parse_from_buffer];

    c.bench_functions("TGA Parser", functions, sample::LENA_TGA);
}

criterion_group!(name = benches; config = config(); targets = benchmark);
criterion_main!(benches);
