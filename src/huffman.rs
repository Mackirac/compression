use std::iter::FromIterator;
use crate::util::binary::Bin;
use crate::util::histogram::Hist;
use std::collections::{ HashMap, HashSet };

pub type Dict = HashMap<u8, Vec<bool>>;

pub fn compress(input: &[u8], dict: &Dict) -> Result<Vec<bool>, ()> {
    let mut output = vec!();
    for byte in input {
        if let Some(code) = dict.get(byte) {
            output = [output, code.clone()].concat();
        }
        else { return Err(()) }
    }
    Ok(output)
}

pub fn decompress(input: &[bool], tree: &Tree) -> Result<Vec<u8>, ()> {
    let mut output = vec!();
    let mut node = tree;
    for bit in input.iter() {
        if let Tree::Inner(l, r) = node { node = if *bit { r } else { l } }
        if let Tree::Leaf(v) = node {
            output.push(*v);
            node = tree;
        }
    }
    if node != tree { return Err(()) }
    Ok(output)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tree {
    Inner(Box<Tree>, Box<Tree>),
    Leaf(u8)
}

impl Tree {
    pub fn new(histogram: Hist<u8>) -> Self {
        let mut histogram : Vec<(Self, usize)> = histogram
            .into_iter()
            .map(|(b, c)| (Self::Leaf(b), c))
            .collect();
        histogram.sort_by(|b, a| a.1.cmp(&b.1));
        while histogram.len() > 1 {
            let l = histogram.pop().unwrap();
            let r = histogram.pop().unwrap();
            let node = Self::Inner(Box::new(l.0), Box::new(r.0));
            let count = l.1 + r.1;
            match histogram.binary_search_by(|n| count.cmp(&n.1)) {
                Ok(index) | Err(index) => histogram.insert(index, (node, count))
            }
        }
        histogram.pop().unwrap().0
    }

    pub fn to_dict(&self) -> Dict {
        let mut dict = Dict::new();
        let mut code = vec!();
        let mut stack = vec!(self);
        let mut visited = HashSet::new();

        if let Self::Leaf(v) = self {
            dict.insert(*v, vec!(false));
            return dict;
        }

        Self::post_order_map(&mut stack, &mut code, &mut visited, &mut dict);

        dict
    }

    fn post_order_map<'a>(
        stack: &mut Vec<&'a Self>,
        code: &mut Vec<bool>,
        visited: &mut HashSet<&'a Self>,
        dict: &mut Dict
    ) {
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            match node {
                Self::Leaf(v) => { dict.insert(*v, code.clone()); },
                Self::Inner(l, r) => {
                    if !visited.contains(node) {
                        visited.insert(node);
                        stack.push(node);
                        stack.push(r);
                        stack.push(l);
                        code.push(false);
                        continue;
                    }
                }
            }
            if let Some(false) = code.pop() { code.push(true) }
        }
    }

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
fn compression_test() {
    use crate::util::histogram::hist;

    let input = vec!(0_u8, 0, 0, 2, 2, 2, 2, 5, 5, 10, 10, 10, 10, 10, 15);
    let hist = hist(input.iter().cloned());
    let tree = Tree::new(hist);
    let dict = tree.to_dict();
    let output = compress(&input, &dict).unwrap();
    let decomp = decompress(&output, &tree);

    println!("Tree: {:?}\n", tree);
    println!("Dict: {:?}\n", dict);
    println!("Compression output: {:?}\n", output);
    println!("Decompression output: {:?}", decomp);

    assert_eq!(decomp, Ok(input));
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
