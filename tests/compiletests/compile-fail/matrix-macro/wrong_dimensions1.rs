#[macro_use]
extern crate rulinalg;

fn main() {
    let mat = matrix!(0.0, 1.0; 2.0);
    //~^ error: mismatched types
    println!("{:?}", mat);
}