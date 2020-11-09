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

pub fn pacman(value: &str) -> String {
    let mod_value = value.to_owned() + "000";
    let x = mod_value.split('.').nth(1).unwrap()[..3]
        .parse::<u32>()
        .unwrap();
    let tm = x / 50;

    let mut s = "".to_string();

    for _ in 0..tm {
        s += ". ";
    }
    s += "C ";

    for _ in tm..20 {
        s += "o ";
    }
    s
}

pub fn seconds_to_days(seconds: u64) -> String {
    let days = seconds / 60 / 60 / 24;
    let hours = seconds / 3600 % 24;
    let minutes = seconds % 3600 / 60;
    let sec = seconds % 3600 % 60;

    if days == 0 {
        format!("{}:{:02}:{:02}", hours, minutes, sec)
    } else {
        format!("{}D {}:{:02}:{:02}", days, hours, minutes, sec)
    }
}

pub fn string_to_seconds(text: impl ToString) -> u64 {
    let s = text.to_string();
    let words = s.split(' ');
    let mut seconds = 0;

    for i in words {
        if i.ends_with("s") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0);
        }
        if i.ends_with("m") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 60;
        }
        if i.ends_with("h") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 3600;
        }
        if i.ends_with("D") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 86_400;
        }
        if i.ends_with("W") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 604_800;
        }
        if i.ends_with("M") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 2_628_288;
        }
        if i.ends_with("Y") {
            let num = &i[..i.len() - 1];
            seconds += num.parse::<u64>().unwrap_or(0) * 31_536_000;
        }
    }

    seconds
}
