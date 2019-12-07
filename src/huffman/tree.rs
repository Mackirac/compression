use std::iter::FromIterator;
use crate::util::binary::Bin;
// use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Inner(Box<Tree>, Box<Tree>),
    Leaf(u8)
}

impl Tree {
    pub fn serialize(&self) -> Vec<bool> {
        let mut buffer = vec!();
        let mut stack = vec!(self);
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            match node {
                Self::Leaf(v) => {
                    buffer.push(false);
                    let mut byte = Bin::from_dec(*v as usize, 8).unwrap().unwrap();
                    buffer.append(&mut byte);
                },
                Self::Inner(l, r) => {
                    buffer.push(true);
                    stack.push(r);
                    stack.push(l);
                }
            }
        }
        buffer
    }

    pub fn deserialize(input: impl Iterator<Item=bool>) -> Result<Self, ()> {
        let nodes = Self::map(input)?;
        Self::reduce(nodes)
    }

    fn map(mut input: impl Iterator<Item=bool>) -> Result<Vec<Option<Self>>, ()> {
        let mut nodes = vec!();
        while let Some(bit) = input.next() {
            if bit { nodes.push(None) }
            else {
                let mut byte = [false; 8];
                for i in 0..8 {
                    if let Some(bit) = input.next() {
                        byte[i] = bit;
                    }
                    else { return Err(()) }
                }
                nodes.push(Some(Self::Leaf(
                    Bin::from_iter(&byte).to_dec() as u8
                )));
            }
        }
        Ok(nodes)
    }

    fn reduce(mut input: Vec<Option<Self>>) -> Result<Self, ()> {
        while input.len() >= 3 {
            let length = input.len();
            input = Self::reduce_once(input);
            if length == input.len() { break }
        }
        if input.len() != 1 { return Err(()) }
        input.pop().unwrap().ok_or(())
    }

    fn reduce_once(input: Vec<Option<Self>>) -> Vec<Option<Self>> {
        let mut buffer = vec!();
        let mut i = 0;
        while i < input.len() {
            if i + 2 < input.len() {
                if let (None, Some(l), Some(r)) = (&input[i], &input[i+1], &input[i+2]) {
                    buffer.push(Some(Self::Inner(
                        Box::new(l.clone()),
                        Box::new(r.clone())
                    )));
                    i += 3;
                    continue;
                }
            }
            buffer.push(input[i].clone());
            i += 1;
        }
        return buffer
    }
}

#[test]
fn serialization() {
    let tree = Tree::Inner(
        Box::new(Tree::Inner(
            Box::new(Tree::Leaf(0)),
            Box::new(Tree::Leaf(1))
        )),
        Box::new(Tree::Leaf(2))
    );
    let binary = tree.serialize();
    assert_eq!(
        binary.iter().cloned().map(
            |bit| if bit { "1" } else { "0" }
        ).collect::<Vec<&str>>().concat(),
        "11000000000010000000001000000"
    );
    assert_eq!(Ok(tree), Tree::deserialize(binary.into_iter()));

    let tree = Tree::Inner(
        Box::new(Tree::Inner(
            Box::new(Tree::Leaf(0)),
            Box::new(Tree::Leaf(1))
        )),
        Box::new(Tree::Inner(
            Box::new(Tree::Leaf(2)),
            Box::new(Tree::Leaf(3))
        ))
    );
    let binary = tree.serialize();
    assert_eq!(
        binary.iter().cloned().map(
            |bit| if bit { "1" } else { "0" }
        ).collect::<Vec<&str>>().concat(),
        "110000000000100000001001000000011000000"
    );
    assert_eq!(Ok(tree), Tree::deserialize(binary.into_iter()));

    let binary = vec!(
        true,
            false,
                true, false, false, false, false, false, false, false,
            false,
                true, true, false, false, false, false, false, false,
    );
    assert_eq!(
        Ok(Tree::Inner(
            Box::new(Tree::Leaf(1)),
            Box::new(Tree::Leaf(3))
        )),
        Tree::deserialize(binary.into_iter())
    );

    let binary = vec!(false, false, false, false, false, false, false, false, true);
    assert_eq!(Ok(Tree::Leaf(128)), Tree::deserialize(binary.into_iter()));

    let binary = vec!(true, true, true, true, true, true, true, true, true);
    assert_eq!(Err(()), Tree::deserialize(binary.into_iter()));

    let binary = vec!(false, false, false, false, false, false, false, false);
    assert_eq!(Err(()), Tree::deserialize(binary.into_iter()));
}
