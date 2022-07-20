use solana_program::entrypoint;
pub mod state;
pub mod processor;
pub mod instruction;
use processor::process_intruction;

entrypoint!(process_intruction);

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    // use solana_program::{msg};

    use crate::instruction::InstructionEnum;

    #[test]
    fn verifying_expected_instruction_enum_serialization() {
        let data = InstructionEnum::MintNewCollection.try_to_vec().unwrap();
        println!("data:  {:?}", data);
        assert_eq!(data[0], 1);
    }
}
