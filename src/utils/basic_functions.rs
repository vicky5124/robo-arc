// Capitalizes the first letter of a str.
pub fn capitalize_first(input: &str) -> String {
    // get an array of all the charactesr on the string
    let mut c = input.chars();
    match c.next() {
        // return nothing if the string was empty.
        None => String::new(),
        // capitalize the first character item of the array and collect the array into a string.
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}
