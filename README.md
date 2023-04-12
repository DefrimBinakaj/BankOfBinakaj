This program is a "bank" program where the user
may create an account and later sign in to see their
balance as well as transfer money to other users 
in the database

password are hashed using argon2

program instructions:

to create an account:
cargo run new [name] [password]

to transfer balance to a different user:
cargo run transfer [payer] [payee]

to see your balance:
cargo run balance [name]
