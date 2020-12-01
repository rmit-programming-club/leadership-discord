use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

use std::env;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, BufWriter, Write};

#[group]
#[commands(getpoints, givepoint)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

const PATH: &str = "scores.txt";

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn getpoints(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Your points").await?;
    let user_id = msg.author.id;

    let mut scores: HashMap<String, i32> = HashMap::new();

    {
        let file = File::open(PATH).expect("Can't find the file");
        let filereader = BufReader::new(file);
        for line in filereader.lines() {
            let ln: String = line.unwrap();
            let tokens: Vec<&str> = ln.split(":").collect();

            scores.insert(tokens[0].to_string(), i32::from(tokens[1].to_string().parse::<i32>().unwrap()));
        }
    }

    if !scores.contains_key(user_id.to_string().as_str()) {
        msg.reply(ctx, "You don't have any points").await?;
    } else {
        let cur_score = *scores.get(&user_id.to_string()).unwrap();
        let message : String = "You have ".to_owned() + &*cur_score.to_string() + " points";
        msg.reply(ctx, message).await?;
        scores.insert(user_id.to_string(), cur_score + 1);
    }

    Ok(())
}

#[command]
async fn givepoint(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Command");
    let user_id = msg.author.id;
    let mut scores: HashMap<String, i32> = HashMap::new();

    {
        let file = File::open(PATH).expect("Can't find the file");
        let filereader = BufReader::new(file);
        for line in filereader.lines() {
            let ln: String = line.unwrap();
            let tokens: Vec<&str> = ln.split(":").collect();

            scores.insert(tokens[0].to_string(), i32::from(tokens[1].to_string().parse::<i32>().unwrap()));
        }
    }

    if !scores.contains_key(user_id.to_string().as_str()) {
        scores.insert(user_id.to_string(), 0);
    } else {
        let cur_score = *scores.get(&user_id.to_string()).unwrap();
        scores.insert(user_id.to_string(), cur_score + 1);
    }
    for (key, value) in &scores {
        println!("{}:{}", key, value);
    }

    let file = OpenOptions::new().write(true).open(PATH).unwrap();

    let mut writer = BufWriter::new(file);
    for (key, value) in scores {
        // println!("{} / {}", key, value);
        //writeln!(&mut file,"{}:{}", key, value.to_string());
        let out: String = key + ":" + &*value.to_string() + "\n";
        writer.write(out.as_bytes()).unwrap();
    }

    writer.flush().unwrap();


    msg.reply(ctx, "Gave you a point!").await?;

    Ok(())
}
