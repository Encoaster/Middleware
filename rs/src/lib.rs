//! TODO: Create documentation

mod middleware;

pub use middleware::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn printing() {
        println!("it works");
    }
}
