pub mod connections;
pub mod plugins;
pub mod systems;

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

    #[test]
    fn it_works() {

    }
}
