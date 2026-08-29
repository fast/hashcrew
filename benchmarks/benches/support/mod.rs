pub fn input(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
        .collect()
}
