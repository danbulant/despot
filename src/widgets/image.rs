use std::sync::{Arc, LazyLock};

use cushy::{
    kludgine::{AnyTexture, LazyTexture},
    value::{CallbackDisconnected, CallbackHandle, Destination, Dynamic, Source, Value},
    widgets::Image,
};
use futures_util::lock::Mutex;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use image::imageops::FilterType;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tokio::task::JoinHandle;

use crate::rt::tokio_runtime;

pub trait ImageExt {
    fn new_empty() -> Self;

    fn load_url(&mut self, url: Dynamic<Option<String>>) -> CallbackHandle;

    fn with_url(mut self, url: Dynamic<Option<String>>) -> Self
    where
        Self: Sized,
    {
        self.load_url(url).persist();
        self
    }
}

static CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: HttpCacheOptions::default(),
        }))
        .build()
});

impl ImageExt for Image {
    fn new_empty() -> Self {
        Image::new(Dynamic::new(get_empty_texture()))
    }

    /// Makes the image connected to a URL
    /// Calling this multiple times on a single image may cause memory leaks
    fn load_url(&mut self, url: Dynamic<Option<String>>) -> CallbackHandle {
        // let texture = Dynamic::new(get_empty_texture());
        match &mut self.contents {
            Value::Constant(_) => self.contents = Value::Dynamic(Dynamic::new(get_empty_texture())),
            Value::Dynamic(dynamic) => dynamic.set(get_empty_texture()),
        }
        let texture = match &self.contents {
            Value::Dynamic(dynamic) => dynamic,
            _ => unreachable!(),
        };

        let prev_request_join = Arc::new(Mutex::new(None::<JoinHandle<()>>));
        url.for_each_try({
            let texture = texture.clone();
            move |url| {
                let texture_count = texture.instances();
                if texture_count <= 1 {
                    return Err(CallbackDisconnected);
                }
                println!("loading url {:?}", url);
                let guard = tokio_runtime().enter();
                let url = url.clone();
                let prev_request_join = prev_request_join.clone();
                let texture = texture.clone();
                let client = CLIENT.clone();
                tokio::spawn(async move {
                    let mut prev_request_join = prev_request_join.lock().await;
                    if let Some(prev_request_join) = prev_request_join.take() {
                        prev_request_join.abort();
                    }
                    println!("loading url {:?}", url);
                    if let Some(url) = url {
                        let texture = texture.clone();
                        let client = client.clone();
                        *prev_request_join = Some(tokio::spawn(async move {
                            let response = client.get(url).send().await.unwrap();
                            let bytes = response.bytes().await.unwrap();
                            let image = image::load_from_memory(&bytes).unwrap();
                            // let image = image.resize(128, 128, FilterType::Lanczos3);
                            let image_texture = LazyTexture::from_image(
                                image,
                                cushy::kludgine::wgpu::FilterMode::Linear,
                            );
                            let image_texture = AnyTexture::Lazy(image_texture);
                            texture.set(image_texture);
                        }));
                    } else {
                        texture.set(get_empty_texture());
                    }
                });
                drop(guard);
                Ok(())
            }
        })
    }
}

fn get_empty_texture() -> AnyTexture {
    AnyTexture::Lazy(LazyTexture::from_image(
        image::DynamicImage::ImageRgba8(image::ImageBuffer::new(1, 1)),
        cushy::kludgine::wgpu::FilterMode::Linear,
    ))
}
