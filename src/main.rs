use std::{env, error::Error, fs::File, io::Write, path::PathBuf, str::FromStr};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use youtube_dl::{download_yt_dlp, YoutubeDl};

use url::Url;

async fn download_video(
    url: String,
    save_path: &str,
    yt_dlp_path: PathBuf,
) -> Result<(), youtube_dl::Error> {
    let output = YoutubeDl::new(url)
        .youtube_dl_path(yt_dlp_path)
        .download_to_async(save_path)
        .await;

    output
}

struct Handler {
    channel: u64,
    down_dir: &'static str,
    yt_dlp_path: PathBuf,
}

impl Handler {
    async fn dl_attachments(
        &self,
        msg: &Message,
    ) -> Result<Option<()>, Box<dyn std::error::Error>> {
        let mut return_thing = None;
        for attachment in msg.attachments.iter() {
            println!("attachment found");
            let mut file = File::create(format!("{}/{}", self.down_dir, attachment.filename))?;
            file.write(attachment.download().await?.as_slice())?;
            file.flush()?;
            return_thing = Some(());
        }

        Ok(return_thing)
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("message stuff");
        if msg.channel_id == serenity::all::ChannelId::new(self.channel) && !msg.author.bot {
            let mut response: String = "nothing to do".to_string();
            if msg.content.len() > 0 {
                let _issue_list = match Url::parse(msg.content.as_str()) {
                    Ok(url) => {
                        if let Err(e) =
                            download_video(url.to_string(), self.down_dir, self.yt_dlp_path.clone())
                                .await
                        {
                            println!("error downloading video {:?}", e);
                            response = format!("error downloading video {:?}", e).to_string();
                        } else {
                            println!("{}", url.to_string());
                            println!("succesfully downloaded video");
                            response = "succesfully downloaded video".to_string();
                        }
                    }
                    Err(parse_err) => {
                        response = format!("error with url: {:?}", parse_err);
                    }
                };
            }
            match self.dl_attachments(&msg).await {
                Ok(Some(())) => {
                    response = "succesfully downloaded attachment".to_string();
                }
                Ok(None) => (),
                Err(error) => {
                    response = format!("failed to download attachments: {error}");
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
    let channel_id: u64 = env::var("BOT_CHANNEL_ID")
        .expect("channel id not found in env")
        .parse()
        .expect("channel id variable not valid");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            channel: channel_id,
            down_dir: "./downloads",
            yt_dlp_path: download_yt_dlp(".")
                .await
                .expect("unable to download yt-dlp"),
        })
        .await
        .expect("error creating client");

    if let Err(e) = client.start().await {
        println!("client error: {e:?}");
    }

    Ok(())
}
