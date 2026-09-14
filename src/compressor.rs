pub struct Compressor {
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    sample_rate: f32,
    envelope: f32, // Zustand zwischen Samples, wie x1/y1 beim Biquad
    attack_coeff: f32,
    release_coeff: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            sample_rate,
            envelope: 0.0,
            attack_coeff: (-1.0 / ((10.0 / 1000.0) * sample_rate)).exp(),
            release_coeff: (-1.0 / ((100.0 / 1000.0) * sample_rate)).exp(),
        }

    }

    pub fn set_params(&mut self, threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32) {
        self.threshold_db = threshold_db;
        self.ratio = ratio;
        self.attack_ms = attack_ms;
        self.release_ms = release_ms;
        self.gen_coeff(attack_ms, release_ms);
    }

    fn gen_coeff(&mut self, attack_ms: f32, release_ms: f32) {
        self.attack_coeff =
            (-1.0 / ((attack_ms / 1000.0) * self.sample_rate)).exp();

        self.release_coeff =
            (-1.0 / ((release_ms / 1000.0) * self.sample_rate)).exp();
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // 1) Sample normalisieren (-1.0..1.0) und Pegel in dB umrechnen.
        //    Kleiner Schutz gegen log(0) = -unendlich bei absoluter Stille.
        let normalized = (input / i16::MAX as f32).abs().max(1e-6);
        let level_db = 20.0 * normalized.log10();

        // 2) + 3) Statische Kennlinie: Gain-Reduction in dB berechnen.
        let gain_reduction_db = if level_db > self.threshold_db {
            let over_threshold = level_db - self.threshold_db;
            let compressed = over_threshold / self.ratio;
            compressed - over_threshold // ist <= 0
        } else {
            0.0
        };


        // 5) Envelope glätten: Attack, wenn mehr Reduction gebraucht wird
        //    (gain_reduction_db ist negativer/kleiner als envelope),
        //    sonst Release.
        if gain_reduction_db < self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * gain_reduction_db;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * gain_reduction_db;
        }

        // 6) Aus dem geglätteten Envelope (nicht aus gain_reduction_db!)
        //    den linearen Gain-Faktor berechnen und aufs Sample anwenden.
        let gain = 10f32.powf(self.envelope / 20.0);
        input * gain
    }
}