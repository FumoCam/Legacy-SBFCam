use dotenv;
use enigo::{Enigo, Key, KeyboardControllable, MouseButton, MouseControllable};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::Duration;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use winapi;
use windows_win;

fn move_direction(direction: &str, duration: f64) {
    let direction_char = direction.chars().next().expect("string is empty");
    let direction_key = Key::Layout(direction_char);
    const MOVE_RATIO: f64 = 600.0;
    let true_duration = (duration * MOVE_RATIO).round() as u64;
    let delay: Duration = Duration::from_millis(true_duration);
    let mut enigo = Enigo::new();
    enigo.key_down(direction_key);
    thread::sleep(delay);
    enigo.key_up(direction_key);
}

fn camera_zoom(direction: &str, duration: f64) {
    let direction_char = direction.chars().next().expect("string is empty");
    let direction_key = Key::Layout(direction_char);
    const ZOOM_RATIO: f64 = 25.0;
    let true_duration = (duration * ZOOM_RATIO).round() as u64;
    let delay: Duration = Duration::from_millis(true_duration);
    let mut enigo = Enigo::new();
    enigo.key_down(direction_key);
    thread::sleep(delay);
    enigo.key_up(direction_key);
}

fn camera_x(x_ratio: f32) {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    const EULER_MOUSEX_MULTI: f32 = 2.0; //2.5; // 90 * this will rotate 90 degrees
    let x_amount = (EULER_MOUSEX_MULTI * x_ratio).round() as i32;
    mouse_move(&mut enigo, 0.5, 0.5);
    thread::sleep(DELAY);
    enigo.mouse_down(MouseButton::Right);
    thread::sleep(DELAY);
    enigo.mouse_move_relative(x_amount, 0);
    thread::sleep(DELAY);
    enigo.mouse_up(MouseButton::Right);
    thread::sleep(DELAY);
    mouse_move_trigger(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

fn camera_y(y_ratio: f32) {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    const EULER_MOUSEY_MULTI: f32 = 2.861111; // 180 * this will rotate up/down 100%
    let y_amount = (EULER_MOUSEY_MULTI * y_ratio).round() as i32;
    mouse_move(&mut enigo, 0.5, 0.5);
    thread::sleep(DELAY);
    enigo.mouse_down(MouseButton::Right);
    thread::sleep(DELAY);
    enigo.mouse_move_relative(0, y_amount);
    thread::sleep(DELAY);
    enigo.mouse_up(MouseButton::Right);
    thread::sleep(DELAY);
    mouse_move_trigger(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

fn leap(forward_amount: f64, spacebar_amount: f64) {
    let forward_ms = (forward_amount * 1000.0).round() as u64;
    let spacebar_ms = (spacebar_amount * 1000.0).round() as u64;
    let mut enigo = Enigo::new();
    if forward_ms >= spacebar_ms {
        let forward_delay: Duration = Duration::from_millis(forward_ms - spacebar_ms);
        let spacebar_delay: Duration = Duration::from_millis(spacebar_ms);
        enigo.key_down(Key::Layout('w'));
        enigo.key_down(Key::Space);
        thread::sleep(spacebar_delay);
        enigo.key_up(Key::Space);
        thread::sleep(forward_delay);
        enigo.key_up(Key::Layout('w'));
    } else {
        let spacebar_delay: Duration = Duration::from_millis(spacebar_ms - forward_ms);
        let forward_delay: Duration = Duration::from_millis(forward_ms);
        enigo.key_down(Key::Layout('w'));
        enigo.key_down(Key::Space);
        thread::sleep(forward_delay);
        enigo.key_up(Key::Layout('w'));
        thread::sleep(spacebar_delay);
        enigo.key_up(Key::Space);
    }
}

fn navbar_grief() {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.62, 0.93);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

fn navbar_sit() {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.25, 0.93);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

// fn navbar_character_select() {
//     const DELAY: Duration = Duration::from_millis(300);
//     let mut enigo = Enigo::new();
//     mouse_move(&mut enigo, 0.3, 0.77);
//     thread::sleep(DELAY);
//     mouse_click(&mut enigo);
//     thread::sleep(DELAY);
//     mouse_hide(&mut enigo);
// }

fn mouse_move(enigo: &mut Enigo, x_ratio: f32, y_ratio: f32) {
    const SCREEN_W: f32 = 1280.0;
    const SCREEN_H: f32 = 720.0;

    let x: i32 = (x_ratio * SCREEN_W).round() as i32;
    let y: i32 = (y_ratio * SCREEN_H).round() as i32;
    enigo.mouse_move_to(x, y);
    mouse_move_trigger(enigo);
}
fn mouse_move_trigger(enigo: &mut Enigo) {
    enigo.mouse_move_relative(-1, -1);
    enigo.mouse_move_relative(1, 1);
}
fn mouse_click(enigo: &mut Enigo) {
    enigo.mouse_down(MouseButton::Left);
    enigo.mouse_up(MouseButton::Left);
}
fn mouse_hide(enigo: &mut Enigo) {
    mouse_move(enigo, 1.0, 1.0);
    mouse_move_trigger(enigo);
}

fn send_chat_message(msg: &str) {
    println!("send_chat_message start"); // Debug

    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('/'));
    thread::sleep(Duration::from_millis(400));
    let suffixed_msg = format!("{} ", msg); // Space suffix, to avoid cutoff
    if msg.starts_with("/") {
        enigo.key_sequence(msg.as_ref());
    } else {
        enigo.key_sequence(suffixed_msg.as_ref());
    }
    thread::sleep(Duration::from_millis(150));
    enigo.key_click(Key::Return);

    println!("send_chat_message end"); // Debug
}

fn send_chat_message_ref(msg: &str, mut enigo: Enigo) -> Enigo {
    // Experiment, running send_chat_message twice quickly has a small chance to not execute enigo funcs the 2nd time
    println!("send_chat_message start"); // Debug

    // let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('/'));
    thread::sleep(Duration::from_millis(400));
    let suffixed_msg = format!("{} ", msg); // Space suffix, to avoid cutoff
    if msg.starts_with("/") {
        enigo.key_sequence(msg.as_ref());
    } else {
        enigo.key_sequence(suffixed_msg.as_ref());
    }
    thread::sleep(Duration::from_millis(150));
    enigo.key_click(Key::Return);

    println!("send_chat_message end"); // Debug
    return enigo;
}

fn warp(desired_location: &str) {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('\\'));
    thread::sleep(Duration::from_millis(400));
    let command = format!("warp {}", desired_location.to_string());
    enigo.key_sequence(&command);
    enigo.key_click(Key::Return);
}

fn run_refresh() {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('\\'));
    thread::sleep(Duration::from_millis(400));
    enigo.key_sequence("re");
    enigo.key_click(Key::Return);
}

fn run_die() {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('\\'));
    thread::sleep(Duration::from_millis(400));
    enigo.key_sequence("die");
    enigo.key_click(Key::Return);
}

fn run_explode() {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('\\'));
    thread::sleep(Duration::from_millis(400));
    enigo.key_sequence("explode");
    enigo.key_click(Key::Return);
}

async fn discord_log(
    message: &str,
    author: &str,
    author_url: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    const AUTHOR_AVATAR: &str = "https://brand.twitch.tv/assets/images/black.png";
    let webhook_url =
        env::var("DISCORD_LOG_WEBHOOK_URL").expect("$DISCORD_LOG_WEBHOOK_URL is not set");

    let webhook_data = json!({
        "embeds": [
            {
                "description": message,
                "author": {
                    "name": author,
                    "url": author_url,
                    "icon_url": AUTHOR_AVATAR,
                },
            }
        ]
    });
    let client = reqwest::Client::new();
    let _resp = client.post(webhook_url).json(&webhook_data).send().await?;
    Ok(())
}
async fn notify_admin(
    message: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let webhook_url =
        env::var("DISCORD_ALERT_WEBHOOK_URL").expect("$DISCORD_ALERT_WEBHOOK_URL is not set");
    let author_discord_id = env::var("AUTHOR_DISCORD_ID").expect("$AUTHOR_DISCORD_ID is not set");
    let mut webhook_data = HashMap::new();
    webhook_data.insert("content", format!("<@{}>\n{}", author_discord_id, message));
    webhook_data.insert("username", "SBF Cam".to_string());
    let client = reqwest::Client::new();
    let _resp = client.post(webhook_url).json(&webhook_data).send().await?;
    Ok(())
}
fn check_active(window_title: &str) -> bool {
    if get_active_window() != window_title {
        show_window_by_title(window_title);
        thread::sleep(Duration::from_millis(500));
        return get_active_window() == window_title;
    }
    return true;
}
fn get_active_window() -> String {
    let active_window_hwnd = unsafe { winapi::um::winuser::GetForegroundWindow() };
    const BUFFER_SIZE: usize = 512;
    let mut buffer: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let writ_chars = unsafe {
        winapi::um::winuser::GetWindowTextW(
            active_window_hwnd,
            buffer.as_mut_ptr(),
            BUFFER_SIZE as i32,
        )
    };
    if writ_chars == 0 {
        return "Error".to_string();
    }
    return String::from_utf16_lossy(&buffer[0..writ_chars as usize]);
}
fn show_window_by_title(title: &str) -> bool {
    let window_hwnd_ref_vec = windows_win::raw::window::get_by_title(title, None).unwrap();
    let window_hwnd_ref = window_hwnd_ref_vec.first();
    match window_hwnd_ref {
        Some(window_hwnd_ref) => {
            let window_hwnd_raw = *window_hwnd_ref;
            let success = show_window(window_hwnd_raw);
            if success {
                println!("Successfully activated {title}!", title = title);
            } else {
                eprintln!("Issue in activating {title}", title = title);
            }
            return success;
        }
        None => {
            eprintln!("Couldn't find a window by the name {title}", title = title);
            return false;
        }
    }
}
fn show_window(raw_window_hwnd_ref: *mut winapi::shared::windef::HWND__) -> bool {
    //https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    const SW_MAXIMIZE: winapi::ctypes::c_int = 3;
    const SW_MINIMIZE: winapi::ctypes::c_int = 6;
    unsafe {
        let minimize_success =
            winapi::um::winuser::ShowWindow(raw_window_hwnd_ref, SW_MINIMIZE) != 0;
        if !minimize_success {
            println!("Issue in minimize operation");
        }
        let maximize_success =
            winapi::um::winuser::ShowWindow(raw_window_hwnd_ref, SW_MAXIMIZE) != 0;
        if !maximize_success {
            println!("Issue in maximize operation");
        }
        return minimize_success && maximize_success;
    }
}

#[tokio::main]
pub async fn main() {
    dotenv::dotenv().ok();
    let channel_name = "sbfcam";
    let username = "sbfcamBOT";
    //Oauth (generated with https://twitchtokengenerator.com/):
    let token = env::var("TWITCH_BOT_TOKEN").expect("$TWITCH_BOT_TOKEN is not set");
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        username.to_owned(),
        Some(token.to_owned()),
    ));
    const GAME_NAME: &str = "Roblox";
    let mut processing_queue = false;

    // TODO: Make this less awful
    let mut tp_locations = HashMap::new();
    tp_locations.insert(String::from("big"), String::from("Big Island"));
    tp_locations.insert(String::from("build"), String::from("i forgot how to build"));
    tp_locations.insert(String::from("fountain"), String::from("Fountain"));
    tp_locations.insert(String::from("fumofas"), String::from("Fumofas Park"));
    tp_locations.insert(String::from("minesweeper"), String::from("Minesweeper"));
    tp_locations.insert(String::from("pof"), String::from("plates of fate v1.5"));
    tp_locations.insert(String::from("poolside"), String::from("Poolside"));
    tp_locations.insert(
        String::from("radio"),
        String::from("Radio Rock + Raging Demon Raceway"),
    );
    tp_locations.insert(String::from("ruins"), String::from("Ruins"));
    tp_locations.insert(String::from("shrimp"), String::from("Shreimp Mart"));
    tp_locations.insert(String::from("sky"), String::from("Floating Island"));
    tp_locations.insert(
        String::from("tictactoe"),
        String::from("ultimate tic tac toe"),
    );
    tp_locations.insert(String::from("fire"), String::from("Fireside Island"));

    let valid_tp_locations = tp_locations
        .keys()
        .map(|s| &**s)
        .collect::<Vec<_>>()
        .join(", ");

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let twitch_join_handle = tokio::spawn(async move {
        match client.join(channel_name.to_owned()) {
            Ok(join) => join,
            Err(error) => panic!("Could not join the channel {:?}", error),
        }
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    println!("{}: {}", msg.sender.name, msg.message_text);
                    if msg.message_text.starts_with("!") {
                        let args = msg.message_text.replacen("!", "", 1);
                        let clean_args: Vec<&str> = args.split_whitespace().collect();
                        if clean_args.len() < 1 {
                            continue;
                        }
                        let trigger = clean_args[0].to_lowercase();
                        match trigger.as_ref() {
                            "ping" => client
                                .reply_to_privmsg(String::from("pong"), &msg)
                                .await
                                .unwrap(),
                            "help" => {
                                client
                                    .reply_to_privmsg(
                                        format!(
                                            "For a full list of commands, visit {} . If you just want to play around, try '!m hello', '!move w 2', or '!warp poolside'",
                                            "https://sbf.fumocam.xyz/commands"
                                        ),
                                        &msg,
                                    )
                                    .await.unwrap();
                            }
                            "m" => {
                                let mut msg_args = clean_args.to_owned();
                                msg_args.drain(0..1);
                                let message = msg_args.join(" ");
                                if message.len() == 0 {
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Specify a message! (i.e. !m hello)]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }

                                if (message.starts_with("/") && !(message.starts_with("/e")))
                                    || message.starts_with("[")
                                {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[You cannot run commands other than /e!]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }

                                let mut author_name = msg.sender.name.to_string();

                                // TODO: config option for this
                                if author_name.to_lowercase() == "sbfcam" {
                                    author_name = "[CamDev]".to_string();
                                }

                                let author_msg = format!("{}:", author_name);
                                println!("{}: {}", author_name.to_lowercase(), message);

                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                if !processing_queue {
                                    processing_queue = true;
                                    let success = check_active(GAME_NAME);
                                    if !success {
                                        let _ = notify_admin("Failed to find Roblox!").await;
                                        client
                                            .reply_to_privmsg(
                                                String::from(
                                                    "[Failed to find Roblox! Notified dev.]",
                                                ),
                                                &msg,
                                            )
                                            .await
                                            .unwrap();
                                        processing_queue = false;
                                        continue;
                                    }
                                    let mut chat_enigo = Enigo::new();
                                    if !message.starts_with("/") {
                                        println!("Sending author name: {}", author_msg);
                                        chat_enigo = send_chat_message_ref(&author_msg, chat_enigo);
                                    }
                                    println!("Sending message: {}", message);
                                    chat_enigo =
                                        send_chat_message_ref(&message.to_owned(), chat_enigo);
                                    println!("Sent message: {}", message);
                                    processing_queue = false;
                                }
                            }
                            "dev" => {
                                let message = args.replacen("dev ", "", 1).replacen("dev", "", 1);
                                if message.len() == 0 {
                                    client
                                        .reply_to_privmsg(String::from("[Specify a message, this command is for emergencies! (Please do not misuse it)]"), &msg)
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                let result = notify_admin(&message).await;
                                if result.is_ok() {
                                    client
                                        .reply_to_privmsg(String::from("[Notified dev! As a reminder, this command is only for emergencies. If you were unaware of this and used the command by mistake, please write a message explaining that or you may be timed-out/banned.]"), &msg)
                                        .await
                                        .unwrap();
                                } else {
                                    client
                                        .reply_to_privmsg(String::from("[Error! Was unable to notify dev. Please join the Discord and ping CamDev.]"), &msg)
                                        .await
                                        .unwrap();
                                }
                            }
                            "move" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Specify a direction! (i.e. !move w 1)]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                let valid_directions = vec![
                                    String::from("w"),
                                    String::from("a"),
                                    String::from("s"),
                                    String::from("d"),
                                ];
                                let direction = clean_args[1].to_lowercase();
                                if !valid_directions.contains(&direction) {
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Invalid direction! Specify a proper direction. (w, a, s, d)]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f64 = 15.0;
                                let mut amount: f64 = 1.0;
                                if clean_args.len() > 2 {
                                    let num = clean_args[2].parse::<f64>();
                                    match num {
                                        Ok(val) => {
                                            if val > MAX_AMOUNT || val <= 0.0 {
                                                client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[2], MAX_AMOUNT), &msg).await.unwrap();
                                                continue;
                                            }
                                            amount = val;
                                        }
                                        Err(_why) => {
                                            client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[2]), &msg).await.unwrap();
                                            continue;
                                        }
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                move_direction(&direction, amount);
                                processing_queue = false;
                            }
                            "warp" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            format!("[Specify a location! Map: https://sbf.fumocam.xyz/warp-map (Valid locations: {})]", valid_tp_locations),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                let desired_location = tp_locations.get(clean_args[1]);

                                match desired_location {
                                    Some(desired_location) => {
                                        while processing_queue {
                                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                                100,
                                            ))
                                            .await;
                                        }
                                        processing_queue = true;
                                        let success = check_active(GAME_NAME);
                                        if !success {
                                            let _ = notify_admin("Failed to find Roblox!").await;
                                            client
                                                .reply_to_privmsg(
                                                    String::from(
                                                        "[Failed to find Roblox! Notified dev.]",
                                                    ),
                                                    &msg,
                                                )
                                                .await
                                                .unwrap();
                                            processing_queue = false;
                                            continue;
                                        }
                                        send_chat_message(
                                            format!("[Warping to {}!]", desired_location).as_ref(),
                                        );
                                        leap(1.0, 1.0);
                                        warp(desired_location);
                                        processing_queue = false;
                                    }
                                    None => {
                                        client.reply_to_privmsg(format!("[{} is not a valid location! Map: https://sbf.fumocam.xyz/warp-map (Valid locations: {})]", clean_args[1], valid_tp_locations), &msg).await.unwrap();
                                    }
                                }
                            }
                            "left" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify degrees to rotate! (i.e. !left 90)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f32 = 360.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f32>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 90.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_x(amount);
                                processing_queue = false;
                            }
                            "right" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify degrees to rotate! (i.e. !right 90)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f32 = 360.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f32>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 90.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_x(amount * -1.0);
                                processing_queue = false;
                            }
                            "up" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify degrees to rotate! (i.e. !up 45)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f32 = 180.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f32>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_y(amount);
                                processing_queue = false;
                            }
                            "down" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify degrees to rotate! (i.e. !down 45)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f32 = 180.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f32>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_y(amount * -1.0);
                                processing_queue = false;
                            }
                            "zoomin" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify percent to zoom in! (i.e. !zoomin 50)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f64 = 1000000.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f64>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_zoom("i", amount);
                                processing_queue = false;
                            }
                            "zoomout" => {
                                if clean_args.len() < 2 {
                                    client
                                        .reply_to_privmsg(
                                            String::from(
                                                "[Specify percent to zoom in! (i.e. !zoomin 50)]",
                                            ),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                const MAX_AMOUNT: f64 = 1000000.0;
                                let mut amount = 45.0;
                                let num = clean_args[1].parse::<f64>();
                                match num {
                                    Ok(val) => {
                                        if val > MAX_AMOUNT || val <= 0.0 {
                                            client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                            continue;
                                        }
                                        amount = val;
                                    }
                                    Err(_why) => {
                                        client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number, i.e. 50.]", clean_args[1]), &msg).await.unwrap();
                                        continue;
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                camera_zoom("o", amount);
                                processing_queue = false;
                            }
                            "leap" => {
                                const MAX_AMOUNT: f64 = 2.0;
                                let mut forward_amount: f64 = 1.0;
                                let mut spacebar_amount: f64 = 1.0;
                                if clean_args.len() > 1 {
                                    let num1 = clean_args[1].parse::<f64>();
                                    match num1 {
                                        Ok(val) => {
                                            if val > MAX_AMOUNT || val <= 0.0 {
                                                client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[1], MAX_AMOUNT), &msg).await.unwrap();
                                                continue;
                                            }
                                            forward_amount = val;
                                        }
                                        Err(_why) => {
                                            client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[1]), &msg).await.unwrap();
                                            continue;
                                        }
                                    }
                                }
                                if clean_args.len() > 2 {
                                    let num1 = clean_args[2].parse::<f64>();
                                    match num1 {
                                        Ok(val) => {
                                            if val > MAX_AMOUNT || val <= 0.0 {
                                                client.reply_to_privmsg(format!("[{} is too high/low! Please specify a number between 0 or {}.]", clean_args[2], MAX_AMOUNT), &msg).await.unwrap();
                                                continue;
                                            }
                                            spacebar_amount = val;
                                        }
                                        Err(_why) => {
                                            client.reply_to_privmsg(format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[2]), &msg).await.unwrap();
                                            continue;
                                        }
                                    }
                                }
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                leap(forward_amount, spacebar_amount);
                                processing_queue = false;
                            }
                            "hidemouse" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                let mut enigo = Enigo::new();
                                mouse_hide(&mut enigo);
                                processing_queue = false;
                            }
                            "jump" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                leap(0.0, 1.0);
                                processing_queue = false;
                            }
                            "grief" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                navbar_grief();
                                processing_queue = false;
                            }
                            "refresh" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                run_refresh();
                                processing_queue = false;
                            }
                            "die" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                run_die();
                                processing_queue = false;
                            }
                            "explode" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                run_explode();
                                processing_queue = false;
                            }
                            "sit" => {
                                while processing_queue {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                                processing_queue = true;
                                let success = check_active(GAME_NAME);
                                if !success {
                                    let _ = notify_admin("Failed to find Roblox!").await;
                                    client
                                        .reply_to_privmsg(
                                            String::from("[Failed to find Roblox! Notified dev.]"),
                                            &msg,
                                        )
                                        .await
                                        .unwrap();
                                    processing_queue = false;
                                    continue;
                                }
                                navbar_sit();
                                processing_queue = false;
                            }
                            _ => (),
                        }
                    }

                    let author_url = format!(
                        "https://www.twitch.tv/popout/sbfcam/viewercard/{}",
                        msg.sender.name.to_lowercase()
                    );
                    let _result =
                        discord_log(&msg.message_text, &msg.sender.name, &author_url).await;
                }
                _ => {}
            }
        }
    });

    let anti_afk_join_handle = tokio::spawn(async move {
        let interval_minutes = 10;
        let mut interval =
            tokio::time::interval(Duration::from_millis(interval_minutes * 60 * 1000));
        interval.tick().await;
        loop {
            for _ in 0..3 {
                interval.tick().await;
                let success = check_active(GAME_NAME);
                if !success {
                    let _ = notify_admin("Failed to find Roblox!").await;
                    continue;
                } else {
                    camera_x(45.0);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    camera_x(-90.0);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    camera_x(45.0);
                }
            }
            send_chat_message("You can control this bot live on T witch! Go to t witch.tv and and search my username (without '_03')")
        }
    });

    let mut handles = Vec::new();
    handles.push(twitch_join_handle);
    handles.push(anti_afk_join_handle);
    for handle in handles {
        let _ = handle.await.expect("Panic in task");
    }
    println!("Test_end");
}
