use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Class {
    Ruby,
    Diamond,
    Sapphire
}