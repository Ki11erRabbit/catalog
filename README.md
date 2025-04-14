# Catalog
A simple cataloging app for the home cataloger written in Rust.



## How to Run
First install Rust. The easiest way is to use [rustup](https://www.rust-lang.org/tools/install).

`git clone https://github.com/Ki11erRabbit/catalog`

`cargo run --release`

## How to use
Create a database with the `.sqlite` extension by either typing in an absolute path or by using the file picker to create a new database.

Then hit create/open once you have a database to access it.

Then add some items using the add tab of the application.

Things are stored in this heirarchy: `Rack -> Shelf -> Basket`

You can then search for the exact name of the item in the search tab and it will tell you exactly where you put the item.
