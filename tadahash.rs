fn hash<D: IntoIterator<Item = u8>>(data: D) -> u32 {
    let mut hash: u32 = 656_379_989;

    let mut last_byte = 0;

    for byte in data.into_iter() {
        hash = hash.wrapping_mul(byte as u32);

        hash ^= (byte as u32).rotate_left(5).wrapping_neg();

        hash = hash.wrapping_sub(last_byte as u32);

        last_byte = byte;
    }

    hash.wrapping_mul(205_676_507)
}
