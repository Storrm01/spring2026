fn most_frequent_word(text: &str) -> (String, usize){
    let words: Vec<&str> = text.split_whitespace().collect(); // turn the spaces into nothing, seeing only words and separating them to vectors

    let mut max_count: usize = 0;
    let mut max_word: &str = "";

    for i in 0..words.len(){
        let mut count = 0;

        for j in 0..words.len(){
            if words[i] == words[j]{ // checks amount of words per set of text
                count += 1; // counts upwards per text, so if "the" "the" "the" then it updates it every time
            }
        }

        if count > max_count{
            max_count = count;
            max_word = words[i]; // this replaces the leading word with the newest leading word if that is happening
        }
    }

    (max_word.to_string(), max_count)
}

fn main() { // this is taken from the homework text
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count); // the '{}' '{}' gets replaced with the word/count
}
