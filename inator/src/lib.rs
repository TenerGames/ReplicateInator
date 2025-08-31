pub mod connections;
pub mod plugins;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Eq,PartialEq)]
pub enum NetworkSide{
    Client,
    Server
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
