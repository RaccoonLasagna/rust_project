use img_to_ascii::*;

fn main() {
    let braille = char::from_u32(0x28FF).unwrap();
    let brailleempty = char::from_u32(0x2800).unwrap();
    for _ in 0..100{
        print!("{}", braille)
    }
    println!("");
    for _ in 0..99{
        print!("{}", " ")
    }
    print!("{}", braille)
}