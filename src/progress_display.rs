/*
Small helper sturct to display amount of processed data
*/
const GIBIBYTE: f64 = 1_073_741_824.0;

pub struct ProgressDisplay {
    progress_counter: u16,
    buffer_size_gib: f64,
}

impl ProgressDisplay {
    pub fn new(buffer_size_bytes: usize) -> ProgressDisplay {
        let buffer_size_gib = buffer_size_bytes as f64 / GIBIBYTE;

        ProgressDisplay {
            progress_counter: 0,
            buffer_size_gib,
        }
    }

    pub fn next(&mut self) -> f64 {
        let out = self.progress_counter as f64 * self.buffer_size_gib;
        self.progress_counter += 1;
        out
    }
}
