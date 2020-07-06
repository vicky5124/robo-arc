use std::intrinsics::log10f64;

fn min(x: f64, y: f64) -> f64 {
    x.min(y)
}

fn max(x: f64, y: f64) -> f64 {
    x.max(y)
}


#[derive(Debug, Default)]
pub struct PpCalculation {
    pub score_mods: Vec<String>,
    pub score_max_combo: f64,
    pub score_great: f64,
    pub score_good: f64,
    pub score_meh: f64,
    pub score_miss: f64,
    pub map_aim_strain: f64,
    pub map_speed_strain: f64,
    pub map_max_combo: f64,
    pub map_ar: f64,
    pub map_od: f64,
    pub map_circles: f64,
    pub map_sliders: f64,
    pub map_spinners: f64,
    pub progress: f64,

    map_hit_count: f64,
    total_hits: f64,
    total_successful_hits: f64,
    miss_penality: f64,
    combo_break: f64,
    speed_mul: f64,
    real_acc: f64,
    score_accuracy: f64,
    length_bonus: f64,
    ar_bonus: f64,
    hd_bonus: f64,
}

impl PpCalculation {
    pub fn calculate(&mut self) -> f64 {
        self.map_hit_count = self.map_circles + self.map_sliders + self.map_spinners;
        self.total_hits = self.score_great + self.score_good + self.score_meh + self.score_miss;
        self.total_successful_hits = self.score_great + self.score_good + self.score_meh;

        let objects_over_2k = self.map_hit_count / 2000.0;

        self.length_bonus = 0.95 + 0.4 * min(1.0, objects_over_2k);

        if self.map_hit_count > 2000.0 {
            self.length_bonus += unsafe { log10f64(self.total_hits / 2000.0) * 0.5 };
        }

        self.miss_penality = 0.97_f64.powf(self.score_miss);
        self.combo_break = self.score_max_combo.powf(0.8) / self.map_max_combo.powf(0.8);

        self.speed_mul = self.apply_mods();

        self.ar_bonus = 1.0;

        if self.map_ar > 10.33 {
            self.ar_bonus += 0.3 * (self.map_ar - 10.33);
        } else if self.map_ar < 8.0 {
            self.ar_bonus += 0.01 * (8.0 - self.map_ar)
        }

        self.hd_bonus = 1.0;

        if self.score_mods.contains(&"HD".to_string()) {
            self.hd_bonus *= 1.0 + 0.04 * (12.0 - self.map_ar);
        }

        self.compute_real_accuracy();
    
        let aim_value = self.compute_aim_value();
        let speed_value = self.compute_speed_value();
        let accuracy_value = self.compute_accuracy_value();

        let mut multiplier = 1.12_f64;
    
        if self.score_mods.contains(&"NF".to_string()) {
            multiplier *= 0.90;
        }
        if self.score_mods.contains(&"SO".to_string()) {
            multiplier *= 0.95;
        }

        let total_value = ((aim_value.powf(1.1) + speed_value.powf(1.1) + accuracy_value.powf(1.1)).powf(1.0 / 1.1)) * multiplier;

        total_value
    }

    pub fn compute_aim_value(&self) -> f64 {
        let mut aim_value = (5.0 * max(1.0, self.map_aim_strain / 0.0675) - 4.0).powf(3.0) / 100000.0;

        aim_value *= self.length_bonus;
        aim_value *= self.miss_penality;
        aim_value *= self.combo_break;
        aim_value *= self.ar_bonus;
        aim_value *= self.hd_bonus;

        if self.score_mods.contains(&"FL".to_string()) {
            aim_value *= 1.0 + 0.35 * min(1.0, self.total_hits / 200.0) + (if self.total_hits > 200.0 { 0.3 * min(1.0, (self.total_hits - 200.0) / 300.0) + (if self.total_hits > 50.00 { (self.total_hits - 500.0) / 1200.0 } else { 0.0 }) } else { 0.0 });
        }

        aim_value *= 0.5 + self.score_accuracy / 2.0;
        aim_value *= 0.98 + self.map_od.powf(2.0) / 2500.0;

        aim_value
    }

    pub fn compute_speed_value(&self) -> f64 {
        let mut speed_value = (5.0 * max(1.0, self.map_speed_strain / 0.0675) - 4.0).powf(3.0) / 100000.0;

        speed_value *= self.length_bonus;
        speed_value *= self.miss_penality;
        speed_value *= self.combo_break;

        if self.map_ar > 10.33 {
            speed_value *= self.ar_bonus;
        }

        speed_value *= self.hd_bonus;
        speed_value *= 0.02 + self.score_accuracy;
        speed_value *= 0.96 + self.map_od.powf(2.0) / 1600.0;

        speed_value
    }

    pub fn compute_accuracy_value(&self) -> f64 {
        let mut acc_value = 1.52163_f64.powf(self.map_od) * self.real_acc.powf(24.0) * 2.83;

        acc_value *= min(1.15, (self.map_circles / 1000.0).powf(0.3));

        if self.score_mods.contains(&"HD".to_string()) {
            acc_value *= 1.08;
        }

        if self.score_mods.contains(&"FL".to_string()) {
            acc_value *= 1.02;
        }

        acc_value
    }

    pub fn compute_real_accuracy(&mut self) {
        self.score_accuracy = self.acc_math(self.score_great, self.score_good, self.score_meh, self.score_miss);
        self.real_acc = self.score_accuracy;

        if self.score_mods.contains(&"V2".to_string()) {
            self.map_circles = self.map_hit_count;
        } else {
            self.real_acc = self.acc_math(
                self.score_great - (self.map_sliders * self.progress / 100.0) - (self.map_spinners * self.progress / 100.0),
                self.score_good, self.score_meh, self.score_miss
            );

            self.real_acc = max(0.0, self.real_acc);
        }
    }

    pub fn apply_mods(&mut self) -> f64 {
        20.0
    }

    pub fn acc_math(&self, score_great: f64, score_good: f64, score_meh: f64, score_miss: f64) -> f64 {
        let h = score_great + score_good + score_meh + score_miss;
        (score_meh * 50.0 + score_good * 100.0 + score_great * 300.0) / (h * 300.0)
    }
}

// This is a map to convert the bitwhise number obtained from the api
// To the mods it represents.
// With the short and long versions of the mod names.
//
// This is a module so it can make the compiler not complain about the naming of the constants.
pub mod bitwhise_mods {
    #![allow(non_upper_case_globals)]
    use bitflags::bitflags;
    
    bitflags! {
        pub struct LongMods: u32 {
            const None           = 0;
            const NoFail         = 1;
            const Easy           = 2;
            const TouchDevice    = 4;
            const Hidden         = 8;
            const HardRock       = 16;
            const SuddenDeath    = 32;
            const DoubleTime     = 64;
            const Relax          = 128;
            const HalfTime       = 256;
            const Nightcore      = 512;
            const Flashlight     = 1024;
            const Autoplay       = 2048;
            const SpunOut        = 4096;
            const Relax2         = 8192;    // Autopilot
            const Perfect        = 16384;
            const Key4           = 32768;
            const Key5           = 65536;
            const Key6           = 131_072;
            const Key7           = 262_144;
            const Key8           = 524_288;
            const FadeIn         = 1_048_576;
            const Random         = 2_097_152;
            const Cinema         = 4_194_304;
            const Target         = 8_388_608;
            const Key9           = 16_777_216;
            const KeyCoop        = 33_554_432;
            const Key1           = 67_108_864;
            const Key3           = 134_217_728;
            const Key2           = 268_435_456;
            const ScoreV2        = 536_870_912;
            const Mirror         = 1_073_741_824;
        }
    }
    bitflags! {
        pub struct ShortMods: u32 {
            const NM = 0;
            const NF = 1;
            const EZ = 2;
            const TD = 4;
            const HD = 8;
            const HR = 16;
            const SD = 32;
            const DT = 64;
            const RX = 128;
            const HT = 256;
            const NC = 512;
            const FL = 1024;
            const AT = 2048;
            const SO = 4096;
            const AP = 8192;
            const PF = 16384;
            const K4 = 32768;
            const K5 = 65536;
            const K6 = 131_072;
            const K7 = 262_144;
            const K8 = 524_288;
            const FI = 1_048_576;
            const RD = 2_097_152;
            const CN = 4_194_304;
            const TP = 8_388_608;
            const K9 = 16_777_216;
            const CO = 33_554_432;
            const K1 = 67_108_864;
            const K3 = 134_217_728;
            const K2 = 268_435_456;
            const V2 = 536_870_912;
            const MR = 1_073_741_824;
        }
    }
}

