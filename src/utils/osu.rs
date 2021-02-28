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

        self.apply_mods();

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

        ((aim_value.powf(1.1) + speed_value.powf(1.1) + accuracy_value.powf(1.1)).powf(1.0 / 1.1))
            * multiplier
    }

    pub fn compute_aim_value(&self) -> f64 {
        let mut aim_value =
            (5.0 * max(1.0, self.map_aim_strain / 0.0675) - 4.0).powf(3.0) / 100000.0;

        aim_value *= self.length_bonus;
        aim_value *= self.miss_penality;
        aim_value *= self.combo_break;
        aim_value *= self.ar_bonus;
        aim_value *= self.hd_bonus;

        if self.score_mods.contains(&"FL".to_string()) {
            aim_value *= 1.0
                + 0.35 * min(1.0, self.total_hits / 200.0)
                + (if self.total_hits > 200.0 {
                    0.3 * min(1.0, (self.total_hits - 200.0) / 300.0)
                        + (if self.total_hits > 50.00 {
                            (self.total_hits - 500.0) / 1200.0
                        } else {
                            0.0
                        })
                } else {
                    0.0
                });
        }

        aim_value *= 0.5 + self.score_accuracy / 2.0;
        aim_value *= 0.98 + self.map_od.powf(2.0) / 2500.0;

        aim_value
    }

    pub fn compute_speed_value(&self) -> f64 {
        let mut speed_value =
            (5.0 * max(1.0, self.map_speed_strain / 0.0675) - 4.0).powf(3.0) / 100000.0;

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
        self.score_accuracy = self.acc_math(
            self.score_great,
            self.score_good,
            self.score_meh,
            self.score_miss,
        );
        self.real_acc = self.score_accuracy;

        if self.score_mods.contains(&"V2".to_string()) {
            self.map_circles = self.map_hit_count;
        } else {
            self.real_acc = self.acc_math(
                self.score_great
                    - (self.map_sliders * self.progress / 100.0)
                    - (self.map_spinners * self.progress / 100.0),
                self.score_good,
                self.score_meh,
                self.score_miss,
            );

            self.real_acc = max(0.0, self.real_acc);
        }
    }

    pub fn apply_mods(&mut self) {
        const OD0_MS: f64 = 80.0;
        const OD10_MS: f64 = 20.0;
        const AR0_MS: f64 = 1800.0;
        const AR5_MS: f64 = 1200.0;
        const AR10_MS: f64 = 450.0;

        const OD_MS_STEP: f64 = (OD0_MS - OD10_MS) / 10.0;
        const AR_MS_STEP1: f64 = (AR0_MS - AR5_MS) / 5.0;
        const AR_MS_STEP2: f64 = (AR5_MS - AR10_MS) / 5.0;

        self.speed_mul = 1.0;

        if self.score_mods.contains(&"DT".to_string()) {
            self.speed_mul = 1.5;
        }
        if self.score_mods.contains(&"HT".to_string()) {
            self.speed_mul *= 0.75;
        }
        let mut od_ar_hp_multiplier = 1.0_f64;

        if self.score_mods.contains(&"HR".to_string()) {
            od_ar_hp_multiplier = 1.4;
        }
        if self.score_mods.contains(&"EZ".to_string()) {
            od_ar_hp_multiplier *= 0.5;
        }

        self.map_ar *= od_ar_hp_multiplier;

        let mut arms = if self.map_ar < 5.0 {
            AR0_MS - AR_MS_STEP1 * self.map_ar
        } else {
            AR5_MS - AR_MS_STEP2 * (self.map_ar - 5.0)
        };

        arms = min(AR0_MS, max(AR10_MS, arms));
        arms /= self.speed_mul;

        if arms > AR5_MS {
            self.map_ar = (AR0_MS - arms) / AR_MS_STEP1
        } else {
            self.map_ar = 5.0 + (AR5_MS - arms) / AR_MS_STEP2
        }

        self.map_od *= od_ar_hp_multiplier;

        let mut odms = OD0_MS - (OD_MS_STEP * self.map_od).ceil();
        odms = min(OD0_MS, max(OD10_MS, odms));
        odms /= self.speed_mul;

        self.map_od = (OD0_MS - odms) / OD_MS_STEP;
    }

    pub fn acc_math(
        &self,
        score_great: f64,
        score_good: f64,
        score_meh: f64,
        score_miss: f64,
    ) -> f64 {
        let h = score_great + score_good + score_meh + score_miss;
        (score_meh * 50.0 + score_good * 100.0 + score_great * 300.0) / (h * 300.0)
    }

    pub fn _test() {
        let mut pp = Self {
            score_mods: vec![],
            score_max_combo: 863.0,
            score_great: 556.0,
            score_good: 63.0,
            score_meh: 2.0,
            score_miss: 0.0,
            map_aim_strain: 2.54215,
            map_speed_strain: 2.83749,
            map_max_combo: 865.0,
            map_ar: 9.0,
            map_od: 8.0,
            map_circles: 456.0,
            map_sliders: 164.0,
            map_spinners: 1.0,
            ..Default::default()
        };

        let ppnum = pp.calculate();
        println!("DTN {}", ppnum);

        pp.score_mods = vec!["DT".to_string()];

        let ppnum = pp.calculate();
        println!("DT1 {}", ppnum);

        pp.map_aim_strain = 3.62648;
        pp.map_speed_strain = 4.48349;

        let ppnum = pp.calculate();
        println!("DT2 {}", ppnum);
    }
}
/*
   class diff_calc:
   """
   difficulty calculator.
   fields:
   total: star rating
   aim: aim stars
   speed: speed stars
   nsingles: number of notes that are considered singletaps by
   the difficulty calculator
   nsingles_threshold: number of taps slower or equal to the
   singletap threshold value
   """

   def __init__(self):
   self.strains = []
   # NOTE: i tried pre-allocating this to 600 elements or so
   # and it didn't show appreciable performance improvements

   self.reset()


   def reset(self):
   self.total = 0.0
   self.aim = self.aim_difficulty = self.aim_length_bonus = 0.0
   self.speed = self.speed_difficulty = self.speed_length_bonus = 0.0
   self.nsingles = self.nsingles_threshold = 0


   def __str__(self):
   return """%g stars (%g aim, %g speed)
   %d spacing singletaps
   %d taps within singletap threshold""" % (
   self.total, self.aim, self.speed, self.nsingles,
   self.nsingles_threshold
   )


   def calc_individual(self, difftype, bmap, speed_mul):
   # calculates total strain for difftype. this assumes the
   # normalized positions for hitobjects are already present

   # max strains are weighted from highest to lowest.
   # this is how much the weight decays
   DECAY_WEIGHT = 0.9

   # strains are calculated by analyzing the map in chunks
   # and taking the peak strains in each chunk. this is the
   # length of a strain interval in milliseconds
   strain_step = 400.0 * speed_mul

   objs = bmap.hitobjects
   self.strains[:] = []
   # first object doesn't generate a strain so we begin with
   # an incremented interval end
   interval_end = (
   math.ceil(objs[0].time / strain_step) * strain_step
   )
   max_strain = 0.0

   t = difftype

   for i, obj in enumerate(objs[1:]):
   prev = objs[i]

   d_strain(difftype, obj, prev, speed_mul)

   while obj.time > interval_end:
   # add max strain for this interval
   self.strains.append(max_strain)

# decay last object's strains until the next
    # interval and use that as the initial max strain
decay = pow(
    DECAY_BASE[t],
    (interval_end - prev.time) / 1000.0
)

    max_strain = prev.strains[t] * decay
    interval_end += strain_step


max_strain = max(max_strain, obj.strains[t])


    # don't forget to add the last strain
self.strains.append(max_strain)

    # weight the top strains sorted from highest to lowest
    weight = 1.0
    total = 0.0
    difficulty = 0.0

    strains = self.strains
strains.sort(reverse=True)

    for strain in strains:
total += pow(strain, 1.2)
    difficulty += strain * weight
    weight *= DECAY_WEIGHT


return ( difficulty, total )


    def calc(self, bmap, mods=MODS_NOMOD, singletap_threshold=125):
        """
            calculates difficulty and stores results in self.total,
            self.aim, self.speed, self.nsingles,
            self.nsingles_threshold.
            returns self.
            singletap_threshold is the smallest milliseconds interval
    that will be considered singletappable, defaults to 125ms
            which is 240 bpm 1/2 ((60000 / 240) / 2)
            """

            # non-normalized diameter where the small circle size buff
            # starts
            CIRCLESIZE_BUFF_THRESHOLD = 30.0
            STAR_SCALING_FACTOR = 0.0675 # global stars multiplier

            # 50% of the difference between aim and speed is added to
            # star rating to compensate aim only or speed only maps
            EXTREME_SCALING_FACTOR = 0.5

    PLAYFIELD_WIDTH = 512.0 # in osu!pixels
playfield_center = v2f(
    PLAYFIELD_WIDTH / 2, PLAYFIELD_WIDTH / 2
)

    if bmap.mode != MODE_STD:
    raise NotImplementedError

self.reset()

    # calculate CS with mods
speed_mul, _, _, cs, _ = mods_apply(mods, cs=bmap.cs)

    # circle radius
    radius = (
        (PLAYFIELD_WIDTH / 16.0) *
        (1.0 - 0.7 * (cs - 5.0) / 5.0)
    )

    # positions are normalized on circle radius so that we can
    # calc as if everything was the same circlesize
    scaling_factor = 52.0 / radius

    # low cs buff (credits to osuElements)
    if radius < CIRCLESIZE_BUFF_THRESHOLD:
    scaling_factor *= (
        1.0 + min(CIRCLESIZE_BUFF_THRESHOLD - radius, 5.0) / 50.0
    )


    playfield_center *= scaling_factor

    # calculate normalized positions
    objs = bmap.hitobjects
    prev1 = None
    prev2 = None
    i = 0
    for obj in objs:
    if obj.objtype & OBJ_SPINNER != 0:
obj.normpos = v2f(
    playfield_center.x, playfield_center.y
)
    else:
    obj.normpos = obj.data.pos * scaling_factor

    if i >= 2:
    v1 = prev2.normpos - prev1.normpos
    v2 = obj.normpos - prev1.normpos
dot = v1.dot(v2)
    det = v1.x * v2.y - v1.y * v2.x
obj.angle = abs(math.atan2(det, dot))
    else:
    obj.angle = None

    prev2 = prev1
    prev1 = obj
    i+=1

    b = bmap

    # speed and aim stars
speed = self.calc_individual(DIFF_SPEED, b, speed_mul)
    self.speed = speed[0]
    self.speed_difficulty = speed[1]

aim = self.calc_individual(DIFF_AIM, b, speed_mul)
    self.aim = aim[0]
    self.aim_difficulty = aim[1]

    def length_bonus(star, diff):
    return (
        0.32 + 0.5 * (math.log10(diff + star) - math.log10(star))
    )

    self.aim_length_bonus = length_bonus(self.aim, self.aim_difficulty)
    self.speed_length_bonus = (
        length_bonus(self.speed, self.speed_difficulty)
    )
    self.aim = math.sqrt(self.aim) * STAR_SCALING_FACTOR
    self.speed = math.sqrt(self.speed) * STAR_SCALING_FACTOR
    if mods & MODS_TOUCH_DEVICE != 0:
self.aim = pow(self.aim, 0.8)

    # total stars
    self.total = self.aim + self.speed
    self.total += (
        abs(self.speed - self.aim) *
        EXTREME_SCALING_FACTOR
    )

    # singletap stats
    for i, obj in enumerate(objs[1:]):
        prev = objs[i]

        if obj.is_single:
        self.nsingles += 1

        if obj.objtype & (OBJ_CIRCLE | OBJ_SLIDER) == 0:
        continue

        interval = (obj.time - prev.time) / speed_mul

        if interval >= singletap_threshold:
        self.nsingles_threshold += 1


        return self

        */

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
