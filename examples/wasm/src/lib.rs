#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};

#[embassy::task]
async fn ticker() {
    let window = web_sys::window().expect("no global `window` exists");

    let mut counter = 0;
    loop {
        let document = window.document().expect("should have a document on window");
        let list = document.get_element_by_id("log").expect("should have a log element");

        let li = document.create_element("li").expect("error creating list item element");
        li.set_text_content(Some(&format!("tick {}", counter)));

        list.append_child(&li).expect("error appending list item");
        log::info!("tick {}", counter);
        counter += 1;

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy::main]
async fn main(spawner: Spawner) {
    wasm_logger::init(wasm_logger::Config::default());
    spawner.spawn(ticker()).unwrap();
}
