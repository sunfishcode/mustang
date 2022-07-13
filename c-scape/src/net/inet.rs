#[no_mangle]
extern "C" fn htonl(hostlong: u32) -> u32 {
    hostlong.to_be()
}

#[no_mangle]
extern "C" fn htons(hostshort: u16) -> u16 {
    hostshort.to_be()
}

#[no_mangle]
extern "C" fn ntohl(netlong: u32) -> u32 {
    u32::from_be(netlong)
}

#[no_mangle]
extern "C" fn ntohs(netshort: u16) -> u16 {
    u16::from_be(netshort)
}
