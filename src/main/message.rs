pub use self::message::Message;
pub mod message {
    use crate::main::*;
    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Standard(String),
        Tree(String)
    }

    impl Message {
        pub fn get_contained_message(&self) -> &String {
            match self {
                Self::Standard(x) | Self::Tree(x) => x, 
            }
            //todo fix so that this works so that we don't
            //have to update this method every time a new type of message is added to the message
            //enum.
        }
    }
}
