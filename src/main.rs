#![allow(non_snake_case, unused_variables, unused_imports, dead_code, unused_parens)]

use bcrypt::{DEFAULT_COST, hash, verify, BcryptError};

use sqlite::Error as SqErr;

pub struct UserBase {
    fname: String,
}

#[derive(Debug)]
pub enum UBaseErr {
    DbErr(SqErr),
    HashError(BcryptError),
}

impl From<SqErr> for UBaseErr{
    fn from(s:SqErr)->Self{
        UBaseErr::DbErr(s)
    }
}

impl From<BcryptError> for UBaseErr{
    fn from(b:BcryptError)->Self{
        UBaseErr::HashError(b)
    }
}

impl UserBase {

    pub fn add_user(&self, u_name:&str, p_word:&str, balance:i64)->Result<(),UBaseErr>{

        let conn=sqlite::open(&self.fname)?;

        let hpassArgon = argon2::hash_encoded(p_word.as_bytes(), b"randSalt", &argon2::Config::default()).unwrap();
        
        let mut st= conn.prepare("insert into users(u_name, p_word, balance) values (?,?, ?);")?;
        st.bind(1,u_name)?;
        st.bind(2,&hpassArgon as &str)?;
        st.bind(3,balance)?;
        st.next()?;
        Ok(())
    }
    
    pub fn pay(&self, u_from:&str, u_to:&str, amount:i64)->Result<(),UBaseErr> {

        let conn = sqlite::open(&self.fname)?;
        
        let mut checkBal = conn.prepare("select * from users")?;

        let mut isValid = 0;

        while let Ok(sqlite::State::Row) = checkBal.next() {
            // println!("u_from = {}", checkBal.read::<String>(0).unwrap());
            // println!("amount = {}", checkBal.read::<i64>(2).unwrap());
            if (checkBal.read::<String>(0).unwrap() == u_from
            && checkBal.read::<i64>(2).unwrap() >= amount) {
                isValid = 1;
            }
        }

        if (isValid == 1) {
            let mut st = conn.prepare("insert into transactions (u_from, u_to, t_date, t_amount)
            values (?,?,datetime(\"now\"),?);")?;
        
            st.bind(1,u_from)?;
            st.bind(2,u_to)?;
            st.bind(3,amount)?;
            st.next()?;
            println!("Operation done succesfully!");
            Ok(())
        }
        else {
            println!("Insufficient funds for the transfer");
            Ok(())
        }

            
    }

    pub fn get_transactions_history(&self, u_name:&str) -> Result<(), UBaseErr> {
        let conn = sqlite::open(&self.fname)?;
        let mut st = conn.prepare("select * from transactions where u_from=? or u_to=?;")?;
        st.bind(1,u_name)?;
        st.bind(2,u_name)?;
        while let Ok(sqlite::State::Row) = st.next() {
            let u_from: String = st.read(0)?;
            let u_to: String = st.read(1)?;
            let t_date: String = st.read(2)?;
            let t_amount: i64 = st.read(3)?;
            println!("{} received ${} from {} on {}", u_to, t_amount, u_from, t_date);
        }
        Ok(())
    }

    pub fn get_balance(&self, u_name:&str) -> Result<(), UBaseErr> {
        let conn = sqlite::open(&self.fname)?;
        let mut st = conn.prepare("select * from users where u_name=?;")?;
        st.bind(1,u_name)?;
        while let Ok(sqlite::State::Row) = st.next() {
            let u_name: String = st.read(0)?;
            let balance: i64 = st.read(2)?;
            println!("Balance is ${}", balance);
        }
        Ok(())
    }

}


fn main() {
    // https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html
    let args: Vec<String> = std::env::args().collect();
    println!("---------------------");
    println!("Bank of Binakaj");
    println!("---------------------");
    let userbase = UserBase { fname: "data/users.db".into() };
    let connection = sqlite::open("data/users.db").unwrap();
    
    if args.len() <= 1 {
        println!("Please enter a valid command such as:");
        println!("> cargo run new [name] [password]");
        println!("> cargo run transfer [payer] [payee]");
        println!("> cargo run balance [name]");
        println!(" ");
    }
    else {
        let cmd = &args[1];
        if (cmd == "new" && args.len() == 4) {
            let u_name = &args[2];
            let p_word = &args[3];
            let balance = "500";
            let balance = balance.parse::<i64>().unwrap();
            println!("Adding user {} with password {}...", u_name, p_word);
            userbase.add_user(u_name, p_word, balance).unwrap();
            println!("Operation done succesfully!");
        }
        else if (cmd == "transfer" && args.len() == 5) {
    
            let u_from = &args[2];
            let u_to = &args[3];
            let amount = &args[4];
            let amount = amount.parse::<i64>().unwrap();
    
    
            // input
            let mut inputRaw = String::new();
            println!("Please input your password:");
            let b1 = std::io::stdin().read_line(&mut inputRaw);
            let input = inputRaw.trim();
    
    
            let mut statement = connection.prepare("select * from users").unwrap();
    
            let mut isValid = 0;
    
            while let Ok(sqlite::State::Row) = statement.next() {
                if (&statement.read::<String>(0).unwrap() == u_from
                && argon2::verify_encoded(&statement.read::<String>(1).unwrap(), input.as_bytes()).unwrap()) {
                    isValid = 1;
                }
            }
    
    
    
            if (isValid == 1) {
                println!("Sending money from {} to {}...", u_from, u_to);
                userbase.pay(u_from, u_to, amount).unwrap();
            }
            else {
                println!("Incorrect username/password - quitting program");
            }
    
    
        }
        else if (cmd == "balance" && args.len() == 3) {
    
            let u_name = &args[2];
    
            // input
            let mut inputRaw = String::new();
            println!("Please input your password:");
            let b1 = std::io::stdin().read_line(&mut inputRaw);
            let input = inputRaw.trim();
    
    
            let mut statement = connection.prepare("select * from users").unwrap();
    
            let mut isValid = 0;
    
            while let Ok(sqlite::State::Row) = statement.next() {
                if (&statement.read::<String>(0).unwrap() == u_name
                && argon2::verify_encoded(&statement.read::<String>(1).unwrap(), input.as_bytes()).unwrap()) {
                    isValid = 1;
                }
            }
    
    
    
            if (isValid == 1) {
                println!("Getting balance for {}...", u_name);
                userbase.get_balance(u_name).unwrap();
            }
            else {
                println!("Incorrect username/password - quitting program");
            }
    
        }
        else {
            println!("Invalid command - please run again.");
        }
    }


}

#[cfg(test)]
mod test {
    use chrono::Utc;
    use sqlite::{Connection, State};
    use super::*;

    fn setup() -> (Connection, UserBase) {
        let connection = sqlite::open("data/users.db").unwrap();
        connection
            .execute(
                r#"
                DROP TABLE IF EXISTS users;
                DROP TABLE IF EXISTS transactions;
                create table users(u_name text PRIMARY KEY, p_word text, balance text);
                create table transactions(u_from text, u_to text, t_date integer, t_amount
                text, PRIMARY KEY(u_from,t_date), FOREIGN KEY (u_from) REFERENCES users(u_name),
                FOREIGN KEY (u_to) REFERENCES users(u_name));"#,
            )
            .unwrap();

        (connection, UserBase { fname: "data/users.db".into() })
    }


    // a)
    #[test]
    fn get_transactions_history() {
        let (connection, userbase) = setup();

        userbase.add_user("Matt", "1", 200).unwrap();
        userbase.add_user("Jim", "2", 800).unwrap();
        userbase.add_user("Andrew", "3", 350).unwrap();


        userbase.pay("Jim", "Matt", 140).unwrap();
        userbase.pay("Andrew", "Matt", 100).unwrap();

        userbase.get_transactions_history("Matt").unwrap();

    }

    // b)
    #[test]
    fn pay_balance() {
        let (connection, userbase) = setup();

        userbase.add_user("Matt", "1", 200).unwrap();
        userbase.add_user("Jim", "2", 80).unwrap();
        userbase.add_user("Andrew", "3", 350).unwrap();

        userbase.pay("Jim", "Matt", 140).unwrap();
        userbase.pay("Andrew", "Matt", 100).unwrap();

    }


}



