
extern crate apodize;
extern crate fftw;
extern crate num_complex;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use apodize::hanning_iter;

#[derive(Clone)]
pub struct ChunksWithHop<'a, T: 'a> {
    v: &'a [T],
    size: usize,
    hop: isize,
}

impl<'a, T> Iterator for ChunksWithHop<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {
        if self.v.is_empty() {
            return None;
        }
        let this_chunk_size = std::cmp::min(self.v.len(), self.size);
        let result = Some(&self.v[..this_chunk_size]);
        self.v = &self.v[(this_chunk_size as isize + self.hop) as usize..];
        result
    }
}

fn hanning_window_frame(frame_size: usize, in_frame: &[u8], out_frame: &mut [u8]) {
    for (i, window_sample) in hanning_iter(frame_size).enumerate() {
        out_frame[i] = if i >= in_frame.len() {
            0
        } else {
            ((in_frame[i] as f64) * window_sample) as u8
        }
    }
}

pub fn stft(signal: &Vec<f32>, samples_per_second: usize, window_size: usize, hop: isize) -> SpectrogramResult {
    let window: Vec<f32> = hanning_iter(window_size).map(|c| c as f32).collect();
    let mut pair = fftw::Pair::c2c_1d(window_size,
                                      fftw::SIGN::FFTW_FORWARD,
                                      fftw::FLAG::FFTW_ESTIMATE);
    let mut out_spectogram = Vec::<Vec<f32>>::new();
    let mut chunks_iter = ChunksWithHop {
        v: signal,
        size: window_size,
        hop: hop,
    };
    let mut min_val = std::f32::INFINITY;
    let mut max_val = std::f32::NEG_INFINITY;
    for (i, chunk) in chunks_iter.enumerate() {
        for i in 0..window_size {
            pair.field[i] = num_complex::Complex::<f32>::new(if chunk.len() > i {
                                                                 chunk[i] * window[i]
                                                             } else {
                                                                 0.0
                                                             },
                                                             0.0);
        }
        pair.forward();

        let out_buf: Vec<f32> = pair.coef
            .iter()
            .take(pair.coef.len() / 2 + 1)
            .map(|c| c.norm().log10())
            .map(|val| {
                let positive_val = if val < 0.0 { 0.0 } else { val };
                if positive_val > max_val {
                    max_val = positive_val;
                }
                if positive_val < min_val {
                    min_val = positive_val;
                }
                positive_val
            })
            .collect();
        out_spectogram.push(out_buf);
    }
    SpectrogramResult{
        v: out_spectogram,
        x_axis_bounds_samples: [0, signal.len()],
        y_axis_bounds_hz: [0, samples_per_second],
        magnitude_bounds: [min_val, max_val],
    }
}

pub struct SpectrogramResult {
    pub v: Vec<Vec<f32>>,
    pub x_axis_bounds_samples: [usize; 2],
    pub y_axis_bounds_hz: [usize; 2],
    pub magnitude_bounds: [f32; 2],
}
