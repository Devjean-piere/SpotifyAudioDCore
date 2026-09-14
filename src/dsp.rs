use crate::eq::{BandEq, GraphicEq};
use crate::structs::BandEqGains;

pub fn change_volume(samples: &mut [(i16, i16)], volume: f32) {
    for (left, right) in samples.iter_mut() {
        let l = (*left as f32) * volume;
        let r = (*right as f32) * volume ;

        *left = l.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        *right = r.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    }
}

pub fn band_eq(
    samples: &mut [(i16, i16)],
    gains: BandEqGains,
    eq_left: &mut BandEq,
    eq_right: &mut BandEq,
) {
    let min_val = i16::MIN as f32;
    let max_val = i16::MAX as f32;

    for (left, right) in samples.iter_mut() {
        let mut l_in = *left as f32;
        let mut r_in = *right as f32;

        eq_left.process(&mut l_in, gains);
        eq_right.process(&mut r_in, gains);

        *left = l_in.clamp(min_val, max_val) as i16;
        *right = r_in.clamp(min_val, max_val) as i16;
    }
}

pub fn graph_eq(
    samples: &mut [(i16, i16)],
    eq_left: &mut GraphicEq,
    eq_right: &mut GraphicEq,
) {
    for (left, right) in samples.iter_mut() {
        let l_in = *left as f32;
        let r_in = *right as f32;

        let l_out = eq_left.process(l_in);
        let r_out = eq_right.process(r_in);

        *left = l_out.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        *right = r_out.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    }
}