/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2015 Andres Vahter (andres.vahter@gmail.com)
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use ffipredict;
use tle;

use std::default::Default;
use time;
use ::sat::Sat;
use ::julian_time::{julian_timestamp, julian_to_unix};


#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct Location {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: i32,
}

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.alt_m == other.alt_m &&
        (self.lat_deg - other.lat_deg).abs() < 0.000_000_1 &&
        (self.lon_deg - other.lon_deg).abs() < 0.000_000_1
    }
}

#[test]
fn location_partialeq() {
    let first  = Location { lat_deg: 56.7865,            lon_deg: 21.4444,            alt_m: 8 };
    let second = Location { lat_deg: 56.786500000000004, lon_deg: 21.444399999999998, alt_m: 8 };
    assert_eq!(first, second);
}


#[derive(Debug)]
pub struct Predict {
    pub sat: Sat,

    p_sat: ffipredict::sat_t,
    p_qth: ffipredict::qth_t,
}

impl Predict {

    pub fn new(tle: &tle::Tle, location: &Location) -> Predict {
        let tle_t = tle::create_tle_t(tle).unwrap();

        let sgps: ffipredict::sgpsdp_static_t = Default::default();
        let dps: ffipredict::deep_static_t = Default::default();
        let deep_arg: ffipredict::deep_arg_t = Default::default();
        let pos: ffipredict::vector_t = Default::default();
        let vel: ffipredict::vector_t = Default::default();

        let mut sat_t = ffipredict::sat_t{
            name: b"placeholder\0".as_ptr() as *const i8,
            nickname: b"placeholder\0".as_ptr() as *const i8,
            website: b"placeholder\0".as_ptr() as *const i8,
            tle: tle_t,
            flags: 0,
            sgps: sgps,
            dps: dps,
            deep_arg: deep_arg,
            pos: pos,
            vel: vel,

            jul_epoch: 0.0,
            jul_utc: 0.0,
            tsince: 0.0,
            aos: 0.0,
            los: 0.0,
            az: 0.0,
            el: 0.0,
            range: 0.0,
            range_rate: 0.0,
            ra: 0.0,
            dec: 0.0,
            ssplat: 0.0,
            ssplon: 0.0,
            alt: 0.0,
            velo: 0.0,
            ma: 0.0,
            footprint: 0.0,
            phase: 0.0,
            meanmo: 0.0,
            orbit: 0,
            otype: ffipredict::orbit_type_t::ORBIT_TYPE_UNKNOWN,
        };

        let sat: Sat = Default::default();
        let mut qth = ffipredict::qth_t {
            name: b"placeholder\0".as_ptr() as *const i8,
            loc: b"placeholder\0".as_ptr() as *const i8,
            desc: b"placeholder\0".as_ptr() as *const i8,
            lat: location.lat_deg,
            lon: location.lon_deg,
            alt: location.alt_m,
            qra: b"placeholder\0".as_ptr() as *const i8,
            wx: b"placeholder\0".as_ptr() as *const i8,
        };

        unsafe {ffipredict::select_ephemeris(&mut sat_t)};
        unsafe {ffipredict::gtk_sat_data_init_sat(&mut sat_t, &mut qth)};

        Predict{sat: sat, p_sat: sat_t, p_qth: qth}
    }

    pub fn update(&mut self, timeoption: Option<time::Tm>) {
        let juliantime = match timeoption {
            Some(t) => julian_timestamp(t),
            None => unsafe {ffipredict::get_current_daynum()}
        };

        // we do not have AOS with some satellites, therefore option is used
        let aos = match unsafe {ffipredict::find_aos(&mut self.p_sat, &mut self.p_qth, juliantime, 1.0)} {
            0.0 => None,
            aos => Some(julian_to_unix(aos)),
        };
        let los = match unsafe {ffipredict::find_los(&mut self.p_sat, &mut self.p_qth, juliantime, 1.0)} {
            0.0 => None,
            los => Some(julian_to_unix(los)),
        };

        unsafe {ffipredict::predict_calc(&mut self.p_sat, &mut self.p_qth, juliantime)};

        self.sat.aos                = aos;
        self.sat.los                = los;
        self.sat.az_deg             = self.p_sat.az;
        self.sat.el_deg             = self.p_sat.el;
        self.sat.range_km           = self.p_sat.range;
        self.sat.range_rate_km_sec  = self.p_sat.range_rate;
        self.sat.lat_deg            = self.p_sat.ssplat;
        self.sat.lon_deg            = self.p_sat.ssplon;
        self.sat.alt_km             = self.p_sat.alt;
        self.sat.vel_km_s           = self.p_sat.velo;
        self.sat.orbit_nr           = self.p_sat.orbit as u64;
    }
}
