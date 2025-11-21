#[allow(dead_code)]
const SMD291AXT4: [(f32, f32); 6] = [
    (0.0, 25.0),
    (30.0, 100.0),
    (120.0, 150.0),
    (150.0, 183.0),
    (210.0, 235.0),
    (240.0, 183.0),
];

pub struct Profile {}

#[allow(dead_code)]
impl Profile {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_target(&self, time: f32) -> f32 {
        time
    }
}
