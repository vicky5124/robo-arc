use regex::Regex;
use std::cell::LazyCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct PasteData {
    code: String,
    upload: String,
    extension: String,
}

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
    let re = LazyCell::new(|| {
        Regex::new(r"((?P<years>\d+?)Y|years)?((?P<months>\d+?)M|months)?((?P<weeks>\d+?)W|weeks)?((?P<days>\d+?)D|days)?((?P<hours>\d+?)h|hr|hours)?((?P<minutes>\d+?)m|min|minutes)?((?P<seconds>\d+?)s|sec|seconds)?").unwrap()
    });
    let text = &text.to_string();
    let captures = re.captures(text);
    let caps = if let Some(caps) = captures {
        caps
    } else {
        return 0;
    };

    let mut seconds = 0;
    for name in [
        "years", "months", "weeks", "days", "hours", "minutes", "seconds",
    ] {
        if let Some(time) = caps.name(name) {
            let time: u64 = time.as_str().parse().unwrap();
            seconds += match name {
                "years" => time * 31_557_600, // 365.25 days (.25 to take leap years into account)
                "months" => time * 2_592_000, // 30 days
                "weeks" => time * 604_800,
                "days" => time * 86_400,
                "hours" => time * 3_600,
                "minutes" => time * 60,
                "seconds" => time,
                _ => 0,
            };
        } else {
            continue;
        }
    }

    seconds
}

pub async fn create_paste(code: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let upload_data = PasteData {
        code: code.to_string(),
        upload: "paste".to_string(),
        extension: String::new(),
    };

    let response = client
        .post("https://5124.16-b.it:8088/paste")
        .form(&upload_data)
        .send()
        .await?;

    if response.status() != 200 {
        Ok(String::new())
    } else {
        Ok(response.url().to_string())
    }
}
