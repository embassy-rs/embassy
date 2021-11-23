use std::env::args;
use std::path::PathBuf;
use stm32_metapac_gen::*;

fn main() {
    let out_dir = PathBuf::from("out");
    let data_dir = PathBuf::from("../stm32-data/data");

    let args: Vec<String> = args().collect();

    let chips = match &args[..] {
        [_, chip] => {
            vec![chip.clone()]
        }
        [_] => {
            std::fs::read_dir(data_dir.join("chips"))
                .unwrap()
                .filter_map(|res| res.unwrap().file_name().to_str().map(|s| s.to_string()))
                .filter(|s| s.ends_with(".yaml"))
                .filter(|s| !s.starts_with("STM32GBK")) // cursed weird STM32G4
                .map(|s| s.strip_suffix(".yaml").unwrap().to_string())
                .collect()
        }
        _ => panic!("usage: stm32-metapac-gen [chip?]"),
    };

    gen(Options {
        out_dir,
        data_dir,
        chips,
    })
}
