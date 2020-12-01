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
use serenity::model::id::{RoleId, UserId};
use serenity::static_assertions::_core::str::FromStr;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput, GetItemInput, AttributeValue};


#[group]
#[commands(getpoints, givepoints)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}


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

    match get_points(&user_id).await {
        Ok(points) => {
            msg.channel_id.send_message(&ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title(username + "'s points");
                    e.description(points.to_string());

                    e
                });
                m
            }).await?;
        }
        ,
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }

    Ok(())
}

async fn get_points(user_id: &str) -> Result<i64, String> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let mut get_item_input: GetItemInput = Default::default();
    let mut key: HashMap<String, AttributeValue> = HashMap::new();
    let mut key_val: AttributeValue = Default::default();
    key_val.s = Some(user_id.to_string());

    key.insert("discord_id".to_string(), key_val);
    get_item_input.key = key;
    get_item_input.table_name = "TPCMemberPoints".to_string();

    match client.get_item(get_item_input).await {
        Ok(output) => 
            match output.item {
                Some(item) => {
                    let points = item["points"].n.as_ref().unwrap();

                    Ok(points.parse::<i64>().unwrap())
                }
                None => {
                    Ok(0)
                }
        },
        Err(err) =>
            Err(err.to_string())
    }
}

async fn set_points(user_id: &str, points: i64) -> Result<i64, String> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let mut put_item_input : PutItemInput = Default::default();
    let mut new_item: HashMap<String, AttributeValue> = HashMap::new();
    let mut key: AttributeValue = Default::default();
    key.s = Some(user_id.to_string());

    let mut points_attr: AttributeValue = Default::default();
    points_attr.n = Some(points.to_string());

    new_item.insert("discord_id".to_string(), key);
    new_item.insert("points".to_string(), points_attr);
    put_item_input.item = new_item;
    put_item_input.table_name = "TPCMemberPoints".to_string();

    match client.put_item(put_item_input).await {
        Ok(_) => {
            Ok(points)
        },
        Err(err) =>
            Err(err.to_string())
        
    }
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

    match get_points(&user_id).await {
        Ok(points) => {
            let new_points = points + amt;
            match set_points(&user_id, new_points).await {
                Ok(_) => {
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
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            }
        },
        Err(err) => {
            println!("Error: {:?}", err);
        }
    };

    Ok(())
}
