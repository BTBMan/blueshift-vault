use blueshift_vault::ID;
use mollusk_svm::Mollusk;

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_deposit() {
        let mollusk = Mollusk::new(&ID, "target/deploy/blueshift_vault.so");
    }
}
