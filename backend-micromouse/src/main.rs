use crate::map::Map;

pub mod map;
pub mod measurement;
pub mod direction;
pub mod position;

#[cfg(test)]
pub mod tests;

fn main() {
    let map = Map::<4>::new();
    println!("{}", map);
}
