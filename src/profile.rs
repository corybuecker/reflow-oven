use heapless::Vec;
use pid::Pid;
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
    pub heatsoak_temperature_target: f32,
    pub cooling_time_target: f32,
}

#[allow(dead_code)]
impl Profile {
    pub fn new() -> Self {
        let heatsoak_target = 80.0;
        let cooling_time_target = 235.0;

        let mut spline = Spline::default();
        let keys: Vec<Key<f32, f32>, 9> = SMD291AXT4
            .iter()
            .map(|(time, target)| Key::new(*time, *target, splines::Interpolation::CatmullRom))
            .collect();

        for key in keys {
            spline.add(key);
        }

        let pid = Pid::new(spline.sample(0.0).unwrap_or(0.0), 100.0);

        Self {
            spline,
            pid,
            heatsoak_temperature_target: heatsoak_target,
            cooling_time_target,
        }
    }

    pub fn desired_temperature(&self, time: f32) -> f32 {
        self.spline.sample(time).unwrap_or(0.0)
    }

    pub fn control_output(&mut self, time: f32, current_temperature: f32) -> f32 {
        let current_time = time;
        let setpoint = self.spline.sample(time).unwrap_or(0.0);

        self.pid.setpoint(setpoint);

        if current_time < 30.0 {
            // During preheat, we're actually cooling from 80°C to track the setpoint
            // This phase needs gentler control
            self.pid.p(2.0, 10.0); // Lower P since we're not fighting thermal mass quite as much
            self.pid.i(0.005, 0.3); // Moderate I
            self.pid.d(1.0, 1.0); // Moderate D
        } else if current_time < 120.0 {
            // Soak
            self.pid.p(2.0, 10.0);
            self.pid.i(0.01, 0.8);
            self.pid.d(2.0, 1.0);
        } else if current_time < 210.0 {
            // Reflow ramp
            self.pid.p(2.5, 10.0);
            self.pid.i(0.005, 0.5);
            self.pid.d(3.0, 1.0); // High D to prevent overshoot at peak
        } else {
            // Cooling: turn off
            self.pid.p(0.0, 10.0);
            self.pid.i(0.0, 0.0);
            self.pid.d(0.0, 0.0);
        }

        let output = self.pid.next_control_output(current_temperature).output;

        output.clamp(0.0, 100.0)
    }
}
