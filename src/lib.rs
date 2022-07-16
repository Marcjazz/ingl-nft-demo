use solana_program::entrypoint;
pub mod state;
pub mod processor;
pub mod instruction;
use processor::process_intruction;

entrypoint!(process_intruction);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
