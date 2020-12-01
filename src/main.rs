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
use serenity::model::id::{RoleId, UserId};
use serenity::static_assertions::_core::str::FromStr;

#[group]
#[commands(getpoints, givepoints)]
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
    let mut user_id = msg.author.id.to_string();

    //get args
    let mut content = msg.content.to_string();
    content.remove(0);
    let sections: Vec<&str> = content.split_ascii_whitespace().collect();
    if sections.len() > 1 {
        user_id = sections[1].to_string();
    }

    let user = UserId::from_str(&user_id).unwrap_or(UserId(333)).to_user(ctx).await;
    let username: String;
    match user {
        Ok(u) => {username = u.name}
        _ => {
            msg.channel_id.send_message(&ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title("Point Count");
                    e.description("Could not find user");
                    e.color(0x6e10aau64);
                    e
                });
                m
            }).await?;
            return Ok(());
        }
    }

    let mut scores: HashMap<String, i64> = HashMap::new();

    {
        let file = File::open(PATH).expect("Can't find the file");
        let filereader = BufReader::new(file);
        for line in filereader.lines() {
            let ln: String = line.unwrap();
            let tokens: Vec<&str> = ln.split(":").collect();

            scores.insert(tokens[0].to_string(), i64::from(tokens[1].to_string().parse::<i64>().unwrap()));
        }
    }

    if !scores.contains_key(user_id.to_string().as_str()) {
        msg.channel_id.send_message(&ctx, |m| {
            m.content("");
            m.embed(|e| {
                e.title("Point Count");
                e.description("User ".to_owned() + &username + " doesn't have any points");

                e
            });
            m
        }).await?;

    } else {
        let cur_score = *scores.get(&user_id.to_string()).unwrap();
        let message : String = username.as_str().to_owned() + " has " + &*cur_score.to_string() + " points";

        msg.channel_id.send_message(&ctx, |m| {
            m.content("");
            m.embed(|e| {
                e.title("Point Count");
                e.description(message);

                e
            });
            m
        }).await?;

        scores.insert(user_id.to_string(), cur_score + 1);
    }

    Ok(())
}

#[command]
async fn givepoints(ctx: &Context, msg: &Message) -> CommandResult {

    let msg_member = msg.member.to_owned();
    let sender_roles = msg_member.unwrap().roles;

    if !sender_roles.contains(&RoleId(449076533223751691)) &&
     !sender_roles.contains(&RoleId(778454540814909472)) {
        return Ok(())
    }

    println!("{}", msg.content);
    //get args
    let mut content = msg.content.to_string();
    content.remove(0);
    let sections: Vec<&str> = content.split_ascii_whitespace().collect();
    let user_id = sections[1].to_string();
    let amt = sections[2].parse::<i64>().unwrap();


    println!("Command");
    let mut scores: HashMap<String, i64> = HashMap::new();

    {
        let file = File::open(PATH).expect("Can't find the file");
        let filereader = BufReader::new(file);
        for line in filereader.lines() {
            let ln: String = line.unwrap();
            let tokens: Vec<&str> = ln.split(":").collect();

            scores.insert(tokens[0].to_string(), i64::from(tokens[1].to_string().parse::<i64>().unwrap()));
        }
    }

    if !scores.contains_key(user_id.to_string().as_str()) {
        scores.insert(user_id.to_string(), amt);
    } else {
        let cur_score = *scores.get(&user_id.to_string()).unwrap();
        scores.insert(user_id.to_string(), cur_score + amt);
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

    let output = "Gave ".to_owned() + &user_id + " " + amt.to_string().as_str() + " points!";
    msg.channel_id.send_message(&ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title("Given points!");
            e.description(output);

            e
        });
        m
    }).await?;

    Ok(())
}
