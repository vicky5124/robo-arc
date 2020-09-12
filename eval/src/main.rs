
#![allow(unused_variables)]
#![allow(redundant_semicolons)]
use std::error::Error;

use twilight_http::Client;
use twilight_model::channel::message::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::new("NjAxNzQ5NTEyNDU2ODk2NTIy.XTG1lA.00vqYftnL89yDkYuecz-KLUZv3s");
    let ctx = &client;

    let msg: Message = serde_json::from_str(r####"{"id":"754359297269301258","type":0,"content":",eval ```\nuse twilight_mention::Mention;\n\nuse twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};\n\nlet embed = EmbedBuilder::new()\n    .description(\"Here's a list of reasons why Twilight is the best pony:\")?\n    .field(EmbedFieldBuilder::new(\"Wings\", \"She has wings.\")?.inline())\n    .field(EmbedFieldBuilder::new(\"Horn\", \"She can do magic, and she's really good at it.\")?.inline())\n    .build()?;\n\nclient.create_message(msg.channel_id)\n    .content(msg.author.mention().to_string())?\n    .embed(embed)?\n    .await?;```","channel_id":"702161938012045383","author":{"id":"182891574139682816","username":"nitsuga5124","avatar":"3edfc6c4dccfecb12986b2bdfc744e7a","discriminator":"2207","bot":false},"attachments":[],"embeds":[],"pinned":false,"mention_everyone":false,"tts":false,"timestamp":"2020-09-12T15:14:31.787Z","flags":0,"mention_roles":[],"mentions":[],"edited_timestamp":null}"####)?;


use twilight_mention::Mention;

use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

let embed = EmbedBuilder::new()
    .description("Here's a list of reasons why Twilight is the best pony:")?
    .field(EmbedFieldBuilder::new("Wings", "She has wings.")?.inline())
    .field(EmbedFieldBuilder::new("Horn", "She can do magic, and she's really good at it.")?.inline())
    .build()?;

client.create_message(msg.channel_id)
    .content(msg.author.mention().to_string())?
    .embed(embed)?
    .await?;;

    Ok(())
}
