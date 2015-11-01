#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate gpredict;

use gpredict::{Predict, Location, Tle};

use std::thread;

fn conf_logger() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            let t = time::now();
            let ms = t.tm_nsec/1000_000;
            format!("{}.{:3} [{}] {}", t.strftime("%Y-%m-%dT%H:%M:%S").unwrap(), ms, level, msg)
        }),
        output: vec![fern::OutputConfig::stderr()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}

fn main() {
    // setup fern logger
    conf_logger();

    // start processing
    info!("predict example started");

    let tle: Tle = Tle {
        name: "GRIFEX".to_string(),
        line1: "1 40379U 15003D   15243.42702278  .00003367  00000-0  17130-3 0  9993".to_string(),
        line2: "2 40379  99.1124 290.6779 0157088   8.9691 351.4280 15.07659299 31889".to_string()
    };

    let location: Location = Location{lat_deg:58.64560, lon_deg: 23.15163, alt_m: 8};
    let mut predict: Predict = Predict::new(&tle, &location);

    loop {
        // these two are the same:
        //predict.update(Some(time::now_utc()));
        predict.update(None);

        info!("aos        : {:}", predict.sat.aos.expect("do not have AOS with this satellite").to_utc().rfc3339());
        info!("los        : {:}", predict.sat.los.expect("do not have LOS with this satellite").to_utc().rfc3339());
        info!("az         : {:.2}°", predict.sat.az_deg);
        info!("el         : {:.2}°", predict.sat.el_deg);
        info!("range      : {:.0} km", predict.sat.range_km);
        info!("range rate : {:.3} km/sec\n", predict.sat.range_rate_km_sec);

        thread::sleep_ms(1000);
    }
}
