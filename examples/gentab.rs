// this generates the table used for the rolling CRC hash

fn main() {
    let poly: u64 = 0xc96c5795d7870f42; // ECMA polynomial
    for i in 0..256 {
        let mut r: u64 = i;
        for _ in 0..8 {
            if r & 1 == 1 {
                r = (r >> 1) ^ poly;
            } else {
                r >>= 1;
            }
        }
        print!("0x{:016x}, ", r);
        if i % 4 == 3 {
            print!("\n");
        }
    }
}
