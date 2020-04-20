use crate::ConnectionPool;
use std::{
    time::Duration,
    sync::Arc,
};
use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;
use serde::Deserialize;
use reqwest::Url;

use serenity::{
    prelude::Context,
    model::{
        id::ChannelId,
        channel::Embed,
    }
};

#[derive(Deserialize)]
pub struct Post {
    sample_url: String,
    pub md5: String,
    id: u64,
}

async fn check_new_posts(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<ConnectionPool>().unwrap();

    let mut data = sqlx::query!("SELECT * FROM new_posts")
        .fetch(pool)
        .boxed();


    while let Some(i) = data.try_next().await? {
        let base_url = i.booru_url;
        let tags = i.tags;
        let webhooks = i.webhook.unwrap_or(Vec::new());
        let channels = i.channel_id.unwrap_or(Vec::new());
        let mut md5s = i.sent_md5.unwrap_or(vec![]);

        if base_url == "yande.re" {
            let url = Url::parse_with_params("https://yande.re/post/index.json",
                                             &[("tags", &tags), ("limit", &"100".to_string())])?;
            let resp = reqwest::get(url)
                .await?
                .json::<Vec<Post>>()
                .await?;

            for post in resp {
                if !md5s.contains(&post.md5) {
                    for channel in &channels {
                        if let Err(why) = ChannelId(*channel as u64).send_message(&ctx, |m|{
                            m.embed(|e| {
                                e.title("Original Post");
                                e.url(format!("https://yande.re/post/show/{}", post.id));
                                e.image(post.sample_url.clone())
                            })
                        }).await {
                            eprintln!("Error while sending message >>> {}", why);
                        };
                    }

                    for webhook in &webhooks {
                        let mut split = webhook.split('/');
                        let id = split.nth(5).unwrap().parse::<u64>()?;
                        let token = split.nth(0).unwrap();

                        let hook = &ctx.http.get_webhook_with_token(id, token).await?;

                        let embed = Embed::fake(|e| {
                            e.title("Original Post");
                            e.url(format!("https://yande.re/post/show/{}", post.id));
                            e.image(post.sample_url.clone())
                        });
                        
                        hook.execute(&ctx.http, false, |m|{
                            m.embeds(vec![embed])
                        }).await?;
                    }

                    &md5s.push(post.md5);
                    sqlx::query!(
                        "UPDATE new_posts SET sent_md5 = $1 WHERE booru_url = $2 AND tags = $3",
                        &md5s, &base_url, &tags
                    ).execute(pool).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn notification_loop(ctx: Arc<Context>) {
    let ctx = Arc::new(ctx);
    loop {
        let ctx1 = Arc::clone(&ctx);
        tokio::spawn(async move {
            if let Err(why) = check_new_posts(Arc::clone(&ctx1)).await {
                eprintln!("An error occurred while running check_new_posts() >>> {}", why);
            }
        });

        //let ctx2 = Arc::clone(&ctx);
        //tokio::spawn(async move {
        //    if let Err(why) = check_new_posts(Arc::clone(&ctx2)).await {
        //        eprintln!("An error occurred while running check_new_posts() >>> {}", why);
        //    }
        //});

        tokio::time::delay_for(Duration::from_secs(120)).await;
    }
}

