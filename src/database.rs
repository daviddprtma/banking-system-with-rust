// SPDX-License-Identifier: Unlicense

use std::path::PathBuf;

use crate::luhn::AccountNumber;

use rand::prelude::*;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Account {
    pub id: u64,
    pub account_number: String,
    pub balance: usize,
    pub pin: String,
}

impl Account {
    pub fn new() -> Result<Self> {
        let account_number = AccountNumber::default();
        let db = initialise_bankdb()?;
        let query_string = format!(
            "SELECT 1 FROM account where account_number='{}';",
            account_number
        );

        

        let account = create_account(&account_number, 0)?;
        Ok(account)
    }
}

#[cfg(not(test))]
fn database_path() -> PathBuf {
	PathBuf::from("bank.s3db")
}

#[cfg(test)]
fn database_path() -> PathBuf {
	PathBuf::from("mock_bank.s3db")
}

pub fn initialise_bankdb() -> Result<Connection> {
    let db = Connection::open(database_path())?;

    let command = "CREATE TABLE IF NOT EXISTS account(
id INTEGER PRIMARY KEY,
account_number TEXT,
pin TEXT DEFAULT '000000',
balance INTEGER DEFAULT 0
)
";

    db.execute(command, ())?;
    Ok(db)
}

// create_account
pub fn create_account(data:&AccountNumber, balance:u64) -> Result<Account> {
    let db = initialise_bankdb()?;
    let account_number = data.to_string();
    let pin = format!("{:06}", rand::thread_rng().gen_range(0..999999));
    let balance = balance;

    db.execute(
        "INSERT INTO account (account_number, pin, balance) VALUES (?1, ?2, ?3)",
        (&account_number, &pin, &balance),
    )?;

    let id = db.last_insert_rowid();
    Ok(Account {
        id: id as u64,
        account_number,
        balance: balance as usize,
        pin,
    })
}
pub fn deposit(amount: &str, pin: &str, account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        db.execute(
            "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
            (amount, account_number),
        )?;

        let query_string = format!(
            "SELECT balance FROM account where account_number='{}';",
            account_number
        );

        let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

        println!(
            "The account number `{}` now has a balance of `{}`.\n",
            &account_number, &amount_from_db
        );
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}
pub fn transfer(
	amount: &str,
	pin: &str,
	origin_account: &str,
	target_account: &str,
) -> Result<(Account, Account)> {
	if *origin_account == *target_account {
    	return Err(rusqlite::Error::QueryReturnedNoRows); // Makes sense. We haven't returned any.
	}

	// Create new binding
	let origin_account = fetch_account(origin_account)?;
	let target_account = fetch_account(target_account)?;

	let correct_pin = origin_account.pin == pin;

	if correct_pin {

    	let amount = amount
        	.parse::<u64>().map_err(|_| {
            	rusqlite::Error::QueryReturnedNoRows
        	})?;

    	if amount > origin_account.balance as u64 {
    	} else {
        	let db = initialise_bankdb()?;
        	// Add money to account 2
        	db.execute(
            	"UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
            	(amount, &target_account.account_number),
        	)?;

        	// Subtract money from account 1
        	db.execute(
            	"UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
            	(amount, &origin_account.account_number),
        	)?;

    	};
	} else {
    	return Err(rusqlite::Error::QueryReturnedNoRows);
	}

	let origin_account = fetch_account(&origin_account.account_number)?;
	let target_account = fetch_account(&target_account.account_number)?;

	Ok((origin_account, target_account))
}
pub fn withdraw(amount: &str, pin: &str, account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        let query_string = format!(
            "SELECT balance FROM account where account_number='{}';",
            account_number
        );

        let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

        println!(
            "The account number `{}` has a balance of `{}`.\n",
            &account_number, &amount_from_db
        );

        let amount = amount
            .parse::<usize>()
            .expect("Not able to parse string to usize");

        if amount > amount_from_db {
            eprintln!(
                "You are trying to withdraw that exceeds your current deposit... aborting...\n"
            );
        } else {
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                (amount, account_number),
            )?;

            let query_string = format!(
                "SELECT balance FROM account where account_number='{}';",
                account_number
            );

            let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                &account_number, &amount_from_db
            );
        };
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}
pub fn delete_account(account_number: &str, pin: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        &account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;
    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        db.execute(
            "DELETE FROM account WHERE account_number=?1",
            (account_number,),
        )?;
        println!("DELETED ACCOUNT: {}", &account_number);
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}
pub fn show_balance(account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT balance FROM account where account_number='{}';",
        account_number
    );

    let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

    println!(
        "The account number `{}` now has a balance of `{}`.\n",
        &account_number, &amount_from_db
    );
    Ok(())
}

fn fetch_account(account: &str) -> Result<Account> {
	let db = initialise_bankdb()?;
	let mut stmt = db.prepare("SELECT id, account_number, balance, pin FROM account")?;
	let accounts = stmt.query_map([], |row| {
    	Ok(Account {
        	id: row.get(0)?,
        	account_number: row.get(1)?,
        	balance: row.get(2)?,
        	pin: row.get(3)?,
    	})
	})?;

	let accounts = accounts.flatten().find(|acc| acc.account_number == account);
	if let Some(fetched_account) = accounts {
    		Ok(fetched_account)
	} else {
    		Err(rusqlite::Error::QueryReturnedNoRows)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
	fn transferred_balance_is_correct() -> Result<()> {
		// 1) Fill the missing code here
		let deposit_balance = "10000";
        let origin_account = Account::new()?;
        let target_account = Account::new()?;

		
        // let's deposit first
    	deposit(deposit_balance, &origin_account.pin, &origin_account.account_number)?;
        
		// 2) Fill the missing code here
        transfer(deposit_balance, &origin_account.pin, &origin_account.account_number, &target_account.account_number)?;

		// assert_eq!(*deposit_balance, origin_account.balance.to_string());

		// 3) Fill the missing code here
        let origin_account = fetch_account(&origin_account.account_number)?;
        let target_account = fetch_account(&target_account.account_number)?;

        assert_eq!("0".to_string(), origin_account.balance.to_string());

    	assert_eq!(deposit_balance.to_owned(), target_account.balance.to_string());
		// Nothing further here
		Ok(())
	}

    #[test]
	fn created_account_is_correct_fetched_from_db() -> Result<()> {
    		let acc1 = Account::new()?;
    		let acc2 = fetch_account(&acc1.account_number)?;

    		assert_eq!(acc1.id, acc2.id);

    		Ok(())
	}

}

	