
#![allow(unused_variables)]
#![allow(redundant_semicolons)]
use std::error::Error;

use twilight_http::Client;
use twilight_model::channel::message::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::new("NjAxNzQ5NTEyNDU2ODk2NTIy.XTG1lA.00vqYftnL89yDkYuecz-KLUZv3s");
    let ctx = &client;

    let msg: Message = serde_json::from_str(r####"{"id":758437729842364458,"attachments":[],"author":{"id":182891574139682816,"avatar":"3edfc6c4dccfecb12986b2bdfc744e7a","bot":false,"discriminator":2207,"username":"nitsuga5124"},"channel_id":702161938012045383,"content":",eval ```rs\nprintln!(\"test {}\", msg.author.name);\neprintln!(\"test\");\n```","edited_timestamp":null,"embeds":[],"guild_id":182892283111276544,"type":0,"member":{"deaf":false,"joined_at":"2016-05-19T16:28:38.271Z","mute":false,"nick":null,"roles":[182894738100322304,182894970720616452,182895142770966528,535751542800908288,671252603602075648,721022481745444915,721096287419760680]},"mention_everyone":false,"mention_roles":[],"mention_channels":null,"mentions":[],"nonce":"758437729468940288","pinned":false,"reactions":[],"timestamp":"2020-09-23T21:20:45.886Z","tts":false,"webhook_id":null,"activity":null,"application":null,"message_reference":null,"flags":0}"####)?;


println!("test {}", msg.author.name);
eprintln!("test");
;

    Ok(())
}
