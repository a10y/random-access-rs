use divan::Bencher;
use fastlanes::BitPacking;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

trait Vector<T> {
    /// Perform a random access to a specific element in the vector.
    fn access(&self, index: usize) -> T;
}

struct SnappyVector<T> {
    bytes: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T: Copy> SnappyVector<T> {
    fn new(values: Vec<T>) -> Self {
        let output: Vec<u8> = Vec::with_capacity(values.len());
        let mut compressed = snap::write::FrameEncoder::new(output);
        // Take the T and turn it into a set of bytes instead.
        let byte_array = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                values.as_ptr() as *mut u8,
                values.len() * size_of::<T>(),
                values.capacity() * size_of::<T>(),
            ))
        };
        compressed.write(byte_array.as_slice()).unwrap();
        compressed.flush().unwrap();

        Self {
            bytes: compressed.into_inner().unwrap(),
            _phantom: PhantomData,
        }
    }

    fn decompress_at(&self, index: usize) -> T {
        let mut decompressed = snap::read::FrameDecoder::new(self.bytes.as_slice());
        let mut output: Vec<u8> = Vec::new();
        decompressed.read_to_end(&mut output).unwrap();
        let byte_array = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                output.as_ptr() as *mut T,
                output.len() / size_of::<T>(),
                output.capacity() / size_of::<T>(),
            ))
        };
        byte_array[index]
    }
}

#[divan::bench(sample_count = 500)]
fn snappy(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let values: Vec<u32> = (0..1024).cycle().take(65_536).collect();
            SnappyVector::new(values)
        })
        .bench_values(|snappy| {
            let value = snappy.decompress_at(500);
            assert_eq!(value, 500);
            value
        })
}

// Now let's try with lightweight compression: BitPacking.
#[divan::bench(sample_count = 1_000)]
fn bitpacking(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let values: Vec<u32> = (0..1024).cycle().take(65_536).collect();

            // Pack from 32 -> 10 bits per value. This will take every 1024 u32's and turn them into 1024/32 * 10 = 320 values.
            let mut compressed: Vec<u32> = Vec::new();

            for chunk in 0..64 {
                let mut packed = [032; 320];
                let unpacked = &values[chunk * 1024..(chunk + 1) * 1024];
                // SAFETY: lol
                unsafe { BitPacking::unchecked_pack(10, unpacked, &mut packed) };
                compressed.extend_from_slice(&packed);
            }

            compressed
        })
        .bench_values(|compressed| {
            let value = fastlanes_random_access(&compressed, 500);
            // make sure we get the right thing back
            assert_eq!(value, 500);
            value
        })
}

fn fastlanes_random_access(packed: &[u32], index: usize) -> u32 {
    let chunk = index / 1024;
    let offset = index % 1024;
    let packed_chunk: &[u32] = &packed[chunk * 320..(chunk + 1) * 320];
    let mut unpacked = [0; 1024];
    // SAFETY: myguy chill out this is a test case
    unsafe { BitPacking::unchecked_unpack_single(10, &packed_chunk, offset) }
}

fn main() {
    divan::main();
}
