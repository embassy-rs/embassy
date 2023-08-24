register!(RxStatus, 0, u32, {
    #[doc = "Indicates length of the received frame"]
    byte_count @ 0..15,
    #[doc = "Indicates a packet over 50,000 bit times occurred or that a packet was dropped since the last receive"]
    long_event @ 16,
    #[doc = "Indicates that at some time since the last receive, a carrier event was detected"]
    carrier_event @ 18,
    #[doc = "Indicates that frame CRC field value does not match the CRC calculated by the MAC"]
    crc_error @ 20,
    #[doc = "Indicates that frame length field value in the packet does not match the actual data byte length and specifies a valid length"]
    length_check_error @ 21,
    #[doc = "Indicates that frame type/length field was larger than 1500 bytes (type field)"]
    length_out_of_range @ 22,
    #[doc = "Indicates that at the packet had a valid CRC and no symbol errors"]
    received_ok @ 23,
    #[doc = "Indicates packet received had a valid Multicast address"]
    multicast @ 24,
    #[doc = "Indicates packet received had a valid Broadcast address."]
    broadcast @ 25,
    #[doc = "Indicates that after the end of this packet, an additional 1 to 7 bits were received"]
    dribble_nibble @ 26,
    #[doc = "Current frame was recognized as a control frame for having a valid type/length designating it as a control frame"]
    receive_control_frame @ 27,
    #[doc = "Current frame was recognized as a control frame containing a valid pause frame opcode and a valid destination address"]
    receive_pause_control_frame @ 28,
    #[doc = "Current frame was recognized as a control frame but it contained an unknown opcode"]
    receive_unknown_opcode @ 29,
    #[doc = "Current frame was recognized as a VLAN tagged frame"]
    receive_vlan_type_detected @ 30,
});
