mod buffer;

pub use buffer::{InstantStat, StatWindow};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
