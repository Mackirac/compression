use std::thread;
use std::iter::FromIterator;
use crate::util::binary::Bin;

pub fn encode(input: Vec<u8>) -> Vec<bool> {
    let threads : Vec<_> = (0..8).map(|index| {
        let input = input.clone();
        thread::spawn(move || {
            compress(2_u8.pow(index), input)
        })
    }).collect();
    let (exp, mut code) = threads
        .into_iter()
        .map(|t| t.join().unwrap())
        .enumerate()
        .min_by(|a, b| a.1.len().cmp(&b.1.len()))
        .unwrap();
    println!("Exp: {:?}", exp);
    let mut output = Bin::from_dec(exp, 3).unwrap().unwrap();
    output.append(&mut code);
    output
}

pub fn decode(input: &[bool]) -> Result<Vec<u8>, ()> {
    let exp = Bin::from_iter(&input[0..3]).to_dec() as u32;
    decompress(2_u8.pow(exp), input[3..].iter().cloned())
}

fn compress(factor: u8, input: impl IntoIterator<Item=u8>) -> Vec<bool> {
    let mut output = vec!();
    let rest_len = (factor as f32).log2().floor() as u32;
    for byte in input {
        for _ in 0..byte/factor { output.push(true) }
        output.push(false);
        output.append(&mut Bin::from_dec(
            (byte%factor) as usize,
            rest_len
        ).unwrap().unwrap());
    }
    output
}

fn decompress(factor: u8, mut input: impl Iterator<Item=bool>) -> Result<Vec<u8>, ()> {
    let mut output = vec!();
    let rest_len = (factor as f32).log2().floor() as u32;
    let mut counter = 0;
    while let Some(v) = input.next() {
        if v { counter += 1 }
        else {
            let mut rest = Vec::with_capacity(rest_len as usize);
            for _ in 0..rest_len {
                if let Some(v) = input.next() { rest.push(v) }
                else { return Err(()) }
            }
            output.push(counter * factor + Bin::from_iter(rest).to_dec() as u8);
            counter = 0;
        }
    }
    Ok(output)
}

#[test]
fn encode_test() {
    use std::path::Path;
    use std::fs::File;
    use std::io::Read;

    let path = Path::new("C:/Users/Mateus/Desktop/image.bmp");
    let mut file = File::open(path).unwrap();
    let mut buffer = vec!();

    file.read_to_end(&mut buffer).unwrap();
    let encoded = encode(buffer.clone());

    println!("Tamanho da entrada: {:?} bits.", buffer.len() * 8);
    println!("Tamanho da saída: {:?} bits.", encoded.len());

    println!(
        "Taxa de compressão: {:.2?}%",
        100. * (1. - (encoded.len() as f64 / (buffer.len() * 8) as f64))
    );

    let decoded = decode(&encoded);
    assert_eq!(decoded, Ok(buffer));
}
