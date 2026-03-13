#[derive(Debug)]
pub struct BankAccount{
    balance: f64,
}

impl BankAccount{
    pub fn new(initial_balance: f64) -> BankAccount{
        BankAccount{
            balance: initial_balance
        }
    }

    pub fn deposit(&mut self, amount: f64){
        if amount < 0.0{
            return;
        }
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: f64){
        if amount < 0.0 || amount > self.balance{
            return;
        }
        self.balance -= amount;
    }

    pub fn balance(&self) -> f64{
        self.balance
    }
}

#[cfg(test)]
mod tests {

    // Allows us to access BankAccount
    use super::*;

    // Test creating a new account
    #[test]
    fn test_new_account() {

        let account = BankAccount::new(100.0);

        // Check if balance was set correctly
        assert_eq!(account.balance(), 100.0);
    }

    // Test depositing money
    #[test]
    fn test_deposit() {

        let mut account = BankAccount::new(100.0);

        account.deposit(50.0);

        assert_eq!(account.balance(), 150.0);
    }

    // Test withdrawing money
    #[test]
    fn test_withdraw() {

        let mut account = BankAccount::new(100.0);

        account.withdraw(40.0);

        assert_eq!(account.balance(), 60.0);
    }

    // Test withdrawing too much money
    #[test]
    fn test_withdraw_too_much() {

        let mut account = BankAccount::new(100.0);

        // Attempt to withdraw more than balance
        account.withdraw(200.0);

        // Balance should remain unchanged
        assert_eq!(account.balance(), 100.0);
    }

    // Test depositing a negative number
    #[test]
    fn test_deposit_negative() {

        let mut account = BankAccount::new(100.0);

        account.deposit(-50.0);

        // Balance should not change
        assert_eq!(account.balance(), 100.0);
    }

    // Test withdrawing a negative number
    #[test]
    fn test_withdraw_negative() {

        let mut account = BankAccount::new(100.0);

        account.withdraw(-30.0);

        // Balance should not change
        assert_eq!(account.balance(), 100.0);
    }
}