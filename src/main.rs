use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

struct Transaction {
    id: i32,
    amount: f64,
    withdraw_account: String,
    deposit_account: String,
}

struct Account {
    name: String,
    balance: RefCell<f64>, // This is needed because later on when we .find() the account we need to update, we can't
                           // update the balance directly because it is protected by an immutable reference returned by .find().
                           // We need to create a mutable reference to the balance to update it, and RefCell allows the
                           // .borrow_mut() function to return a mutable reference to the balance.
}

/// This executes a transaction on the supplied accounts
fn execute_transaction(transaction: &Transaction, accounts: &[Account; 3]) {
    // Print the transaction id
    println!(">> Current Transaction: {}", transaction.id);

    // If a withdraw_account is given, withdraw the amount from the account
    if transaction.withdraw_account.len() > 0 {
        // Find the account to withdraw from
        let withdraw_account = accounts
            .iter()
            .find(|account| account.name == transaction.withdraw_account);

        // If the account exists, withdraw the amount from it
        match withdraw_account {
            None => println!(">> Account {} not found", transaction.withdraw_account),
            Some(account) => {
                println!(
                    ">> Withdrawing {} from {}",
                    transaction.amount, account.name
                );

                // Update the balance
                *account.balance.borrow_mut() -= transaction.amount;
            }
        }
    }

    // If a deposit_account is given, deposit the amount to the account
    if transaction.deposit_account.len() > 0 {
        // Find the account to deposit to
        let deposit_account = accounts
            .iter()
            .find(|account| account.name == transaction.deposit_account);

        // If the account exists, deposit the amount to it
        match deposit_account {
            None => println!(">> Account {} not found", transaction.deposit_account),
            Some(account) => {
                println!(">> Depositing {} to {}", transaction.amount, account.name);

                // Update the balance
                *account.balance.borrow_mut() += transaction.amount;
            }
        }
    }

    // Sleep for 2 seconds
    //thread::sleep(std::time::Duration::from_secs(2));

    println!(">> Transaction {} Completed", transaction.id);
}

fn main() {
    // Create transactions to execute
    let transactions: [Transaction; 7] = [
        Transaction {
            id: 1,
            amount: 5.0,
            withdraw_account: String::from("A1"),
            deposit_account: String::from("A2"),
        },
        Transaction {
            id: 2,
            amount: 7.0,
            withdraw_account: String::from("A3"),
            deposit_account: String::from(""),
        },
        Transaction {
            id: 3,
            amount: 10.0,
            withdraw_account: String::from("A2"),
            deposit_account: String::from("A1"),
        },
        Transaction {
            id: 4,
            amount: 15.0,
            withdraw_account: String::from("A1"),
            deposit_account: String::from("A3"),
        },
        Transaction {
            id: 5,
            amount: 8.0,
            withdraw_account: String::from("A1"),
            deposit_account: String::from("A3"),
        },
        Transaction {
            id: 6,
            amount: 3.0,
            withdraw_account: String::from("A2"),
            deposit_account: String::from("A1"),
        },
        Transaction {
            id: 7,
            amount: 6.0,
            withdraw_account: String::from("A3"),
            deposit_account: String::from("A1"),
        },
    ];
    // Store the length (because of some memory moving shenanigans in the while loop)
    let transactions_length = transactions.len();

    // Create an array of 10 accounts, all starting at 0.0
    let accounts: [Account; 3] = [
        Account {
            name: String::from("A1"),
            balance: RefCell::new(0.0),
        },
        Account {
            name: String::from("A2"),
            balance: RefCell::new(0.0),
        },
        Account {
            name: String::from("A3"),
            balance: RefCell::new(0.0),
        },
    ];

    // Create an atomic reference counter for the array of accounts with a Mutex lock
    let accounts_reference = Arc::new(Mutex::new(accounts));
    // Create an atomic reference counter for the array of transactions with a Mutex lock
    let transactions_reference = Arc::new(Mutex::new(transactions));

    // Keep track of the threads created.
    let mut handles: Vec<JoinHandle<()>> = vec![];

    // Execute each transaction in a separate thread, with a max of 2 threads.
    let mut iterator = 0;
    while iterator < transactions_length {
        // Create a pointer to a reference to accounts_reference in the heap.
        let accounts_reference = Arc::clone(&accounts_reference);
        // Create a pointer to a reference to transactions_reference in the heap.
        let transactions_reference = Arc::clone(&transactions_reference);

        // Spawn a new thread.
        let handle = thread::spawn(move || {
            println!("Spawning thread {}", iterator + 1);

            // Get a mutex lock on the accounts_pointer (released when out of scope)
            let accounts = accounts_reference.lock().unwrap();
            // Get a mutex lock on the transactions_pointer (released when out of scope)
            let transactions = transactions_reference.lock().unwrap();

            println!("Executing Thread {}", iterator + 1);

            // Get the transaction
            let transaction = &transactions[iterator];

            // Execute the transaction
            execute_transaction(transaction, accounts.deref());

            println!("Thread {} Completed", iterator + 1);
        });

        // Increment loop
        iterator += 1;

        // Add the handle to the list of handles.
        handles.push(handle);
    }

    // Wait for threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Print the final balances
    println!("\nFinal Balances:");
    for account in accounts_reference.lock().unwrap().deref() {
        println!("{}: {}", account.name, account.balance.borrow());
    }
}
