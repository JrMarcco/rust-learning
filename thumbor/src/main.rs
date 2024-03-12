use std::num::NonZeroUsize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract::{Extension, Path},
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Router,
};
use bytes::Bytes;
use image::ImageFormat;
use lru::LruCache;
use percent_encoding::{percent_decode_str, percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{info, instrument};

use engine::Photon;
use pb::*;

use crate::engine::Engine;

mod engine;
mod pb;

#[derive(Deserialize)]
struct Params {
    spec: String,
    url: String,
}

type Cache = Arc<Mutex<LruCache<u64, Bytes>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cache_size: usize = 1024;
    let cache: Cache = Arc::new(Mutex::new(LruCache::new(
        NonZeroUsize::new(cache_size).expect("cache size must be non-zero"),
    )));

    let app = Router::new()
        .route("/image/:spec/:url", get(generate))
        .layer(Extension(cache));

    let addr: String = "127.0.0.1:8080".parse().unwrap();

    print_test_url("https://images.pexels.com/photos/1562477/pexels-photo-1562477.jpeg?auto=compress&cs=tinysrgb&dpr=3&h=750&w=1260");

    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn generate(
    Path(Params { spec, url }): Path<Params>,
    Extension(cache): Extension<Cache>,
) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let url = percent_decode_str(&url).decode_utf8_lossy();
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let data = retrieve_img(&url, cache)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut engine: Photon = data
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    engine.apply(&spec.specs);
    let img = engine.generate(ImageFormat::Jpeg);

    info!("Finished processing: image size {}", img.len());
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/jpeg"));

    Ok((headers, img))
}

#[instrument(level = "info", skip(cache))]
async fn retrieve_img(url: &str, cache: Cache) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);

    let key = hasher.finish();

    let g = &mut cache.lock().await;
    let data = match g.get(&key) {
        Some(v) => {
            info!("Match cache {}", key);
            v.to_owned()
        }
        None => {
            info!("Retrieve url");
            let rsp = reqwest::get(url).await?;
            let data = rsp.bytes().await?;
            g.put(key, data.clone());
            data
        }
    };

    Ok(data)
}

fn print_test_url(url: &str) {
    use std::borrow::Borrow;

    let resize_spec = Spec::new_resize(500, 800, resize::SampleFilter::CatmullRom);
    let wm_spec = Spec::new_watermark(20, 20);
    let filter_spec = Spec::new_filter(filter::Filter::Oceanic);

    let image_spec = ImageSpec::new(vec![resize_spec, wm_spec, filter_spec]);

    let str: String = image_spec.borrow().into();
    let test_image = percent_encode(url.as_bytes(), NON_ALPHANUMERIC).to_string();

    print!(
        "test url: http://127.0.0.1:8080/image/{}/{}",
        str, test_image
    );
}
