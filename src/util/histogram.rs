use std::hash::Hash;
use std::collections::HashMap;

pub type Hist<T> = HashMap<T, usize>;

pub fn hist<T>(input: impl IntoIterator<Item=T>) -> Hist<T>
    where T: Hash + Eq
{
    let mut hist = Hist::new();
    for entry in input.into_iter() {
        if let Some(count) = hist.get_mut(&entry) {
            *count += 1;
        }
        else {
            hist.insert(entry, 1);
        }
    }
    hist
}
