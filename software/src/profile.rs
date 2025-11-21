use defmt::Debug2Format;
use heapless::Vec;
use pid::{ControlOutput, Pid};
use splines::{Key, Spline};

#[allow(dead_code)]
const SMD291AXT4: [(f32, f32); 9] = [
    (-1.0, 25.0),
    // documentation starts here
    (0.0, 25.0),
    (30.0, 100.0),
    (120.0, 150.0),
    (150.0, 183.0),
    (210.0, 235.0),
    (240.0, 183.0),
    // documentation ends here
    (270.0, 25.0),
    (300.0, 25.0),
];

pub struct Profile {
    spline: Spline<f32, f32>,
    pid: Pid<f32>,
    pub heatsoak_target: f32,
}

#[allow(dead_code)]
impl Profile {
    pub fn new() -> Self {
        let heatsoak_target = 40.0;

        let mut spline = Spline::default();
        let keys: Vec<Key<f32, f32>, 9> = SMD291AXT4
            .iter()
            .map(|(time, target)| Key::new(*time, *target, splines::Interpolation::CatmullRom))
            .collect();

        for key in keys {
            spline.add(key);
        }

        let mut pid = Pid::new(spline.sample(0.0).unwrap_or(0.0), 60.0);

        // adjust here
        pid.p(1.0, 5.0);
        pid.i(0.01, 1.0);
        pid.d(0.1, 5.0);
        defmt::info!("{}", Debug2Format(&pid));
        Self {
            spline,
            pid,
            heatsoak_target,
        }
    }

    pub fn desired_temperature(&self, time: f32) -> f32 {
        self.spline.sample(time).unwrap_or(0.0)
    }

    pub fn control_output(&mut self, time: f32, current_temperature: f32) -> ControlOutput<f32> {
        let target = self.spline.sample(time).unwrap_or(0.0);
        self.pid.setpoint(target);
        self.pid.next_control_output(current_temperature)
    }
}
