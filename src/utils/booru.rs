use serenity::framework::standard::Args;
use rand::Rng;

pub fn obtain_tags_unsafe(raw_args: Args) -> Vec<String> {
    let args = raw_args.raw_quoted().collect::<Vec<&str>>();
    let mut tags = Vec::new();

    for arg in args {
        match arg {
            "-x" => tags.push("rating:Explicit"),
            "-q" => tags.push("rating:Questionable"),
            "-s" => tags.push("rating:Safe"),
            "-n" => {
                let choices = ["rating:Questionable", "rating:Explicit"];
                let r = rand::thread_rng().gen_range(0, choices.len());
                let choice = choices[r];
                tags.push(choice)
            },
            _ => tags.push(arg),
        }
    }
    tags.iter().map(|x| x.to_string()).collect()
}

pub fn obtain_tags_safe(raw_args: Args) -> Vec<String> {
    let args = raw_args.raw_quoted().collect::<Vec<&str>>();
    let mut tags = vec!["rating:Safe"];

    for arg in args {
        if !arg.starts_with("rating:") {
            &tags.push(arg);
        }
    }
    tags.iter().map(|x| x.to_string()).collect()
}

pub fn illegal_check(tags: &mut Vec<String>) -> Vec<String> {
    let banlist = vec!["loli", "lolicon", "shota", "shotacon", "swastika", "gore", "guro", "smoking", "underage", "underaged", "jailbait"];
    let mut new_tags = Vec::new();
    for tag in tags{
        if !banlist.contains(&&tag[..]) {
            new_tags.push(tag.to_owned());
        }
    }
    new_tags
}
