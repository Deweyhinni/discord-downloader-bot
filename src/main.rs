use std::{env, error::Error, path::PathBuf, str::FromStr};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use youtube_dl::{download_yt_dlp, YoutubeDl};

use url::Url;

async fn download_video(url: String, save_path: &str, yt_dlp_path: PathBuf) -> Result<(), youtube_dl::Error> {
    let output = YoutubeDl::new(url)
        .youtube_dl_path(yt_dlp_path)
        .download_to_async(save_path)
        .await;

    output
}

struct Handler {
    channel: u64,
    down_dir: &'static str,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("message stuff");
        if msg.channel_id == serenity::all::ChannelId::new(self.channel) && !msg.author.bot {
            let response: String;
            let _issue_list = match Url::parse(msg.content.as_str()) {
                Ok(_) => {
                    if let Err(e) = download_video(msg.content, self.down_dir, download_yt_dlp(".").await.unwrap()).await {
                        println!("error downloading video {:?}", e);
                        response = format!("error downloading video {:?}", e).to_string();
                    } else {
                        println!("succesfully downloaded video");
                        response = "succesfully downloaded video".to_string();
                    }
                },
                Err(parse_err) => {
                    response = format!("error with url: {:?}", parse_err);
                }
            };
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN").expect("token not found in env");
    let channel_id: u64 = env::var("BOT_CHANNEL_ID").expect("channel id not found in env").parse().expect("channel id variable not valid");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(&token, intents).event_handler(Handler {channel: channel_id, down_dir: "./downloads"}).await.expect("error creating client");
    
    if let Err(e) = client.start().await {
        println!("client error: {e:?}");
    }

    Ok(())
}
