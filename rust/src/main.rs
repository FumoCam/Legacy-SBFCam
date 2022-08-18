use chrono::Timelike;
use dotenv;
use enigo::{Enigo, Key, KeyboardControllable, MouseButton, MouseControllable};
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;
use twitch_irc::transport::tcp::TCPTransport;
use twitch_irc::transport::tcp::TLS;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use winapi;
use windows_win;

fn capitalize_string(s: &str) -> String {
    return s[0..1].to_uppercase() + &s[1..];
}

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

fn send_system_chat(msg: &str) {
    let mut enigo = Enigo::new();
    let suffixed_msg = format!("{} ", msg); // Space suffix, to avoid cutoff
    let type_delay = Duration::from_millis(400);
    let send_delay = Duration::from_millis(150);

    enigo.key_click(Key::Layout('/'));
    thread::sleep(type_delay);
    if msg.starts_with("/") {
        // Chat command
        enigo.key_sequence(msg.as_ref());
    } else {
        // Standard message
        enigo.key_sequence(suffixed_msg.as_ref());
    }
    thread::sleep(send_delay);
    enigo.key_click(Key::Return);
}

fn send_user_chat(author: &str, msg: &str) {
    let mut enigo = Enigo::new();

    let suffixed_author = format!("{} ", author); // Space suffix, to avoid cutoff
    let suffixed_msg = format!("{} ", msg); // Space suffix, to avoid cutoff
    let type_delay = Duration::from_millis(400);
    let send_delay = Duration::from_millis(150);
    let author_delay = Duration::from_millis(500);

    // Author
    enigo.key_click(Key::Layout('/'));
    thread::sleep(type_delay);
    enigo.key_sequence(&suffixed_author);
    thread::sleep(send_delay);
    enigo.key_click(Key::Return);

    thread::sleep(author_delay);

    // Message
    enigo.key_click(Key::Layout('/'));
    thread::sleep(type_delay);
    enigo.key_sequence(&suffixed_msg);
    thread::sleep(send_delay);
    enigo.key_click(Key::Return);
}

fn run_console_command(command: &str) {
    let mut enigo = Enigo::new();
    enigo.key_click(Key::Layout('\\'));
    thread::sleep(Duration::from_millis(400));
    enigo.key_sequence(command);
    enigo.key_click(Key::Return);
}

async fn discord_log(
    message: &str,
    author: &str,
    is_chat_log: bool,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    const AUTHOR_AVATAR: &str = "https://brand.twitch.tv/assets/images/black.png";
    let author_url = format!(
        "https://www.twitch.tv/popout/sbfcam/viewercard/{}",
        author.to_lowercase()
    );
    let env_key = if is_chat_log {
        "DISCORD_CHAT_LOG_WEBHOOK_URL"
    } else {
        "DISCORD_LOG_WEBHOOK_URL"
    };
    let webhook_url = env::var(env_key).expect(&format!("{} is not set", env_key));
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
    webhook_data.insert(
        "content",
        format!(
            "<@{}>\n{}\n<https://twitch.tv/sbfcam>",
            author_discord_id, message
        ),
    );
    let client = reqwest::Client::new();
    let _resp = client.post(webhook_url).json(&webhook_data).send().await?;
    Ok(())
}

fn join_server(game_id: i64, server_id: &str) {
    match join_game_selenium(game_id.to_owned(), server_id) {
        Err(_e) => eprintln!("Selenium Error\n{}", _e),
        _ => (),
    }
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
fn trigger_restart() {
    println!("Restart subprocess started");
    let output = Command::new("cmd")
        .args(["/C", "shutdown", "/f", "/r", "/t", "0"])
        .output()
        .expect("failed to execute restart");
    println!("Restart subprocess finished");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
}
fn terminate_running_exe(exe_name: &str) {
    println!("EXE termination subprocess started ({})", exe_name);
    let output = Command::new("cmd")
        .args(["/C", "taskkill", "/f", "/IM", &exe_name])
        .output()
        .expect("failed to execute Roblox termination");
    println!("EXE termination subprocess finished ({})", exe_name);
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
}

#[derive(Clone)]
pub enum Instruction {
    CheckActive {
        window_title: String,
    },
    ConsoleCommand {
        command: String,
    },
    HideMouse {},
    JoinServer {
        server_id: String,
    },
    Grief {},
    Leap {
        forward_amount: f64,
        spacebar_amount: f64,
    },
    MoveCameraX {
        x_ratio: f32,
    },
    MoveCameraY {
        y_ratio: f32,
    },
    MoveDirection {
        direction: String, // TODO: Make char
        duration: f64,
    },
    Restart {},
    Sit {},
    SystemChatMessage {
        message: String,
    },
    TerminateGame {},
    UserChatMessage {
        author: String,
        message: String,
    },
    Wait {
        amount_ms: u64,
    },
    WaitWithMessage {
        amount_ms: u64,
        message: String,
    },
    ZoomCamera {
        direction: String, // TODO: Make char
        duration: f64,
    },
}

pub struct InstructionPair {
    execution_order: i64,
    instruction: Instruction,
}

impl PartialOrd for InstructionPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.execution_order.partial_cmp(&other.execution_order)
    }
}

impl PartialEq for InstructionPair {
    fn eq(&self, other: &Self) -> bool {
        self.execution_order == other.execution_order
    }
}

pub struct SystemInstruction {
    client: Option<Rc<TwitchIRCClient<TCPTransport<TLS>, StaticLoginCredentials>>>,
    chat_message: Option<PrivmsgMessage>,
    instructions: Vec<InstructionPair>,
}

pub async fn queue_processor(
    mut channel_receiver: UnboundedReceiver<SystemInstruction>,
    hud_sender: UnboundedSender<HUDInstruction>,
    bot_config: BotConfig,
) {
    let mut instruction_history: Vec<InstructionPair> = Vec::new();

    loop {
        let system_instruction = channel_receiver.recv().await.unwrap();
        let client_opt = system_instruction.client;
        let chat_message_opt = system_instruction.chat_message;
        let mut instruction_pairs = system_instruction.instructions;
        let mut client_origin = false;
        if let (Some(_client), Some(_chat_message)) = (&client_opt, &chat_message_opt) {
            client_origin = true; // Action was client-requested
        }

        // TODO: Better sort
        instruction_pairs
            .sort_by(|a, b| a.execution_order.partial_cmp(&b.execution_order).unwrap());

        for instruction_pair in &instruction_pairs {
            let mut success = true;
            let history_entry: InstructionPair = InstructionPair {
                execution_order: instruction_history.len() as i64,
                instruction: instruction_pair.instruction.clone(),
            };

            match &instruction_pair.instruction {
                Instruction::CheckActive { window_title } => {
                    println!("check_active");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }

                    success = check_active(&window_title);
                    if !success {
                        let _ = notify_admin("Failed to find Roblox!").await;
                        if let (Some(client), Some(chat_message)) = (&client_opt, &chat_message_opt)
                        {
                            client
                                .reply_to_privmsg(
                                    String::from("[Failed to find Roblox! Notified dev.]"),
                                    &chat_message,
                                )
                                .await
                                .unwrap();
                        }
                        break; // Will exit the instruction_pair loop
                    }
                }
                Instruction::ConsoleCommand { command } => {
                    println!("console_command");
                    if command.starts_with("warp") {
                        instruction_history.clear(); // Reset instruction history per-warp
                    }
                    instruction_history.push(history_entry); // Record even if system-requested
                    run_console_command(&command);
                }
                Instruction::HideMouse {} => {
                    println!("hide_mouse");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    let mut enigo = Enigo::new();
                    mouse_hide(&mut enigo);
                }
                Instruction::Grief {} => {
                    println!("grief");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    navbar_grief();
                }
                Instruction::Leap {
                    forward_amount,
                    spacebar_amount,
                } => {
                    println!("leap");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    leap(*forward_amount, *spacebar_amount);
                }
                Instruction::MoveCameraX { x_ratio } => {
                    println!("move_camera_x");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    camera_x(*x_ratio);
                }
                Instruction::MoveCameraY { y_ratio } => {
                    println!("move_camera_y");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    camera_y(*y_ratio);
                }
                Instruction::MoveDirection {
                    direction,
                    duration,
                } => {
                    println!("move_direction");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    move_direction(direction, *duration);
                }
                Instruction::Restart {} => {
                    println!("restart");
                    trigger_restart();
                }
                Instruction::Sit {} => {
                    println!("sit");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    navbar_sit();
                }
                Instruction::SystemChatMessage { message } => {
                    println!("system_chat_message");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    send_system_chat(&message);
                }
                Instruction::TerminateGame {} => {
                    println!("terminate_game");
                    terminate_running_exe(&bot_config.game_executable);
                }
                Instruction::UserChatMessage { author, message } => {
                    println!("user_chat_message");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    send_user_chat(&author, &message);
                }
                Instruction::Wait { amount_ms } => {
                    println!("wait {}", amount_ms);
                    let duration = tokio::time::Duration::from_millis(*amount_ms);
                    tokio::time::sleep(duration).await;
                }
                Instruction::WaitWithMessage { amount_ms, message } => {
                    println!("wait_with_message {} {}", amount_ms, message);

                    match hud_sender.send(HUDInstruction::TimedMessage {
                        message: message.to_string(),
                        time: amount_ms.to_owned(),
                    }) {
                        Err(_e) => eprintln!("HUD Channel Error"),
                        _ => (),
                    }
                    let duration = tokio::time::Duration::from_millis(*amount_ms);
                    tokio::time::sleep(duration).await;
                }
                Instruction::ZoomCamera {
                    direction,
                    duration,
                } => {
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    println!("zoom_camera");
                    camera_zoom(direction, *duration);
                }
                Instruction::JoinServer { server_id } => {
                    // Deliberately synchronous/blocking
                    println!("join_server {}", &server_id);
                    join_server(bot_config.game_id.to_owned(), &server_id);
                    let loaded_in = cv_check_loaded_in(&bot_config.game_name.to_owned());
                    println!("Loaded in: {}", loaded_in);
                    if !loaded_in {
                        let _ = notify_admin("Failed to load in!").await;
                    }
                }
            }

            if !success {
                eprintln!("Failed instruction processing");
            }
        }
    }
}

pub fn get_warp_locations() -> (HashMap<String, String>, String) {
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
    tp_locations.insert(String::from("beach"), String::from("Beach"));
    tp_locations.insert(String::from("devil"), String::from("Scarlet Devil Mansion"));
    tp_locations.insert(String::from("highway"), String::from("Highway"));
    tp_locations.insert(String::from("rat"), String::from("Rat Sewers"));

    let valid_tp_locations = tp_locations
        .keys()
        .map(|s| &**s)
        .collect::<Vec<_>>()
        .join(", ");

    return (tp_locations, valid_tp_locations);
}

pub async fn twitch_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    hud_sender: UnboundedSender<HUDInstruction>,
    bot_config: BotConfig,
) {
    let twitch_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_config.twitch_bot_username.to_owned(),
        Some(bot_config.twitch_bot_token.to_owned()),
    ));
    let (mut incoming_messages, raw_client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(twitch_config);

    let rc_client = Rc::new(raw_client);
    let client = &rc_client;

    //Begin Async Loop
    match client.join(bot_config.twitch_channel_name.to_owned()) {
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
                        "ping" => {
                            match hud_sender.send(HUDInstruction::GenericMessage {
                                message: "ping".to_string(),
                            }) {
                                Err(_e) => eprintln!("HUD Channel Error"),
                                _ => (),
                            }
                            client
                                .reply_to_privmsg(String::from("pong"), &msg)
                                .await
                                .unwrap();
                        }
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
                            let _discord_webook_result =
                                discord_log(&message, &msg.sender.name, true).await;

                            if (message.starts_with("/") && !(message.starts_with("/e")))
                                || message.starts_with("[")
                            {
                                client
                                    .reply_to_privmsg(
                                        String::from("[You cannot run commands other than /e!]"),
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

                            let success = check_active(&bot_config.game_name);
                            if !success {
                                let _ = notify_admin("Failed to find Roblox!").await;
                                client
                                    .reply_to_privmsg(
                                        String::from("[Failed to find Roblox! Notified dev.]"),
                                        &msg,
                                    )
                                    .await
                                    .unwrap();
                                continue;
                            }
                            if message.starts_with("/") {
                                println!("Sending chat command from '{}'\n{}", author_msg, message);
                                let chat_command_instructions = SystemInstruction {
                                    client: Some(client.clone()),
                                    chat_message: Some(msg.to_owned()),
                                    instructions: vec![
                                        InstructionPair {
                                            execution_order: 0,
                                            instruction: Instruction::CheckActive {
                                                window_title: bot_config.game_name.to_owned(),
                                            },
                                        },
                                        InstructionPair {
                                            execution_order: 1,
                                            instruction: Instruction::SystemChatMessage {
                                                message: message,
                                            },
                                        },
                                    ],
                                };
                                match queue_sender.send(chat_command_instructions) {
                                    Err(_e) => eprintln!("Chat Command Channel Error"),
                                    _ => (),
                                }
                            } else {
                                println!("Sending chat message\n{}: {}", author_msg, message);
                                let chat_command_instructions = SystemInstruction {
                                    client: Some(client.clone()),
                                    chat_message: Some(msg.to_owned()),
                                    instructions: vec![
                                        InstructionPair {
                                            execution_order: 0,
                                            instruction: Instruction::CheckActive {
                                                window_title: bot_config.game_name.to_owned(),
                                            },
                                        },
                                        InstructionPair {
                                            execution_order: 1,
                                            instruction: Instruction::UserChatMessage {
                                                message: message,
                                                author: author_msg,
                                            },
                                        },
                                    ],
                                };
                                match queue_sender.send(chat_command_instructions) {
                                    Err(_e) => eprintln!("User Chat Channel Error"),
                                    _ => (),
                                }
                            }
                        }
                        "dev" => {
                            let message = format!(
                                "{}: {}",
                                &msg.sender.name,
                                args.replacen("dev ", "", 1).replacen("dev", "", 1),
                            );
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

                            let move_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::MoveDirection {
                                            direction: direction,
                                            duration: amount,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(move_instructions) {
                                Err(_e) => eprintln!("MoveDirection Channel Error"),
                                _ => (),
                            }
                        }
                        "warp" => {
                            if clean_args.len() < 2 {
                                client
                                    .reply_to_privmsg(
                                        format!("[Specify a location! Map: https://sbf.fumocam.xyz/warp-map (Valid locations: {})]", bot_config.valid_tp_locations),
                                        &msg,
                                    )
                                    .await
                                    .unwrap();
                                continue;
                            }
                            let desired_location = bot_config.tp_locations.get(clean_args[1]);

                            match desired_location {
                                Some(desired_location) => {
                                    let warp_instructions = SystemInstruction {
                                        client: Some(client.clone()),
                                        chat_message: Some(msg.to_owned()),
                                        instructions: vec![
                                            InstructionPair {
                                                execution_order: 0,
                                                instruction: Instruction::CheckActive {
                                                    window_title: bot_config.game_name.to_owned(),
                                                },
                                            },
                                            InstructionPair {
                                                execution_order: 1,
                                                instruction: Instruction::SystemChatMessage {
                                                    message: format!(
                                                        "[Warping to {}!]",
                                                        desired_location
                                                    )
                                                    .to_string(),
                                                },
                                            },
                                            InstructionPair {
                                                execution_order: 1,
                                                instruction: Instruction::Leap {
                                                    // Clear any sitting effects
                                                    forward_amount: 1.0,
                                                    spacebar_amount: 1.0,
                                                },
                                            },
                                            InstructionPair {
                                                execution_order: 1,
                                                instruction: Instruction::ConsoleCommand {
                                                    command: format!("warp {}", desired_location)
                                                        .to_string(),
                                                },
                                            },
                                        ],
                                    };
                                    match queue_sender.send(warp_instructions) {
                                        Err(_e) => eprintln!("Warp Channel Error"),
                                        _ => (),
                                    }
                                }
                                None => {
                                    client.reply_to_privmsg(format!("[{} is not a valid location! Map: https://sbf.fumocam.xyz/warp-map (Valid locations: {})]", clean_args[1], bot_config.valid_tp_locations), &msg).await.unwrap();
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
                            #[allow(unused_assignments)]
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
                            let camera_left_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::MoveCameraX { x_ratio: amount },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_left_instructions) {
                                Err(_e) => eprintln!("Camera Left Channel Error"),
                                _ => (),
                            }
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
                            #[allow(unused_assignments)]
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
                            let camera_right_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::MoveCameraX {
                                            x_ratio: amount * -1.0,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_right_instructions) {
                                Err(_e) => eprintln!("Camera Right Channel Error"),
                                _ => (),
                            }
                        }
                        "up" => {
                            if clean_args.len() < 2 {
                                client
                                    .reply_to_privmsg(
                                        String::from("[Specify degrees to rotate! (i.e. !up 45)]"),
                                        &msg,
                                    )
                                    .await
                                    .unwrap();
                                continue;
                            }
                            const MAX_AMOUNT: f32 = 180.0;
                            #[allow(unused_assignments)]
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
                            let camera_up_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::MoveCameraY { y_ratio: amount },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_up_instructions) {
                                Err(_e) => eprintln!("Camera Up Channel Error"),
                                _ => (),
                            }
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
                            #[allow(unused_assignments)]
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
                            let camera_down_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::MoveCameraY {
                                            y_ratio: amount * -1.0,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_down_instructions) {
                                Err(_e) => eprintln!("Camera Down Channel Error"),
                                _ => (),
                            }
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
                            #[allow(unused_assignments)]
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
                            let camera_zoom_in_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ZoomCamera {
                                            direction: "i".to_string(),
                                            duration: amount,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_zoom_in_instructions) {
                                Err(_e) => eprintln!("Camera Zoom In Channel Error"),
                                _ => (),
                            }
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
                            #[allow(unused_assignments)]
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
                            let camera_zoom_out_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ZoomCamera {
                                            direction: "o".to_string(),
                                            duration: amount,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(camera_zoom_out_instructions) {
                                Err(_e) => eprintln!("Camera Zoom Out Channel Error"),
                                _ => (),
                            }
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
                            let leap_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::Leap {
                                            forward_amount: forward_amount,
                                            spacebar_amount: spacebar_amount,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(leap_instructions) {
                                Err(_e) => eprintln!("Leap Channel Error"),
                                _ => (),
                            }
                        }
                        "hidemouse" => {
                            let hide_mouse_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::HideMouse {},
                                }],
                            };
                            match queue_sender.send(hide_mouse_instructions) {
                                Err(_e) => eprintln!("Hide Mouse Channel Error"),
                                _ => (),
                            }
                        }
                        "jump" => {
                            let jump_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::Leap {
                                            forward_amount: 0.0,
                                            spacebar_amount: 1.0,
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(jump_instructions) {
                                Err(_e) => eprintln!("Jump Channel Error"),
                                _ => (),
                            }
                        }
                        "grief" => {
                            let jump_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::Grief {},
                                    },
                                ],
                            };
                            match queue_sender.send(jump_instructions) {
                                Err(_e) => eprintln!("Grief Channel Error"),
                                _ => (),
                            }
                        }
                        "refresh" => {
                            let refresh_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: "re".to_string(),
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(refresh_instructions) {
                                Err(_e) => eprintln!("Refresh Channel Error"),
                                _ => (),
                            }
                        }
                        "die" => {
                            let die_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: "die".to_string(),
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(die_instructions) {
                                Err(_e) => eprintln!("Die Channel Error"),
                                _ => (),
                            }
                        }
                        "explode" => {
                            let explode_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: "explode".to_string(),
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(explode_instructions) {
                                Err(_e) => eprintln!("Explode Channel Error"),
                                _ => (),
                            }
                        }
                        "sit" => {
                            let sit_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::Sit {},
                                    },
                                ],
                            };
                            match queue_sender.send(sit_instructions) {
                                Err(_e) => eprintln!("Sit Channel Error"),
                                _ => (),
                            }
                        }
                        "size" => {
                            if clean_args.len() < 2 {
                                client
                                    .reply_to_privmsg(
                                        String::from("[Specify a size! The default is 'base'. (doll, shimmy, base, deka)]"),
                                        &msg,
                                    )
                                    .await
                                    .unwrap();
                                continue;
                            }
                            let valid_sizes = vec![
                                String::from("base"),
                                String::from("shimmy"),
                                String::from("doll"),
                                String::from("deka"),
                            ];
                            let size = clean_args[1].to_lowercase();
                            if !valid_sizes.contains(&size) {
                                client
                                    .reply_to_privmsg(
                                        String::from("[Invalid size! Specify a size, the default is 'base'. (doll, shimmy, base, deka)]"),
                                        &msg,
                                    )
                                    .await
                                    .unwrap();
                                continue;
                            }
                            let size_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.to_owned()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.to_owned(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: format!(
                                                "changefumo SBFCam Momiji {}",
                                                capitalize_string(&size) // console is case sensitive
                                            ),
                                        },
                                    },
                                ],
                            };
                            match queue_sender.send(size_instructions) {
                                Err(_e) => eprintln!("Size Channel Error"),
                                _ => (),
                            }
                        }
                        "rejoin" => {
                            let mod_1 = env::var("TWITCH_MOD_1")
                                .expect("$TWITCH_MOD_1 is not set")
                                .to_lowercase();

                            let author_name = msg.sender.name.to_string().to_lowercase();
                            let message: &str;
                            if author_name.to_lowercase() == mod_1.to_lowercase() {
                                let result =
                                    force_rejoin(queue_sender.clone(), bot_config.clone()).await;
                                if result {
                                    message = "[Rejoin queued successfully!]";
                                } else {
                                    message = "[Failed to queue rejoin, Roblox API may be down!]";
                                }
                            } else {
                                message = "[You do not have permissions to run this!]";
                            }
                            client
                                .reply_to_privmsg(message.to_string(), &msg)
                                .await
                                .unwrap();
                        }
                        _ => {
                            client
                                .reply_to_privmsg(
                                    String::from(
                                        "[Not a valid command! Type !help for a list of commands.]",
                                    ),
                                    &msg,
                                )
                                .await
                                .unwrap();
                        }
                    }
                }

                let _result = discord_log(&msg.message_text, &msg.sender.name, false).await;
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct BotConfig {
    game_name: String,
    game_executable: String,
    game_id: i64,
    tp_locations: HashMap<String, String>,
    twitch_bot_username: String,
    twitch_bot_token: String,
    player_token: String,
    twitch_channel_name: String,
    valid_tp_locations: String,
}

pub fn init_config() -> BotConfig {
    let game_name: String = "Roblox".to_string();
    let game_executable: String = "RobloxPlayerBeta.exe".to_string();
    let game_id: i64 = 7363647365;
    let twitch_channel_name: String = "sbfcam".to_string();
    let twitch_bot_username: String = "sbfcamBOT".to_string();

    let (tp_locations, valid_tp_locations) = get_warp_locations();

    //Oauth (generated with https://twitchtokengenerator.com/):
    let twitch_bot_token = env::var("TWITCH_BOT_TOKEN").expect("$TWITCH_BOT_TOKEN is not set");
    let player_token = env::var("PLAYER_TOKEN").expect("$PLAYER_TOKEN is not set");

    let bot_config = BotConfig {
        game_name,
        game_executable,
        game_id,
        tp_locations: tp_locations,
        twitch_bot_username,
        twitch_bot_token: twitch_bot_token,
        player_token: player_token,
        twitch_channel_name,
        valid_tp_locations: valid_tp_locations,
    };
    return bot_config;
}

pub async fn anti_afk_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
) {
    let interval_minutes = 10;
    let mut interval = tokio::time::interval(Duration::from_millis(interval_minutes * 60 * 1000));
    interval.tick().await;
    loop {
        for _ in 0..3 {
            interval.tick().await;

            let anti_afk_instructions = SystemInstruction {
                client: None,
                chat_message: None,
                instructions: vec![
                    InstructionPair {
                        execution_order: 0,
                        instruction: Instruction::CheckActive {
                            window_title: bot_config.game_name.to_owned(),
                        },
                    },
                    InstructionPair {
                        execution_order: 1,
                        instruction: Instruction::MoveCameraX { x_ratio: 45.0 },
                    },
                    InstructionPair {
                        execution_order: 2,
                        instruction: Instruction::Wait { amount_ms: 500 },
                    },
                    InstructionPair {
                        execution_order: 3,
                        instruction: Instruction::MoveCameraX { x_ratio: -90.0 },
                    },
                    InstructionPair {
                        execution_order: 4,
                        instruction: Instruction::Wait { amount_ms: 500 },
                    },
                    InstructionPair {
                        execution_order: 5,
                        instruction: Instruction::MoveCameraX { x_ratio: 45.0 },
                    },
                ],
            };
            match queue_sender.send(anti_afk_instructions) {
                Err(_e) => eprintln!("Anti-AFK Channel Error"),
                _ => (),
            }
        }

        let advert_instructions = SystemInstruction {
            client: None,
            chat_message: None,
            instructions: vec![
                InstructionPair {
                    execution_order: 0,
                    instruction: Instruction::CheckActive {
                        window_title: bot_config.game_name.to_owned(),
                    },
                },
                InstructionPair {
                    execution_order: 1,
                    instruction: Instruction::SystemChatMessage { message: "You can control this bot live on T witch! Go to t witch.tv and and search my username (without the underscore/number)".to_string()},
                },
            ],
        };
        match queue_sender.send(advert_instructions) {
            Err(_e) => eprintln!("Anti-AFK Channel Error"),
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameInstance {
    id: String,
    players: Vec<String>,
}
const UNKNOWN_SERVER_ID: &str = "UNKNOWN";

pub async fn get_instances(game_id: i64) -> Result<Option<Vec<GameInstance>>, Box<dyn Error>> {
    let api_url: String = format!(
        "https://games.roblox.com/v1/games/{game_id}/servers/Public",
        game_id = game_id
    );
    let response = reqwest::get(api_url).await?;

    if !(&response.status().is_success()) {
        eprint!("Error\n{}", response.text().await?);
        return Ok(Option::None);
    }
    let body = response.text().await?;
    // println!("{}", body);

    // TODO: This json line below can silently stop further code execution, if ": serde_json::Value" isnt used. Scary!
    let body_json: serde_json::Value = serde_json::from_str(&body)?;

    let debug_body = serde_json::to_string_pretty(&body_json).unwrap();
    println!("Body:\n{}", debug_body);

    if !(body_json.is_object()) {
        eprint!("Error, body not an object\n{}", body);
        return Ok(Option::None);
    }
    let body_obj = body_json.as_object().unwrap();
    if !(body_obj.contains_key("data")) {
        eprint!("Error, missing data\n{}", body);
        return Ok(Option::None);
    }
    let response_data = body_obj.get("data").unwrap();
    if !(response_data.is_array()) {
        eprint!("Error, data not array\n{}", body);
        return Ok(Option::None);
    }
    let data = response_data.as_array().unwrap();
    let mut instance_list = Vec::new();
    for instance in data {
        let place = instance.as_object().unwrap();
        let players_raw = place.get("playerTokens").unwrap().as_array().unwrap();
        let mut players = Vec::new();
        for player in players_raw {
            players.push(player.as_str().unwrap().to_string());
        }
        let instance = GameInstance {
            id: place.get("id").unwrap().as_str().unwrap().to_string(),
            players: players,
        };
        instance_list.push(instance)
    }

    Ok(Some(instance_list))
}

async fn get_current_server(
    player_token: String,
    instance_list: Vec<GameInstance>,
) -> GameInstance {
    for instance in instance_list {
        println!("{}", instance.id);
        if instance.players.contains(&player_token) {
            return instance;
        }
    }
    return GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    };
}

async fn check_in_server(player_token: String, instance_list: Vec<GameInstance>) -> bool {
    let current_server = get_current_server(player_token, instance_list).await;
    return current_server.id != UNKNOWN_SERVER_ID.to_string();
}

async fn get_best_server(instance_list: Vec<GameInstance>) -> GameInstance {
    let mut best_server = GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    };
    for instance in instance_list {
        if best_server.players.len() < instance.players.len() {
            best_server = instance;
        }
    }
    return best_server;
}

async fn check_in_best_server(player_token: String, instance_list: Vec<GameInstance>) -> bool {
    const LOW_SWITCH: usize = 2;
    const LOW_SWITCH_THRESH: usize = 7;
    const HIGH_SWITCH: usize = 5;
    let best_server = get_best_server(instance_list.clone()).await;
    if best_server.id == UNKNOWN_SERVER_ID.to_string() {
        // Somehow, our instance list is empty. Stay in current server for safety.
        return true;
    }

    let mut current_server = GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    };
    for instance in instance_list {
        println!("{}", instance.id);
        if instance.players.contains(&player_token) {
            current_server = instance.clone();
            break;
        }
    }
    if best_server.id == UNKNOWN_SERVER_ID.to_string() {
        // Somehow, our instance list is empty. Stay in current server for safety.
        return true;
    }

    // If the best server doesn't have more than LOW_SWITCH_THRESH,
    // A server would need LOW_SWITCH more players to trigger a switch.
    // Otherwise, a server would need HIGH_SWITCH more players to trigger a switch.

    let required_difference = if best_server.players.len() <= LOW_SWITCH_THRESH {
        LOW_SWITCH
    } else {
        HIGH_SWITCH
    };
    let difference = best_server.players.len() - current_server.players.len();
    println!("[Server difference {}]", difference);
    println!("[Required switch difference {}]", required_difference);
    return difference <= required_difference;
}

async fn _force_rejoin(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
    instance_list: Vec<GameInstance>,
) {
    let best_server = get_best_server(instance_list.clone()).await;
    let current_server =
        get_current_server(bot_config.player_token.to_owned(), instance_list.clone()).await;

    let mut instructions: Vec<InstructionPair> = vec![];
    if current_server.id != UNKNOWN_SERVER_ID && check_active(&bot_config.game_name.to_owned()) {
        instructions.push(InstructionPair {
            execution_order: 0,
            instruction: Instruction::SystemChatMessage {
                message: "[Force-rejoin queued! Be right back!]".to_string(),
            },
        });
        instructions.push(InstructionPair {
            execution_order: 1,
            instruction: Instruction::Wait { amount_ms: 7000 },
        });
    }
    instructions.push(InstructionPair {
        execution_order: 2,
        instruction: Instruction::TerminateGame {},
    });
    instructions.push(InstructionPair {
        execution_order: 3,
        instruction: Instruction::WaitWithMessage {
            amount_ms: 10000,
            message: String::from("ABIDING ROBLOX RATE-LIMIT"),
        },
    });
    instructions.push(InstructionPair {
        execution_order: 4,
        instruction: Instruction::JoinServer {
            server_id: best_server.id.to_owned(),
        },
    });
    instructions.push(InstructionPair {
        execution_order: 5,
        instruction: Instruction::CheckActive {
            window_title: bot_config.game_name.to_owned(),
        },
    });
    instructions.push(InstructionPair {
        execution_order: 6,
        instruction: Instruction::SystemChatMessage {
            message: "[Crash recovery complete! Ready to accept commands.]".to_string(),
        },
    });

    let join_instruction = SystemInstruction {
        client: None,
        chat_message: None,
        instructions: instructions,
    };
    match queue_sender.send(join_instruction) {
        Err(_e) => eprintln!("JoinServer Channel Error"),
        _ => (),
    }
}

async fn force_rejoin(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
) -> bool {
    let get_instances_result = get_instances(bot_config.game_id).await;
    match get_instances_result {
        Ok(instance_list_option) => match instance_list_option {
            Some(instance_list) => {
                let best_server = get_best_server(instance_list.clone()).await;
                if best_server.id == UNKNOWN_SERVER_ID.to_string() {
                    eprintln!("[Roblox API check failed]");
                    return false;
                } else {
                    _force_rejoin(
                        queue_sender.clone(),
                        bot_config.clone(),
                        instance_list.clone(),
                    )
                    .await;
                    return true;
                }
            }
            None => {
                eprintln!("[Roblox API check failed]");
                return false;
            }
        },
        Err(_error) => {
            eprintln!("[Roblox API check failed]");
            return false;
        }
    }
}

async fn server_check_logic(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
    instance_list: Vec<GameInstance>,
) {
    let in_server =
        check_in_server(bot_config.player_token.to_owned(), instance_list.clone()).await;
    let in_best_server = if in_server {
        check_in_best_server(bot_config.player_token.to_owned(), instance_list.clone()).await
    } else {
        false
    };

    if in_server && in_best_server {
        return;
    }

    let best_server = get_best_server(instance_list.clone()).await;
    let current_server =
        get_current_server(bot_config.player_token.to_owned(), instance_list.clone()).await;
    let difference = best_server.players.len() - current_server.players.len();

    let mut instructions = vec![
        InstructionPair {
            execution_order: 0,
            instruction: Instruction::CheckActive {
                window_title: bot_config.game_name.to_owned(),
            },
        },
        InstructionPair {
            execution_order: 1,
            instruction: Instruction::SystemChatMessage {
                message: format!(
                    "[Moving servers! There is a server with {} more players. See you there!]",
                    difference
                ),
            },
        },
        InstructionPair {
            execution_order: 2,
            instruction: Instruction::Wait { amount_ms: 7000 },
        },
        InstructionPair {
            execution_order: 3,
            instruction: Instruction::TerminateGame {},
        },
        InstructionPair {
            execution_order: 4,
            instruction: Instruction::WaitWithMessage {
                amount_ms: 10000,
                message: String::from("ABIDING ROBLOX RATE-LIMIT"),
            },
        },
        InstructionPair {
            execution_order: 5,
            instruction: Instruction::JoinServer {
                server_id: best_server.id.to_owned(),
            },
        },
        InstructionPair {
            execution_order: 6,
            instruction: Instruction::CheckActive {
                window_title: bot_config.game_name.to_owned(),
            },
        },
        InstructionPair {
            execution_order: 7,
            instruction: Instruction::SystemChatMessage {
                message: "[Server relocation complete! Ready to accept commands.]".to_string(),
            },
        },
    ];
    if !(in_server) {
        // Crash likely
        println!("Potential crash detected");

        let is_active = check_active(&bot_config.game_name.to_owned());
        let exe_status = if is_active { "Running" } else { "Not found" };
        let message = format!("Likely crash detected | {}", exe_status);
        _ = notify_admin(&message).await;

        instructions = vec![
            InstructionPair {
                execution_order: 0,
                instruction: Instruction::TerminateGame {},
            },
            InstructionPair {
                execution_order: 1,
                instruction: Instruction::WaitWithMessage {
                    amount_ms: 10000,
                    message: String::from("ABIDING ROBLOX RATE-LIMIT"),
                },
            },
            InstructionPair {
                execution_order: 2,
                instruction: Instruction::JoinServer {
                    server_id: best_server.id.to_owned(),
                },
            },
            InstructionPair {
                execution_order: 3,
                instruction: Instruction::CheckActive {
                    window_title: bot_config.game_name.to_owned(),
                },
            },
            InstructionPair {
                execution_order: 4,
                instruction: Instruction::SystemChatMessage {
                    message: "[Crash recovery complete! Ready to accept commands.]".to_string(),
                },
            },
        ];
    }

    let join_instruction = SystemInstruction {
        client: None,
        chat_message: None,
        instructions: instructions,
    };
    match queue_sender.send(join_instruction) {
        Err(_e) => eprintln!("JoinServer Channel Error"),
        _ => (),
    }
}

async fn server_check_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
) {
    let interval_minutes = 3;
    let mut interval = tokio::time::interval(Duration::from_millis(interval_minutes * 60 * 1000));
    loop {
        interval.tick().await;
        let get_instances_result = get_instances(bot_config.game_id).await;
        match get_instances_result {
            Ok(instance_list_option) => match instance_list_option {
                Some(instance_list) => {
                    let best_server = get_best_server(instance_list.clone()).await;
                    if best_server.id == UNKNOWN_SERVER_ID.to_string() {
                        eprint!("[Roblox API check failed]")
                    } else {
                        server_check_logic(
                            queue_sender.clone(),
                            bot_config.clone(),
                            instance_list.clone(),
                        )
                        .await;
                    }
                }
                None => {
                    eprintln!("[Roblox API check failed]");
                }
            },
            Err(_error) => {
                eprintln!("[Roblox API check failed]");
            }
        }
    }
}

fn join_game_selenium(
    game_id: i64,
    instance_id: &str,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    println!("Selenium subprocess started");
    let driver = match env::var("CHROME_DRIVER_FILE_NAME") {
        Ok(val) => val,
        Err(_e) => "chromedriver.exe".to_string(),
    };
    let output = Command::new("cmd")
        .args([
            "/C",
            "poetry",
            "run",
            "python",
            "join_game.py",
            "--game",
            &game_id.to_string(),
            "--instance",
            &instance_id,
            "--driver",
            &driver,
        ])
        .current_dir("../python")
        .output()
        .expect("failed to execute process");
    println!("Selenium subprocess finished");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

pub async fn restart_warn(restart_warn_ticker: i32) -> i32 {
    let now = chrono::offset::Local::now();
    if now.hour() != 3 {
        return -1;
    }
    const FIVE_MINUTE_WARNING: i32 = 55;
    const ONE_MINUTE_WARNING: i32 = 58;
    const REBOOT_TIME: i32 = 59;
    let minute = now.minute() as i32;

    if restart_warn_ticker == minute || minute < FIVE_MINUTE_WARNING - 1 {
        return -1;
    }

    // Don't do anything if our uptime is less than two minutes
    unsafe {
        const TWO_MINUTES: u64 = 120000; // 1000 * 60 * 2, Could be one minute, but safety padding is nice
        if winapi::um::sysinfoapi::GetTickCount64() < TWO_MINUTES {
            return -1;
        }
    }

    if minute == FIVE_MINUTE_WARNING {
        return FIVE_MINUTE_WARNING;
    } else if minute == ONE_MINUTE_WARNING {
        return ONE_MINUTE_WARNING;
    } else if minute == REBOOT_TIME {
        return REBOOT_TIME;
    } else {
        return -1;
    };
}

pub async fn restart_logic(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
    restart_return: i32,
) {
    let message = match restart_return {
        55 => "[Restarting in 5 minutes!]",
        58 => "[Restarting in 1 minute!]",
        59 => "[Restarting!]",
        _ => "[Restarting soon!]", // ???
    };
    let mut instructions = vec![
        InstructionPair {
            execution_order: 0,
            instruction: Instruction::CheckActive {
                window_title: bot_config.game_name.to_owned(),
            },
        },
        InstructionPair {
            execution_order: 1,
            instruction: Instruction::SystemChatMessage {
                message: message.to_string(),
            },
        },
    ];
    if restart_return == 59 {
        // Add restart to the instructions
        instructions.push(InstructionPair {
            execution_order: 2,
            instruction: Instruction::Wait { amount_ms: 1000 },
        });
        instructions.push(InstructionPair {
            execution_order: 3,
            instruction: Instruction::Restart {},
        });
    }
    let restart_system_instruction = SystemInstruction {
        client: None,
        chat_message: None,
        instructions: instructions,
    };
    match queue_sender.send(restart_system_instruction) {
        Err(_e) => eprintln!("Restart chat command error"),
        _ => (),
    }
}

// Start Temperature/OpenHardwareMonitor Code
#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
struct OpenHardwareMonitorSensorValue {
    id: i32,
    Text: String,
    // Children: Vec<serde_json::Value>,
    Min: String,
    Value: String,
    Max: String,
    ImageURL: String,
}
#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
struct OpenHardwareMonitorSensorType {
    id: i32,
    Text: String,
    Children: Vec<OpenHardwareMonitorSensorValue>,
    Min: String,
    Value: String,
    Max: String,
    ImageURL: String,
}
#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
struct OpenHardwareMonitorHardware {
    id: i32,
    Text: String,
    Children: Vec<OpenHardwareMonitorSensorType>,
    Min: String,
    Value: String,
    Max: String,
    ImageURL: String,
}
#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
struct OpenHardwareMonitorComputer {
    id: i32,
    Text: String,
    Children: Vec<OpenHardwareMonitorHardware>,
    Min: String,
    Value: String,
    Max: String,
    ImageURL: String,
}
#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
struct OpenHardwareMonitorBase {
    id: i32,
    Text: String,
    Children: Vec<OpenHardwareMonitorComputer>,
    Min: String,
    Value: String,
    Max: String,
    ImageURL: String,
}

pub async fn get_hardware_data() -> Result<Option<String>, Box<dyn Error>> {
    // Assumes a running OpenHardwareMonitor reporting server. Ring0 in rust is not worth it.
    // https://openhardwaremonitor.org/downloads/
    let api_url: String = String::from("http://localhost:8085/data.json");
    let response = reqwest::get(api_url).await?;

    if !(&response.status().is_success()) {
        eprint!("Error\n{}", response.text().await?);
        return Ok(Option::None);
    }
    let body = response.text().await?;

    match serde_json::from_str::<OpenHardwareMonitorBase>(&body) {
        Ok(report_data) => {
            let mut cpu_temp = "";
            let mut cpu_load = "";
            let hardware_list = &report_data.Children[0].Children;
            for hardware_item in hardware_list {
                let sensor_types = &hardware_item.Children;
                for sensor_type in sensor_types {
                    if sensor_type.Text != "Temperatures" && sensor_type.Text != "Load" {
                        continue;
                    }
                    let sensors = &sensor_type.Children;
                    for sensor in sensors {
                        if sensor.Text == "CPU Package" {
                            cpu_temp = &sensor.Value;
                        } else if sensor.Text == "CPU Total" {
                            cpu_load = &sensor.Value;
                        } else if !cpu_temp.is_empty() && !cpu_load.is_empty() {
                            // We got what we wanted
                            break;
                        }
                    }
                }
            }
            if cpu_temp.is_empty() {
                cpu_temp = "ERR";
            }
            if cpu_load.is_empty() {
                cpu_load = "ERR";
            }
            let response_json = json!(
                {
                    "cpu_temp": cpu_temp.to_string(),
                    "cpu_load": cpu_load.to_string()
                }
            );
            let response_json_str = serde_json::to_string(&response_json).unwrap();
            return Ok(Some(response_json_str));
        }
        Err(e) => {
            println!("{}", body);
            eprintln!("Error! {:#?}", e);
            return Ok(Option::None);
        }
    }
}

pub async fn clock_tick_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    hud_sender: UnboundedSender<HUDInstruction>,
    bot_config: BotConfig,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    let mut restart_warn_ticker = -1; // TODO: This needs a way cleaner system
    let hw_stat_error_val = String::from("{'cpu_temp': 'ERR', 'cpu_load': 'ERR'}");
    let mut hw_stat_loop = 0;
    const HW_STAT_LOOP_MAX: i32 = 2;
    loop {
        interval.tick().await;

        // Restart logic
        let restart_return = restart_warn(restart_warn_ticker).await;
        if restart_warn_ticker != restart_return && restart_return != -1 {
            restart_warn_ticker = restart_return;
            restart_logic(queue_sender.clone(), bot_config.clone(), restart_return).await;
        }

        // Hardware stats logic
        if hw_stat_loop < HW_STAT_LOOP_MAX {
            // Only check for/send updates every HW_STAT_LOOP_MAX seconds
            hw_stat_loop = hw_stat_loop + 1;
            continue;
        } else {
            hw_stat_loop = 0;
        }
        let hw_stat_result = get_hardware_data().await;
        let hw_stat_data: String;
        match hw_stat_result {
            Ok(valid_result) => match valid_result {
                Some(result) => {
                    hw_stat_data = result;
                }
                None => {
                    eprintln!("[Open Hardware Monitor query failed]");
                    hw_stat_data = hw_stat_error_val.to_owned();
                }
            },
            Err(_error) => {
                eprintln!("[Open Hardware Monitor query error]\n{}", _error);
                hw_stat_data = hw_stat_error_val.to_owned();
            }
        }
        match hud_sender.send(HUDInstruction::SystemMonitorUpdate { data: hw_stat_data }) {
            Err(_e) => eprintln!("[HUD] System Monitor Update Channel Error"),
            _ => (),
        }
    }
}

fn get_pixel(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    red: i32,
    green: i32,
    blue: i32,
) -> bool {
    println!("Get Pixel subprocess started");
    let detect_value = format!("{},{},{}", &red, &green, &blue);
    let output = Command::new("cmd")
        .args([
            "/C",
            "poetry",
            "run",
            "python",
            "cv_functions.py",
            "--func",
            "detect_pixels",
            "--left",
            &left.to_string(),
            "--top",
            &top.to_string(),
            "--width",
            &width.to_string(),
            "--height",
            &height.to_string(),
            "--detect_value",
            &detect_value,
        ])
        .current_dir("../python")
        .output()
        .expect("failed to execute process");
    println!("Get Pixel subprocess finished");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    match serde_json::from_str::<bool>(&String::from_utf8_lossy(&output.stdout)) {
        Ok(return_val) => {
            println!("Pixel Response: {}", return_val);
            return return_val;
        }
        Err(e) => {
            eprintln!("Error! {:#?}", e);
            return false;
        }
    }
}

fn cv_get_backpack_hover(window_title: &str) -> bool {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.47, 0.95);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.47, 0.95);
    return get_pixel(690, 669, 10, 5, 173, 173, 173);
}
fn cv_get_navbar(window_title: &str) -> bool {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.47, 0.99);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.47, 0.99);
    return get_pixel(690, 669, 10, 5, 246, 246, 246);
}
fn cv_get_navbar_hidden(window_title: &str) -> bool {
    check_active(&window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_hide(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
    return !(get_pixel(690, 669, 10, 5, 246, 246, 246));
}

fn cv_check_loaded_in(window_title: &str) -> bool {
    let max_seconds = 120;
    let delay_seconds = 2;
    let attempts = max_seconds / delay_seconds;
    let delay: Duration = Duration::from_millis((delay_seconds * 1000) as u64);

    let mut program_load_success = false;
    for _attempt in 0..attempts {
        if check_active(window_title) {
            program_load_success = true;
            break;
        }
        thread::sleep(delay);
    }
    if !program_load_success {
        return false;
    }

    let mut backpack_hover_success = false;
    for _attempt in 0..attempts {
        if cv_get_backpack_hover(window_title) {
            backpack_hover_success = true;
            break;
        }
        thread::sleep(delay);
    }
    if !backpack_hover_success {
        return false;
    }

    let mut get_navbar_success = false;
    for _attempt in 0..attempts {
        if cv_get_navbar(window_title) {
            get_navbar_success = true;
            break;
        }
        thread::sleep(delay);
    }
    if !get_navbar_success {
        return false;
    }

    let mut get_navbar_hidden_success = false;
    for _attempt in 0..attempts {
        if cv_get_navbar_hidden(window_title) {
            get_navbar_hidden_success = true;
            break;
        }
        thread::sleep(delay);
    }
    if !get_navbar_hidden_success {
        return false;
    }

    return true;
}

// START HUD STUFF
#[derive(Clone)]
pub enum HUDInstruction {
    AddClient {
        client_id: u64,
        responder: simple_websockets::Responder,
    },
    RemoveClient {
        client_id: u64,
    },
    ClientMessage {
        message: String,
    },
    ClientBinaryMessage {
        binary_message: Vec<u8>,
    },
    GenericMessage {
        message: String,
    },
    TimedMessage {
        message: String,
        time: u64,
    },
    SystemMonitorUpdate {
        data: String,
    },
}
pub async fn hud_loop(mut hud_receiver: UnboundedReceiver<HUDInstruction>) {
    let mut clients: HashMap<u64, simple_websockets::Responder> = HashMap::new();
    loop {
        let hud_instruction = hud_receiver.recv().await.unwrap();

        match hud_instruction {
            HUDInstruction::AddClient {
                client_id,
                responder,
            } => {
                clients.insert(client_id, responder);
            }
            HUDInstruction::RemoveClient { client_id } => {
                clients.remove(&client_id);
            }
            HUDInstruction::ClientMessage { message } => {
                println!("[HUD] ClientMessage: {}", message);
            }
            HUDInstruction::ClientBinaryMessage { binary_message } => {
                println!("[HUD] ClientBinaryMessage: {:?}", binary_message);
            }
            HUDInstruction::GenericMessage { message } => {
                println!(
                    "[HUD] Sending message to all connected clients ({}): {}",
                    clients.len(),
                    message
                );
                for (_client_id, client_responder) in &clients {
                    client_responder.send(simple_websockets::Message::Text(message.to_owned()));
                }
            }
            HUDInstruction::TimedMessage { message, time } => {
                println!(
                    "[HUD] Sending timed message to all connected clients ({}): {}",
                    clients.len(),
                    message
                );
                let message_obj = json!({
                    "type": "timed_message",
                    "value": {
                        "seconds": time,
                        "message": message
                    }
                });
                for (_client_id, client_responder) in &clients {
                    client_responder
                        .send(simple_websockets::Message::Text(message_obj.to_string()));
                }
            }
            HUDInstruction::SystemMonitorUpdate { data } => {
                // println!(
                //     "[HUD] Sending system monitor update to all connected clients ({}): {}",
                //     clients.len(),
                //     data
                // );
                let message_obj = json!({
                    "type": "system_monitor_update",
                    "value": {
                        "data": data
                    }
                });
                for (_client_id, client_responder) in &clients {
                    client_responder
                        .send(simple_websockets::Message::Text(message_obj.to_string()));
                }
            }
        }
    }
}
pub async fn hud_ws_server(hud_sender: UnboundedSender<HUDInstruction>) {
    let event_hub = simple_websockets::launch(8080).expect("[HUD] failed to listen on port 8080");
    loop {
        match event_hub.poll_async().await {
            simple_websockets::Event::Connect(client_id, responder) => {
                println!("[HUD] A client connected with id #{}", client_id);
                match hud_sender.send(HUDInstruction::AddClient {
                    client_id: client_id,
                    responder: responder,
                }) {
                    Err(_e) => eprintln!("[HUD] Client Connect Channel Error"),
                    _ => (),
                }
            }
            simple_websockets::Event::Disconnect(client_id) => {
                println!("[HUD] A client connected with id #{}", client_id);
                match hud_sender.send(HUDInstruction::RemoveClient {
                    client_id: client_id,
                }) {
                    Err(_e) => eprintln!("[HUD] Client Disconnect Channel Error"),
                    _ => (),
                }
            }
            simple_websockets::Event::Message(client_id, message) => {
                println!(
                    "[HUD] Received a message from client #{}: {:?}",
                    client_id, message
                );
                let hud_message_instruction = match message {
                    simple_websockets::Message::Text(text) => {
                        HUDInstruction::ClientMessage { message: text }
                    }
                    simple_websockets::Message::Binary(bytes) => {
                        HUDInstruction::ClientBinaryMessage {
                            binary_message: bytes,
                        }
                    }
                };
                match hud_sender.send(hud_message_instruction) {
                    Err(_e) => eprintln!("[HUD] Client Message Channel Error"),
                    _ => (),
                }
            }
        }
    }
}
// END HUD STUFF

// =================
// [Production Main]
// =================
#[tokio::main]
pub async fn main() {
    dotenv::from_filename("..\\.env").ok();

    let bot_config = init_config();

    let (hud_sender, hud_receiver): (
        UnboundedSender<HUDInstruction>,
        UnboundedReceiver<HUDInstruction>,
    ) = unbounded_channel();
    let (queue_sender, queue_receiver): (
        UnboundedSender<SystemInstruction>,
        UnboundedReceiver<SystemInstruction>,
    ) = unbounded_channel();

    let twitch_task = twitch_loop(queue_sender.clone(), hud_sender.clone(), bot_config.clone());
    let anti_afk_task = anti_afk_loop(queue_sender.clone(), bot_config.clone());
    let queue_processor_task =
        queue_processor(queue_receiver, hud_sender.clone(), bot_config.clone());
    let server_check_task = server_check_loop(queue_sender.clone(), bot_config.clone());
    let clock_tick_task =
        clock_tick_loop(queue_sender.clone(), hud_sender.clone(), bot_config.clone());
    let hud_loop_task = hud_loop(hud_receiver);
    let hud_ws_server_task = hud_ws_server(hud_sender.clone());

    check_active(&bot_config.game_name.to_owned());

    let (_r1, _r2, _r3, _r4, _r5, _r6, _r7) = tokio::join!(
        twitch_task,
        anti_afk_task,
        queue_processor_task,
        server_check_task,
        clock_tick_task,
        hud_loop_task,
        hud_ws_server_task
    );
}

//===========
//[Test Main]
//===========
// #[tokio::main]
// pub async fn main() {
//     dotenv::from_filename("..\\.env").ok();

//     let bot_config = init_config();

//     let (hud_sender, hud_receiver): (
//         UnboundedSender<HUDInstruction>,
//         UnboundedReceiver<HUDInstruction>,
//     ) = unbounded_channel();
//     let (queue_sender, queue_receiver): (
//         UnboundedSender<SystemInstruction>,
//         UnboundedReceiver<SystemInstruction>,
//     ) = unbounded_channel();

//     // let twitch_task = twitch_loop(queue_sender.clone(), hud_sender.clone(), bot_config.clone());
//     // let anti_afk_task = anti_afk_loop(queue_sender.clone(), bot_config.clone());
//     let queue_processor_task =
//         queue_processor(queue_receiver, hud_sender.clone(), bot_config.clone());
//     // let server_check_task = server_check_loop(queue_sender.clone(), bot_config.clone());
//     let clock_tick_task =
//         clock_tick_loop(queue_sender.clone(), hud_sender.clone(), bot_config.clone());
//     let hud_loop_task = hud_loop(hud_receiver);
//     let hud_ws_server_task = hud_ws_server(hud_sender.clone());

//     // check_active(&bot_config.game_name.to_owned());

//     let (
//         _r1,
//         _r2,
//         _r3,
//         _r4, // , _r5, _r6, _r7
//     ) = tokio::join!(
//         // twitch_task,
//         // anti_afk_task,
//         queue_processor_task,
//         // server_check_task,
//         clock_tick_task,
//         hud_loop_task,
//         hud_ws_server_task
//     );
// }