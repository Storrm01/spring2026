use std::fs::File;
use std::io::{Write, BufReader, BufRead};

struct Book {
    title: String,
    author: String,
    year: u16,
}

fn save_books(books: &Vec<Book>, filename: &str) {
    // TODO: Implement this function
    let mut file = File::create(filename).expect("Unable to create file");
    // Hint: Use File::create() and write!() macro
    for book in books{
        writeln!(file, "{},{},{}", book.title, book.author, book.year)
            .expect("Unable to write to file");
    }
}

fn load_books(filename: &str) -> Vec<Book> {
    // TODO: Implement this function
    let file = File::open(filename).expect("Unable to open file");
    let reader = BufReader::new(file);

    let mut books: Vec<Book> = Vec::new();
    // Hint: Use File::open() and BufReader
    for line in reader.lines(){
        let line = line.expect("Unable to read line");

        let parts: Vec<&str> = line.split(',').collect();

        let book = Book{
            title: parts[0].to_string(),
            author: parts[1].to_string(),
            year: parts[2].parse::<u16>().unwrap(),
        };
        books.push(book);
    }
    books
}

fn main() {
    let books = vec![
        Book { title: "1984".to_string(), author: "George Orwell".to_string(), year: 1949 },
        Book { title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string(), year: 1960 },
    ];

    save_books(&books, "books.txt");
    println!("Books saved to file.");

    let loaded_books = load_books("books.txt");
    println!("Loaded books:");
    for book in loaded_books {
        println!("{} by {}, published in {}", book.title, book.author, book.year);
    }
}