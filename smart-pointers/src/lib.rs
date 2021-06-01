pub mod cell;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        cell::Cell::new(0);
    }
}
