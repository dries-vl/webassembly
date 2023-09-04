use cfg_if::cfg_if;
use crate::state::texture;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[derive(Debug)]
pub enum ResourceError {
    TextureError(texture::TextureError),
}

#[cfg(target_arch = "wasm32")]
// serve files as a local webserver and make http requests to get them
fn format_url(file_name: &str) -> String {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("/webassembly/res") {
        origin = format!("{}/webassembly/res", origin);
    }
    format!("{}/{}", origin, file_name)
}

pub async fn load_string(file_name: &str) -> String {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let window = web_sys::window().unwrap();
            let fetch_promise = window.fetch_with_str(&url);

            let js_future = JsFuture::from(fetch_promise);
            let result = js_future.await.unwrap();

            use wasm_bindgen::JsCast;
            let response: web_sys::Response = result.dyn_into().unwrap();
            let text_promise = response.text().unwrap();

            let js_future = JsFuture::from(text_promise);
            let txt: String = js_future.await.unwrap().as_string().unwrap();

        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path).unwrap();
        }
    }

    txt
}

pub async fn load_binary(file_name: &str) -> Vec<u8> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let window = web_sys::window().unwrap();
            let fetch_promise = window.fetch_with_str(&url);

            let js_future = JsFuture::from(fetch_promise);
            let result = js_future.await.unwrap();

            use wasm_bindgen::JsCast;
            let response: web_sys::Response = result.dyn_into().unwrap();
            let array_buffer_promise = response.array_buffer().unwrap();

            let js_future = JsFuture::from(array_buffer_promise);
            let array_buffer: js_sys::ArrayBuffer = js_future.await.unwrap().dyn_into().unwrap();

            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let mut data = vec![0; uint8_array.length() as usize];
            uint8_array.copy_to(&mut data);

        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path).unwrap();
        }
    }

    data
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<texture::Texture, texture::TextureError> {
    let data = load_binary(file_name).await;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}
