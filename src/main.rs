use std::{env, time::SystemTime};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use magick_rust::{magick_wand_genesis, MagickWand, PixelWand};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.mentions_user_id(540791795693649921) {
            return;
        }

        if msg.attachments.len() == 0 {
            return send_message(&ctx, &msg, "what you want").await;
        }

        /*
        if msg.attachments.len() > 1 {
            return send_message(&ctx, &msg, "too many attachments").await;
        }
        */

        if msg.content.split_whitespace().count() <= 1 {
            return send_message(&ctx, &msg, "what?").await;
        }
        println!("{}: {}", msg.author.name, msg.content);

        send_message(&ctx, &msg, "starting convert...").await;
        let time = SystemTime::now();

        for attach in &msg.attachments {
            let mut data = reqwest::get(&attach.url).await.unwrap().bytes().await.unwrap().to_vec();
            if !infer::is_image(&data) {
                return send_message(&ctx, &msg, "not an image").await;
            }

            data = convert_image(&msg, &data).await;
            send_file(&ctx, &msg, None, data).await;
        }
        send_message(&ctx, &msg, &format!("response took {:.3}s", time.elapsed().unwrap().as_secs_f32())).await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn convert_image(msg: &Message, data: &Vec<u8>) -> Vec<u8> {
    let wand = MagickWand::new();
    let mut background_color = PixelWand::new();
    background_color.set_color("none").unwrap();

    wand.read_image_blob(&data).unwrap();
    if wand.get_image_height() > 1024 || wand.get_image_width() > 1024 {
        wand.fit(1024, 1024);
    }
    let width = wand.get_image_width();
    let height = wand.get_image_height();
    for param in msg.content.split(" ") {
        match param {
            "flip" => {
                wand.flip_image().unwrap();
            }
            "mirror" => {
                wand.flop_image().unwrap();
            }
            "color" => {
                wand.kmeans(16, 10, 5.0).unwrap();
            }
            "rotate" => {
                wand.rotate_image(&background_color, 45.0).unwrap();
            }
            "fry" => {
                wand.sharpen_image(50.0, 20.0).unwrap();
                wand.modulate_image(100.0, 400.0, 100.0).unwrap();
            }
            "liquid" => {
                wand.liquid_rescale_image(wand.get_image_width() / 2, wand.get_image_height() / 2, 0.0, 50.0).unwrap();
                wand.liquid_rescale_image(width, height, 0.0, 50.0).unwrap();
            }
            _ => {}
        }
    }
    return wand.write_image_blob("png").unwrap();
}

async fn send_message(ctx: &Context, msg: &Message, response: &str) {
    if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| m.content(response)).await {
        println!("Error sending message: {:?}", why);
    }
}

async fn send_file(ctx: &Context, msg: &Message, response: Option<&str>, data: Vec<u8>) {
    let files = vec![(data.as_slice(), "image_generated_by_ris_very_good_image_service.png")];

    let response = if response == None {
        ""
    }else {
        response.unwrap()
    };
    
    if let Err(why) = msg.channel_id.send_files(&ctx.http, files, |m| m.content(response)).await {
        println!("Error sending message: {:?}", why);
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Could not find .env file, did you forget to create one? err");

    magick_wand_genesis();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}