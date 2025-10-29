fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

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
        micropb_gen::Config::new().max_bytes(1024),
    );
    g.configure(
        ".CtrlMsg_Event_ESPInit.init_data",
        micropb_gen::Config::new().max_bytes(64),
    );
    g.configure(
        ".CtrlMsg_Req_VendorIEData.payload",
        micropb_gen::Config::new().max_bytes(64),
    );

    g.compile_protos(&["src/esp_hosted_config.proto"], format!("{}/proto.rs", out_dir))
        .unwrap();

    println!("cargo:rerun-if-changed=src/esp_hosted_config.proto");
}
