use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use photon_rs::transform::SamplingFilter;
use prost::Message;

mod abi;
pub use abi::*;

impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

impl From<&ImageSpec> for String {
    fn from(value: &ImageSpec) -> Self {
        let data = value.encode_to_vec();
        BASE64_URL_SAFE_NO_PAD.encode(data)
    }
}

impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = BASE64_URL_SAFE_NO_PAD.decode(value)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

impl filter::Filter {
    pub fn to_str(self) -> Option<&'static str> {
        match self {
            filter::Filter::Unspecified => None,
            filter::Filter::Oceanic => Some("oceanic"),
            filter::Filter::Islands => Some("Islands"),
            filter::Filter::Marine => Some("Marine"),
        }
    }
}

impl From<resize::SampleFilter> for SamplingFilter {
    fn from(value: resize::SampleFilter) -> Self {
        match value {
            resize::SampleFilter::Undefined => SamplingFilter::Nearest,
            resize::SampleFilter::Nearest => SamplingFilter::Nearest,
            resize::SampleFilter::Triangle => SamplingFilter::Triangle,
            resize::SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            resize::SampleFilter::Gaussian => SamplingFilter::Gaussian,
            resize::SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

impl Spec {
    pub fn new_resize_seam_carve(width: u32, height: u32) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                resize_type: resize::ResizeType::SeamCarve as i32,
                sample_filter: resize::SampleFilter::Undefined as i32,
            })),
        }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                resize_type: resize::ResizeType::Normal as i32,
                sample_filter: filter as i32,
            })),
        }
    }

    pub fn new_filter(filter: filter::Filter) -> Self {
        Self {
            data: Some(spec::Data::Filter(Filter {
                filter: filter as i32,
            })),
        }
    }

    pub fn new_watermark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::WaterMark(WaterMark { x, y })),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::borrow::Borrow;

    #[test]
    fn encoded_spec_could_be_decoded() {
        let resize_spec = Spec::new_resize(600, 600, resize::SampleFilter::CatmullRom);
        let filter_spec = Spec::new_filter(filter::Filter::Marine);

        let image_spec = ImageSpec::new(vec![resize_spec, filter_spec]);

        let str: String = image_spec.borrow().into();

        assert_eq!(image_spec, str.as_str().try_into().unwrap());
    }
}
