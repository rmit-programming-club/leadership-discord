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
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput, GetItemInput, AttributeValue, ScanInput};


#[group]
#[commands(getpoints, givepoints, givegems, store)]
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

    match get_profile(&user_id).await {
        Ok(profile) => {
            msg.channel_id.send_message(&ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title(username + "'s points");
                    e.description(show_points(profile));

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

struct Profile {
  points: i64,
  credits: i64
}

async fn get_profile(user_id: &str) -> Result<Profile, String> {
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
                    let points = item["points"].n.as_ref().unwrap_or(&"0".to_string()).parse::<i64>().unwrap_or(0);
                    let credits = item["credits"].n.as_ref().unwrap_or(&"0".to_string()).parse::<i64>().unwrap_or(0);

                    Ok(Profile { points, credits })
                }
                None => {
                    Ok(Profile { points: 0, credits: 0 })
                }
        },
        Err(err) =>
            Err(err.to_string())
    }
}

fn show_points(profile: Profile) -> String {
    profile.credits.to_string() + " points\n" + &profile.points.to_string() + " gems"
}

async fn set_profile(user_id: &str, profile: Profile) -> Result<Profile, String> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let mut put_item_input : PutItemInput = Default::default();
    let mut new_item: HashMap<String, AttributeValue> = HashMap::new();
    let mut key: AttributeValue = Default::default();
    key.s = Some(user_id.to_string());

    let mut points_attr: AttributeValue = Default::default();
    points_attr.n = Some(profile.points.to_string());

    let mut credits_attr: AttributeValue = Default::default();
    credits_attr.n = Some(profile.credits.to_string());

    new_item.insert("discord_id".to_string(), key);
    new_item.insert("points".to_string(), points_attr);
    new_item.insert("credits".to_string(), credits_attr);
    put_item_input.item = new_item;
    put_item_input.table_name = "TPCMemberPoints".to_string();

    match client.put_item(put_item_input).await {
        Ok(_) => {
            Ok(profile)
        },
        Err(err) =>
            Err(err.to_string())
        
    }
}

#[command]
async fn givepoints(ctx: &Context, msg: &Message) -> CommandResult {

    if !message_from_admin(msg){
        return Ok(())
    }

    //get args
    let mut content = msg.content.to_string();
    content.remove(0);
    let sections: Vec<&str> = content.split_ascii_whitespace().collect();
    let user_id = sections[1].to_string();
    let amt = sections[2].parse::<i64>().unwrap();

    match get_profile(&user_id).await {
        Ok(profile) => {
            let new_points = profile.points + amt;
            match set_profile(&user_id, Profile {points: new_points, credits: profile.credits }).await {
                Ok(new_profile) => {
                    msg.channel_id.send_message(&ctx, |m| {
                        m.content("");
                        m.embed(|e| {
                            e.title("Given points!");
                            e.description(show_points(new_profile));

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

fn message_from_admin(msg: &Message) -> bool {
    match msg.member.to_owned() {
        None => false,
        Some(member) => {
            member.roles.contains(&RoleId(449076533223751691)) ||
             member.roles.contains(&RoleId(778454540814909472))

        }
    }

}

#[command]
async fn givegems(ctx: &Context, msg: &Message) -> CommandResult {

    if !message_from_admin(msg){
        return Ok(())
    }

    //get args
    let mut content = msg.content.to_string();
    content.remove(0);
    let sections: Vec<&str> = content.split_ascii_whitespace().collect();
    match (sections.get(1), sections.get(2).and_then(|amt| amt.parse::<i64>().ok())){
         (Some(user_id), Some(amt)) => {
            match get_profile(&user_id).await {
                Ok(profile) => {
                    let new_credits = profile.credits + amt;
                    match set_profile(&user_id, Profile {points: profile.points, credits: new_credits }).await {
                        Ok(new_profile) => {
                            msg.channel_id.send_message(&ctx, |m| {
                                m.content("");
                                m.embed(|e| {
                                    e.title("Given gems!");
                                    e.description(show_points(new_profile));

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
        },
        _ => Ok(())
    }
}

struct Product {
  name: String,
  price: i64,
  quantity: i64,
  key: String,
  description: String
}

async fn get_store() -> Result<Vec<Product>,String> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let mut scan_input: ScanInput = Default::default();

    scan_input.table_name = "TPCStore".to_string();

    match client.scan(scan_input).await {
        Ok(output) => 
            match output.items {
                Some(items) => {
                    let products = items.iter().map(|item| {
                        let empty_str : String = "".to_string();
                        let name = item["name"].s.as_ref().unwrap_or(&empty_str);
                        let price = item["price"].n.as_ref().unwrap_or(&"0".to_string()).parse::<i64>().unwrap_or(0);
                        let quantity = item["quantity"].n.as_ref().unwrap_or(&"0".to_string()).parse::<i64>().unwrap_or(0);
                        let key = item["key"].s.as_ref().unwrap_or(&empty_str);
                        let description = item["description"].s.as_ref().unwrap_or(&empty_str);
                        Product { name: name.to_string(), price, key: key.to_string(), description: description.to_string(), quantity }
                    }).collect();
                    Ok(products)
                }
                None => {
                    Ok(Vec::new())
                }
        },
        Err(err) =>
            Err(err.to_string())
    }
}

#[command]
async fn store(ctx: &Context, msg: &Message) -> CommandResult {

    //get args
    let mut content = msg.content.to_string();
    content.remove(0);
    match get_store().await {
        Ok(products) => {
            msg.channel_id.send_message(&ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title("Products: ");
                    let product_lines : Vec<String> = products.iter().map(|product| {
                        format!("{}: {} ({} gems, {} left)\n{}",product.key, product.name, product.price, product.quantity, product.description)
                    }).collect();
                    let message : String = product_lines.join("\n\n");
                    e.description(message);
                    e
                });
                m
            }).await?;
        },
        Err(err) => {
            println!("Error: {:?}", err);
        }
    };
    Ok(())
}
