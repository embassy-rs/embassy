//! ESP-Hosted protobuf message bindings.

/*
Cargo.toml
[build-dependencies]
micropb-gen = "0.6.0"

build.rs
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // Or update the files directly:
    // let out_dir = "src/proto";

    let mut g = micropb_gen::Generator::new();
    g.use_container_heapless();

    g.configure(
        ".",
        micropb_gen::Config::new()
            .max_bytes(32) // For ssid, mac, etc - strings
            .max_len(16) // For repeated fields
            .type_attributes("#[cfg_attr(feature = \"defmt\", derive(defmt::Format))]"),
    );

    // Special config for things that need to be larger
    g.configure(
        ".CtrlMsg_Req_OTAWrite.ota_data",
        micropb_gen::Config::new().max_bytes(256),
    );
    g.configure(
        ".CtrlMsg_Event_ESPInit.init_data",
        micropb_gen::Config::new().max_bytes(64),
    );
    g.configure(
        ".CtrlMsg_Req_VendorIEData.payload",
        micropb_gen::Config::new().max_bytes(64),
    );

    g.compile_protos(
        &["src/proto/fg/esp_hosted_config.proto"],
        format!("{}/fg/mod.rs", out_dir),
    )
    .unwrap();

    let mut g = micropb_gen::Generator::new();
    g.use_container_heapless();

    g.configure(
        ".",
        micropb_gen::Config::new()
            .max_bytes(32) // For ssid, mac, etc - strings
            .max_len(16) // For repeated fields
            .type_attributes("#[cfg_attr(feature = \"defmt\", derive(defmt::Format))]"),
    );

    // Special config for things that need to be larger
    g.configure(".Rpc_Req_OTAWrite.ota_data", micropb_gen::Config::new().max_bytes(256));
    g.configure(".Rpc_Event_ESPInit.init_data", micropb_gen::Config::new().max_bytes(64));

    g.compile_protos(
        &["src/proto/mcu/esp_hosted_rpc.proto"],
        format!("{}/mcu/mod.rs", out_dir),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=src/proto/fg/esp_hosted_config.proto");
    println!("cargo:rerun-if-changed=src/proto/mcu/esp_hosted_rpc.proto");
}
 */

#[allow(unused)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(missing_docs)]
#[allow(clippy::all)]
#[cfg(feature = "esp-hosted-fg")]
pub(crate) mod fg;

#[allow(unused)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(missing_docs)]
#[allow(clippy::all)]
#[cfg(feature = "esp-hosted-mcu")]
pub(crate) mod mcu;
