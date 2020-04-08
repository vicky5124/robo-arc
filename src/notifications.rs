use crate::DatabaseConnection;
use std::{
    time::Duration,
    sync::Arc,
};
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
struct Post {
    sample_url: String,
    md5: String,
}

async fn check_new_posts(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let data_read = ctx.data.read().await;
    let db = data_read.get::<DatabaseConnection>().unwrap();

    let data ={
        let client = db.write().await;
        client.query("SELECT * FROM new_posts",
                     &[]).await?
    };

    for i in data {
        let base_url = i.get::<_, &str>(0);
        let tags = i.get::<_, &str>(1);
        let webhooks = i.get::<_, Option<Vec<&str>>>(2).unwrap_or(Vec::new());
        let channels = i.get::<_, Option<Vec<i64>>>(3).unwrap_or(Vec::new());
        let mut md5s = i.get::<_, Option<Vec<String>>>(4).unwrap();

        if base_url == "yande.re" {
            let url = Url::parse_with_params("https://yande.re/post/index.json",
                                             &[("tags", &tags), ("limit", &"100")])?;
            let resp = reqwest::get(url)
                .await?
                .json::<Vec<Post>>()
                .await?;

            for post in resp {
                if !md5s.contains(&post.md5) {
                    for channel in &channels {
                        ChannelId(*channel as u64).send_message(&ctx, |m|{
                            m.embed(|e| {
                                e.image(post.sample_url.clone())
                            })
                        }).await?;
                    }

                    for webhook in &webhooks {
                        let mut split = webhook.split('/');
                        let id = split.nth(5).unwrap().parse::<u64>()?;
                        let token = split.nth(0).unwrap();

                        let hook = &ctx.http.get_webhook_with_token(id, token).await?;

                        let embed = Embed::fake(|e| {
                            e.image(post.sample_url.clone())
                        });
                        
                        hook.execute(&ctx.http, false, |m|{
                            m.embeds(vec![embed])
                        }).await?;
                    }

                    &md5s.push(post.md5);
                    {
                        let client = db.write().await;
                        client.execute(
                            "UPDATE new_posts SET sent_md5 = $1 WHERE booru_url = $2 AND tags = $3",
                            &[&Some(&md5s), &base_url, &tags]
                        ).await?;
                    }
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

