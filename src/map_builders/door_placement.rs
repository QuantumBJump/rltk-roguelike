use super::{MetaMapBuilder, BuilderMap};
use rltk::RandomNumberGenerator;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    #[allow()]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<DoorPlacement> {
        #![allow(dead_code)]
        Box::new(DoorPlacement{})
    }

    fn doors(&mut self, _rng: &mut RandomNumberGenerator, _build_data: &mut BuilderMap) {

    }
}
