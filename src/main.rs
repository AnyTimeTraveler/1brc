use std::fs::File;
use std::mem::transmute;

use halfbrown::HashMap;
use memmap::MmapOptions;
use tokio;

const NUM_CHUNKS: usize = 64;

#[tokio::main(flavor = "multi_thread", worker_threads = 24)]
async fn main() {
    let file = File::open("measurements.big.csv").unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let data: &[u8] = &*mmap;
    let data: &'static [u8] = unsafe { transmute(data) };

    let chunks = make_chunks(data);

    let threads: Vec<_> = chunks.into_iter()
        .map(|(start, end)| tokio::spawn(process_chunk(data, start, end)))
        .collect();

    let mut total = HashMap::new();

    for handle in threads {
        let chunk = handle.await.unwrap();
        for (name, (count, sum, min, max)) in chunk {
            if let Some((t_count, t_sum, t_min, t_max)) = total.get_mut(&name) {
                *t_count += count;
                *t_sum += sum;
                *t_min = f32::min(*t_min, min);
                *t_max = f32::max(*t_max, max);
            } else {
                total.insert(name, (count, sum, min, max));
            }
        }
    }

    for (name, (count, sum, min, max)) in total {
        println!(r#"{{ "station": "{}", "min": {}, "avg": {}, "max": {} }}"#, String::from_utf8_lossy(&name), min, sum / count as f32, max);
    }
}

async fn process_chunk(data: &[u8], start: usize, end: usize) -> HashMap<Vec<u8>, (u32, f32, f32, f32)> {
    let mut values: HashMap<Vec<u8>, (u32, f32, f32, f32)> = HashMap::new();

    let data = &data[start..end];

    let mut semi_loc;
    let mut newline_loc;
    let mut line_start_loc;
    let mut i = 0;
    loop {
        line_start_loc = i;
        while data[i] != b';' {
            i += 1;
        }
        semi_loc = i;
        i += 1;
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        newline_loc = i;
        i += 1;
        let name = &data[line_start_loc..semi_loc];
        let value = &data[semi_loc + 1..newline_loc];
        let value: f32 = fast_float::parse(&value).unwrap();
        if let Some((count, sum, min, max)) = values.get_mut(name) {
            *count += 1;
            *sum += value;
            if value < *min {
                *min = value;
            } else if value > *max {
                *max = value;
            }
        } else {
            values.insert(name.to_vec(), (1, value, value, value));
        }
        if i >= data.len() {
            return values;
        }
    }
}

fn make_chunks(data: &[u8]) -> [(usize, usize); NUM_CHUNKS] {
    let mut chunks = [(0usize, 0usize); NUM_CHUNKS];

    let chunk_size = data.len() / NUM_CHUNKS;

    let mut current_chunk_start_ptr = 0;
    let mut current_data_ptr = 0;
    for i in 0..NUM_CHUNKS {
        current_data_ptr += chunk_size;
        while current_data_ptr < data.len() && data[current_data_ptr] != b'\n' {
            current_data_ptr += 1;
        }
        chunks[i] = (current_chunk_start_ptr, current_data_ptr);
        current_data_ptr += 1;
        current_chunk_start_ptr = current_data_ptr;
    }

    chunks[NUM_CHUNKS - 1].1 = data.len();

    chunks
}
