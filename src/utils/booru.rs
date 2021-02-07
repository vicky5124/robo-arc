use rand::Rng;
use serenity::framework::standard::Args;

pub static SAFE_BANLIST: [&str; 12] = [
    "swastika",
    "gore",
    "guro",
    "smoking",
    "jailbait",
    "extreme_content",
    "extremely_large_filesize",
    "pussy",
    "dick",
    "nude",
    "partial_nude",
    "tagme",
];

pub static UNSAFE_BANLIST: [&str; 17] = [
    "loli",
    "lolicon",
    "shota",
    "shotacon",
    "swastika",
    "gore",
    "guro",
    "smoking",
    "underage",
    "underaged",
    "jailbait",
    "extreme_content",
    "extremely_large_filesize",
    "contentious_content",
    "cub",
    "young",
    "tagme",
];

// This function parses the arguments on the booru commands and returns a list of the tags.
pub async fn obtain_tags_unsafe(raw_args: Args) -> Vec<String> {
    // transform the arguments into a Vec<&str> for easier management.
    let args = raw_args.raw_quoted().collect::<Vec<&str>>();
    let mut tags = Vec::new();

    // Iterate over every argument of the command.
    for arg in args {
        // if the arg equals to any of the predefined flags, replace the flag with what it's meant.
        match arg {
            // For only NSFW content.
            "-x" | "nsfw" => tags.push("rating:Explicit"),
            // For only BSFW content.
            "-q" | "bsfw" => tags.push("rating:Questionable"),
            // For only SFW content.
            "-s" | "sfw" => tags.push("rating:Safe"),
            // For either NSFW or BSFW content. aka non-safe
            // > "but nitsu, you can search both tags and you will get results from both"
            // > Ik, but this will make it so some boorus won't be able to be used with any other
            // additional tags, like danbooru.
            "-n" => {
                // basically choose a random item from the list.
                let choices = ["rating:Questionable", "rating:Explicit"];
                let r = rand::thread_rng().gen_range(0..choices.len());
                let choice = choices[r];
                // and push that random item to the tags.
                tags.push(choice)
            }
            // Every other tag that doesn't match any of the flags will be passed as is.
            _ => tags.push(arg),
        }
    }
    // convert the tags vector type from Vec<&str> to Vec<String>
    tags.iter().map(|x| (*x).to_string()).collect()
}

// This function parses the arguments for safe content on booru commands and returns a lit of the tags.
pub async fn obtain_tags_safe(raw_args: Args) -> Vec<String> {
    // transform the arguments into a Vec<&str> for easier management.
    let args = raw_args.raw_quoted().collect::<Vec<&str>>();
    // Since this will only allow safe tags, we add rating:Safe to the tags by default
    let mut tags = vec!["rating:Safe"];

    // Iterate over every argument
    for arg in args {
        // if the argument starts with "rating:", don't do anything.
        // we don't want to search for images other than safe rating.
        // we also block -rating: because you can't search "rating:Safe -rating:Sage" in most
        // boorus. It just crashes the search.
        if !arg.contains("rating:") {
            tags.push(arg);
        }
    }
    // transform trags into Vec<String>
    tags.iter().map(|x| (*x).to_string()).collect()
}

// This function removes any illegal tags for not SFW content from the tags.
pub async fn illegal_check_unsafe(tags: &mut Vec<String>) -> Vec<String> {
    // This is a list of tags that are banable from discord.
    // automatically remove them from the arguments if mentioned.
    let mut new_tags = Vec::new();

    // iterate over every tag
    for tag in tags {
        // and add them to a new list if they don't match any of the blacklisted tags.
        if !UNSAFE_BANLIST.contains(&tag.as_str()) {
            new_tags.push(tag.to_owned());
        }
    }
    new_tags
}

// This function removes any illegal tags for SFW contnet from the tags.
pub async fn illegal_check_safe(tags: &mut Vec<String>) -> Vec<String> {
    // This is a list of tags that are banable when sent outside nsfw channels.
    let mut new_tags = Vec::new();
    // Add the tags that don't match any of the blacklist tags to a new tags vector.
    for tag in tags {
        if !SAFE_BANLIST.contains(&tag.as_str()) {
            new_tags.push(tag.to_owned());
        }
    }
    new_tags
}
