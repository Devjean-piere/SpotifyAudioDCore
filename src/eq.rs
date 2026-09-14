use std::f32::consts::TAU;
use crate::structs::BandEqGains;

pub struct Biquad {
    b0: f32, b1: f32, b2: f32, a1: f32, a2: f32,
    x1: f32, x2: f32, y1: f32, y2: f32,
}

impl Biquad {
    pub fn new(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> Self {
        Self { b0, b1, b2, a1, a2, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    pub fn low_shelf(sample_rate: f32, freq: f32, gain_db: f32) -> Self {
        let mut filter = Self::new(0.0, 0.0, 0.0, 0.0, 0.0);
        filter.update_low_shelf(sample_rate, freq, gain_db);
        filter
    }

    pub fn high_shelf(sample_rate: f32, freq: f32, gain_db: f32) -> Self {
        let mut filter = Self::new(0.0, 0.0, 0.0, 0.0, 0.0);
        filter.update_high_shelf(sample_rate, freq, gain_db);
        filter
    }

    pub fn peaking(sample_rate: f32, freq: f32, gain_db: f32) -> Self {
        let mut filter = Self::new(0.0, 0.0, 0.0, 0.0, 0.0);
        filter.update_peaking(sample_rate, freq, gain_db);
        filter
    }

    pub fn update_low_shelf(&mut self, sample_rate: f32, freq: f32, gain_db: f32) {
        let q = 0.707;
        let a = 10f32.powf(gain_db / 40.0);
        let w = TAU * freq / sample_rate;
        let cos_w = w.cos();
        let alpha = w.sin() / (2.0 * q);
        let beta = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) + (a - 1.0) * cos_w + beta;
        self.b0 = (a * ((a + 1.0) - (a - 1.0) * cos_w + beta)) / a0;
        self.b1 = (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w)) / a0;
        self.b2 = (a * ((a + 1.0) - (a - 1.0) * cos_w - beta)) / a0;
        self.a1 = (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w)) / a0;
        self.a2 = ((a + 1.0) + (a - 1.0) * cos_w - beta) / a0;
    }

    pub fn update_high_shelf(&mut self, sample_rate: f32, freq: f32, gain_db: f32) {
        let q = 0.707;
        let a = 10f32.powf(gain_db / 40.0);
        let w = TAU * freq / sample_rate;
        let cos_w = w.cos();
        let alpha = w.sin() / (2.0 * q);
        let beta = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) - (a - 1.0) * cos_w + beta;
        self.b0 = (a * ((a + 1.0) + (a - 1.0) * cos_w + beta)) / a0;
        self.b1 = (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w)) / a0;
        self.b2 = (a * ((a + 1.0) + (a - 1.0) * cos_w - beta)) / a0;
        self.a1 = (2.0 * ((a - 1.0) - (a + 1.0) * cos_w)) / a0;
        self.a2 = ((a + 1.0) - (a - 1.0) * cos_w - beta) / a0;
    }

    pub fn update_peaking(&mut self, sample_rate: f32, freq: f32, gain_db: f32) {
        let q = 1.0;
        let a = 10f32.powf(gain_db / 40.0);
        let w = TAU * freq / sample_rate;
        let cos_w = w.cos();
        let alpha = w.sin() / (2.0 * q);
        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cos_w) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = out;
        out
    }
}

pub struct GraphicEq {
    sample_rate: f32,
    bands: Vec<(f32, Biquad)>,
}

impl GraphicEq {
    pub fn new(sample_rate: f32, frequencies: &[f32]) -> Self {
        let bands = frequencies
            .iter()
            .map(|&freq| (freq, Biquad::peaking(sample_rate, freq, 0.0)))
            .collect();
        Self { sample_rate, bands }
    }

    pub fn set_gains(&mut self, gains: &[f32]) {
        for ((freq, filter), &gain_db) in self.bands.iter_mut().zip(gains) {
            filter.update_peaking(self.sample_rate, *freq, gain_db);
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut sample = input;
        for (_, filter) in self.bands.iter_mut() {
            sample = filter.process(sample);
        }
        sample
    }
}

pub struct BandEq {
    sample_rate: f32,
    low_shelf: Biquad,
    mid_peak: Biquad,
    high_shelf: Biquad,
}

impl BandEq {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            low_shelf: Biquad::low_shelf(sample_rate, 250.0, 0.0),
            mid_peak: Biquad::peaking(sample_rate, 1000.0, 0.0),
            high_shelf: Biquad::high_shelf(sample_rate, 4000.0, 0.0),
        }
    }

    pub fn process(&mut self, input: &mut f32, gains: BandEqGains) {
        let low_db = 20.0 * gains.low.max(1e-4).log10();
        let mid_db = 20.0 * gains.mid.max(1e-4).log10();
        let high_db = 20.0 * gains.high.max(1e-4).log10();

        self.low_shelf.update_low_shelf(self.sample_rate, 250.0, low_db);
        self.mid_peak.update_peaking(self.sample_rate, 1000.0, mid_db);
        self.high_shelf.update_high_shelf(self.sample_rate, 4000.0, high_db);

        let mut sample = *input;
        sample = self.low_shelf.process(sample);
        sample = self.mid_peak.process(sample);
        sample = self.high_shelf.process(sample);
        *input = sample;
    }
}