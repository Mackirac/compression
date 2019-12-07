use std::iter::FromIterator;
use std::fmt::{ self, Debug, Formatter };

#[derive(PartialEq, Eq)]
pub struct Bin {
    bits: Vec<bool>
}

impl Debug for Bin {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut buffer = String::new();
        for i in (0..self.bits.len()).rev() {
            buffer.push(if self.bits[i] { '1' } else { '0' });
        }
        write!(f, "{}", buffer)
    }
}

impl Bin {
    pub fn from_dec(mut dec: usize, bits: u32) -> Result<Self, ()> {
        if dec > 2_u32.pow(bits) as usize - 1 { return Err(()) }
        let mut bits : Vec::<bool> = std::iter::repeat(false).take(bits as usize).collect();
        for i in 0..bits.len() {
            if dec % 2 == 1 { bits[i] = true }
            dec /= 2;
        } 
        Ok(Self { bits })
    }

    pub fn to_dec(&self) -> usize {
        let mut dec = 0_usize;
        for i in 0..self.bits.len() {
            if self.bits[i] { dec += 2_u32.pow(i as u32) as usize }
        }
        dec
    }

    pub fn unwrap(self) -> Vec<bool> {
        self.bits
    }
}

impl FromIterator<bool> for Bin {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item=bool>
    {
        let bits = iter.into_iter().collect();
        Self { bits }
    }
}

impl <'a> FromIterator<&'a bool> for Bin {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item=&'a bool>
    {
        let bits = iter.into_iter().cloned().collect();
        Self { bits }
    }
}

#[test]
fn display_test() {
    let bin = Bin::from_iter(&[false, false, false, true]);
    assert_eq!(format!("{:?}", bin), "1000".to_string());
}

#[test]
fn from_dec_test() {
    assert_eq!(
        Bin::from_dec(8, 5),
        Ok(Bin::from_iter(&[false, false, false, true, false]))
    );
    assert_eq!(
        Bin::from_dec(8, 3),
        Err(())
    );
}

#[test]
fn to_dec_test() {
    let bin = Bin::from_dec(8, 10).unwrap();
    assert_eq!(bin.to_dec(), 8);
    let bin = Bin::from_dec(8, 4).unwrap();
    assert_eq!(bin.to_dec(), 8);
}
