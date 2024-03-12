use image::ImageFormat;

pub use photon::Photon;

use crate::pb::Spec;

mod photon;

pub trait Engine {
    fn apply(&mut self, specs: &[Spec]);
    fn generate(self, format: ImageFormat) -> Vec<u8>;
}

pub trait SpecTransform<T> {
    fn transform(&mut self, op: T);
}
