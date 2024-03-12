use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use lazy_static::lazy_static;
use photon_rs::{
    effects, filters, multiple, native::open_image_from_bytes, transform, PhotonImage,
};

use crate::pb::*;

use super::{Engine, SpecTransform};

lazy_static! {
    static ref WATERMARK: PhotonImage = {
        let data = include_bytes!("../../turtle.png");
        let watermark = open_image_from_bytes(data).unwrap();

        transform::resize(&watermark, 64, 64, transform::SamplingFilter::Nearest)
    };
}

pub struct Photon(PhotonImage);

impl TryFrom<Bytes> for Photon {
    type Error = anyhow::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        Ok(Self(open_image_from_bytes(&value)?))
    }
}

impl Engine for Photon {
    fn apply(&mut self, specs: &[Spec]) {
        for spec in specs.iter() {
            match spec.data {
                Some(spec::Data::Crop(ref v)) => self.transform(v),
                Some(spec::Data::Contrast(ref v)) => self.transform(v),
                Some(spec::Data::Filter(ref v)) => self.transform(v),
                Some(spec::Data::FlipH(ref v)) => self.transform(v),
                Some(spec::Data::FlipV(ref v)) => self.transform(v),
                Some(spec::Data::Resize(ref v)) => self.transform(v),
                Some(spec::Data::WaterMark(ref v)) => self.transform(v),
                _ => {}
            }
        }
    }

    fn generate(self, format: ImageFormat) -> Vec<u8> {
        image_to_buf(self.0, format)
    }
}

fn image_to_buf(img: PhotonImage, format: ImageFormat) -> Vec<u8> {
    let raw_pixels = img.get_raw_pixels();
    let width = img.get_width();
    let height = img.get_height();

    let img_buf = ImageBuffer::from_vec(width, height, raw_pixels).unwrap();
    let dyn_img = DynamicImage::ImageRgba8(img_buf);

    let mut buffer = Cursor::new(Vec::with_capacity(32768));
    dyn_img.write_to(&mut buffer, format).unwrap();
    buffer.into_inner()
}

impl SpecTransform<&Crop> for Photon {
    fn transform(&mut self, op: &Crop) {
        let img = transform::crop(&mut self.0, op.x1, op.y1, op.x2, op.y2);
        self.0 = img;
    }
}

impl SpecTransform<&Contrast> for Photon {
    fn transform(&mut self, op: &Contrast) {
        effects::adjust_contrast(&mut self.0, op.contrast);
    }
}

impl SpecTransform<&FlipH> for Photon {
    fn transform(&mut self, _op: &FlipH) {
        transform::fliph(&mut self.0);
    }
}

impl SpecTransform<&FlipV> for Photon {
    fn transform(&mut self, _op: &FlipV) {
        transform::flipv(&mut self.0)
    }
}

impl SpecTransform<&Filter> for Photon {
    fn transform(&mut self, op: &Filter) {
        match filter::Filter::try_from(op.filter).ok() {
            Some(filter::Filter::Unspecified) => {}
            Some(f) => filters::filter(&mut self.0, f.to_str().unwrap()),
            _ => {}
        }
    }
}

impl SpecTransform<&Resize> for Photon {
    fn transform(&mut self, op: &Resize) {
        let img = match resize::ResizeType::try_from(op.resize_type).unwrap() {
            resize::ResizeType::Normal => transform::resize(
                &self.0,
                op.width,
                op.height,
                resize::SampleFilter::try_from(op.sample_filter)
                    .unwrap()
                    .into(),
            ),
            resize::ResizeType::SeamCarve => transform::seam_carve(&self.0, op.width, op.height),
        };

        self.0 = img;
    }
}

impl SpecTransform<&WaterMark> for Photon {
    fn transform(&mut self, op: &WaterMark) {
        multiple::watermark(&mut self.0, &WATERMARK, op.x, op.y);
    }
}
