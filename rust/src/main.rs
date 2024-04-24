#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::items_after_statements
)]
use chrono::Timelike;
use enigo::{Enigo, Key, KeyboardControllable, MouseButton, MouseControllable};
use phf::phf_set;
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
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

static VALID_DIRECTIONS: phf::Set<char> = phf_set! {'w', 'a', 's', 'd'};

fn capitalize_string(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
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
    const EULER_MOUSEY_MULTI: f32 = 2.861_111; // 180 * this will rotate up/down 100%
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

fn leap(forward_amount: f64, spacebar_amount: f64, direction: char) {
    let forward_ms = (forward_amount * 1000.0).round() as u64;
    let spacebar_ms = (spacebar_amount * 1000.0).round() as u64;
    let mut enigo = Enigo::new();
    if forward_ms >= spacebar_ms {
        let forward_delay: Duration = Duration::from_millis(forward_ms - spacebar_ms);
        let spacebar_delay: Duration = Duration::from_millis(spacebar_ms);
        enigo.key_down(Key::Layout(direction));
        enigo.key_down(Key::Space);
        thread::sleep(spacebar_delay);
        enigo.key_up(Key::Space);
        thread::sleep(forward_delay);
        enigo.key_up(Key::Layout(direction));
    } else {
        let spacebar_delay: Duration = Duration::from_millis(spacebar_ms - forward_ms);
        let forward_delay: Duration = Duration::from_millis(forward_ms);
        enigo.key_down(Key::Layout(direction));
        enigo.key_down(Key::Space);
        thread::sleep(forward_delay);
        enigo.key_up(Key::Layout(direction));
        thread::sleep(spacebar_delay);
        enigo.key_up(Key::Space);
    }
}

fn navbar_grief() {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.62, 0.95);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

fn navbar_sit() {
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.25, 0.95);
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
    let suffixed_msg = format!("{msg} "); // Space suffix, to avoid cutoff
    let type_delay = Duration::from_millis(400);
    let send_delay = Duration::from_millis(150);

    enigo.key_click(Key::Layout('/'));
    thread::sleep(type_delay);
    if msg.starts_with('/') {
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

    // TODO: Undo, temp fix to combat spam safeguards
    // let suffixed_author = format!("{author} "); // Space suffix, to avoid cutoff
    let suffixed_msg = format!("{msg} "); // Space suffix, to avoid cutoff
    let total_msg = format!("{author} {suffixed_msg}");
    let type_delay = Duration::from_millis(400);
    let send_delay = Duration::from_millis(150);
    // let author_delay = Duration::from_millis(500);

    // // Author
    // enigo.key_click(Key::Layout('/'));
    // thread::sleep(type_delay);
    // enigo.key_sequence(&suffixed_author);
    // thread::sleep(send_delay);
    // enigo.key_click(Key::Return);

    // thread::sleep(author_delay);

    // Message
    enigo.key_click(Key::Layout('/'));
    thread::sleep(type_delay);
    enigo.key_sequence(&total_msg);
    thread::sleep(send_delay);
    enigo.key_click(Key::Return);
}

fn open_console_chat() {
    // Open console using chat commands
    const DELAY: Duration = Duration::from_millis(300);
    let mut enigo = Enigo::new();
    send_system_chat(&String::from("/sbfconsole"));
    mouse_move(&mut enigo, 0.5, 0.225);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
}

fn toggle_console_mouse(window_title: &str) {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.875, 0.03);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.875, 0.03);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    println!("hotfix_close_console");
}

fn click_console_input() {
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.5, 0.23);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.5, 0.23);
    thread::sleep(DELAY);
    mouse_click(&mut enigo);
    println!("hotfix_console_clicked");
}

fn run_console_command(window_title: &str, command: &str) {
    check_active(window_title);
    open_console_chat();
    // toggle_console_mouse(window_title);
    thread::sleep(Duration::from_millis(750));
    click_console_input();
    let mut enigo = Enigo::new();
    thread::sleep(Duration::from_millis(750));
    enigo.key_sequence(command);
    thread::sleep(Duration::from_millis(750));
    enigo.key_click(Key::Return);
    // Needs double click?
    thread::sleep(Duration::from_millis(750));
    toggle_console_mouse(window_title);
    thread::sleep(Duration::from_millis(750));
    toggle_console_mouse(window_title);
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
    let webhook_url = env::var(env_key).unwrap_or_else(|_| panic!("{env_key} is not set"));
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
async fn _send_admin_log(
    message: &str,
    do_ping: bool,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let webhook_url =
        env::var("DISCORD_ALERT_WEBHOOK_URL").expect("$DISCORD_ALERT_WEBHOOK_URL is not set");
    let author_discord_id = env::var("AUTHOR_DISCORD_ID").expect("$AUTHOR_DISCORD_ID is not set");
    let mut webhook_data = HashMap::new();
    let base_msg = format!("{message}\n<https://twitch.tv/sbfcam>");
    let mut content = base_msg.clone();
    if do_ping {
        content = format!("{base_msg}\n<@{author_discord_id}>");
    }
    webhook_data.insert("content", content);
    let client = reqwest::Client::new();
    let _resp = client.post(webhook_url).json(&webhook_data).send().await?;
    Ok(())
}
async fn notify_admin(
    message: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    _send_admin_log(message, true).await
}
async fn log_admin(
    message: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
    _send_admin_log(message, false).await
}
fn check_active(window_title: &str) -> bool {
    if get_active_window() != window_title {
        show_window_by_title(window_title);
        thread::sleep(Duration::from_millis(500));
        return get_active_window() == window_title;
    }
    true
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
    String::from_utf16_lossy(&buffer[0..writ_chars as usize])
}
fn show_window_by_title(title: &str) -> bool {
    let window_hwnd_ref_vec = windows_win::raw::window::get_by_title(title, None).unwrap();
    let window_hwnd_ref = window_hwnd_ref_vec.first();
    if let Some(window_hwnd_ref) = window_hwnd_ref {
        let window_hwnd_raw = *window_hwnd_ref;
        let success = show_window(window_hwnd_raw);
        if success {
            println!("Successfully activated {title}!");
        } else {
            eprintln!("Issue in activating {title}");
        }
        success
    } else {
        eprintln!("Couldn't find a window by the name {title}");
        false
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
        minimize_success && maximize_success
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
    println!("EXE termination subprocess started ({exe_name})");
    let output = Command::new("cmd")
        .args(["/C", "taskkill", "/f", "/IM", exe_name])
        .output()
        .expect("failed to execute Roblox termination");
    println!("EXE termination subprocess finished ({exe_name})");
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
        is_server_hop: bool,
    },
    Grief {},
    Leap {
        forward_amount: f64,
        spacebar_amount: f64,
        direction: char,
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
    bot_state: BotState,
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

                    success = check_active(window_title);
                    if !success {
                        notify_admin("Failed to find Roblox!").await.ok();
                        if let (Some(client), Some(chat_message)) = (&client_opt, &chat_message_opt)
                        {
                            client
                                .say_in_reply_to(
                                    chat_message,
                                    String::from("[Failed to find Roblox! Notified dev.]"),
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
                    run_console_command(&bot_config.game_name, command);
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
                    direction,
                } => {
                    println!("leap");
                    if client_origin {
                        instruction_history.push(history_entry);
                    }
                    leap(*forward_amount, *spacebar_amount, *direction);
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
                    send_system_chat(message);
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
                    send_user_chat(author, message);
                }
                Instruction::Wait { amount_ms } => {
                    println!("wait {amount_ms}");
                    let duration = tokio::time::Duration::from_millis(*amount_ms);
                    tokio::time::sleep(duration).await;
                }
                Instruction::WaitWithMessage { amount_ms, message } => {
                    println!("wait_with_message {amount_ms} {message}");

                    if let Err(_e) = hud_sender.send(HUDInstruction::TimedMessage {
                        message: message.clone(),
                        time: *amount_ms,
                    }) {
                        eprintln!("HUD Channel Error");
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
                Instruction::JoinServer {
                    server_id,
                    is_server_hop,
                } => {
                    // Deliberately synchronous/blocking
                    println!("join_game_selenium {}", &server_id);
                    join_game_selenium(bot_config.game_id, server_id);
                    let loaded_in = cv_check_loaded_in(&bot_config.game_name.clone());
                    if *is_server_hop {
                        {
                            // Write to app state and release lock
                            let mut state = bot_state.write().unwrap();
                            state.is_server_hopping = false;
                        }
                    }
                    println!("Loaded in: {loaded_in}");
                    if loaded_in {
                        hotfix_close_motd(&bot_config.game_name);
                        toggle_console_mouse(&bot_config.game_name);
                    } else {
                        notify_admin("Failed to load in!").await.ok();
                    }
                }
            }

            if !success {
                eprintln!("Failed instruction processing");
            }
        }
    }
}

#[must_use]
pub fn get_warp_locations() -> (HashMap<String, String>, String) {
    // TODO: Read from json or make this less awful
    let tp_locations_hashmap = HashMap::from_iter(
        [
            ("beach", "Beach"),
            ("beachhouse", "Beach House"),
            ("big", "Big Island"),
            ("bowling", "Bowling"),
            ("build", "i forgot how to build"),
            ("cave", "Cave"),
            ("devil", "Scarlet Devil Mansion"),
            ("fire", "Fireside Island"),
            ("fountain", "Fountain"),
            ("fumofas", "Fumofas Park"),
            ("highway", "Highway"),
            ("ice", "Ice Town"),
            ("minesweeper", "Minesweeper"),
            ("pof", "plates of fate v1.5"),
            ("poolside", "Poolside"),
            ("radio", "Radio Rock + Raging Demon Raceway"),
            ("ruins", "Ruins"),
            ("sewers", "Rat Sewers"),
            ("shrimp", "Shreimp Mart"),
            ("sky", "Floating Island"),
            ("somewhere", "somewhere"),
        ]
        .map(|(a, b)| (String::from(a), String::from(b))),
    );

    // Create ordered, comma-seperated string from key-value hashmap
    let mut valid_tp_locations_list = tp_locations_hashmap
        .keys()
        .map(|s| &**s)
        .collect::<Vec<_>>();

    // Sort alphabetically, ignoring case
    valid_tp_locations_list.sort_by_key(|key| key.to_lowercase());
    let valid_tp_locations_str = valid_tp_locations_list.join(", ");

    (tp_locations_hashmap, valid_tp_locations_str)
}

#[must_use]
pub fn get_hat_types() -> (HashMap<String, String>, String) {
    // TODO: Read from json or make this less awful
    let hat_types_hashmap = HashMap::from_iter(
        [
            ("none", "none"),
            ("bee", "BeeWriggle"),
            ("bird", "bird"),
            ("box", "MangosBox"),
            ("chito", "Chito1"),
            ("doremy", "DoremyHatOMGG"),
            ("fire1", "Diable Jambe V1"),
            ("fire2", "Diable Jambe V2"),
            ("glasses1", "Keine"),
            ("glasses2", "gagglasses"),
            ("guitar", "GuitarBag"),
            ("hanyuu", "hanyuuhat"),
            ("koishi", "KOISHIIIIIIII"),
            ("marisa1", "MarisaHat"),
            ("marisa2", "Marisav2Hat"),
            ("marisa3", "MarisasoewHat"),
            ("marisa4", "Marisapc98casualHat"),
            ("marisa5", "redmarisahat"),
            ("marisa6", "MarisaufoHat"),
            ("marisa6", "Marisapc98Hat"),
            ("meiling", "meilinghat"),
            ("miko", "MikoCape"),
            ("niko", "NikoHat"),
            ("pancake", "pancak"),
            ("scythe", "ellyscythe"),
            ("sunglasses1", "KaminaGlasses"),
            ("sunglasses2", "Hola"),
            ("strawberry", "Strawberry"),
            ("youmu", "youmumyonandswords"),
            ("yuuri", "Yuuri1"),
        ]
        .map(|(a, b)| (String::from(a), String::from(b))),
    );

    // Create ordered, comma-seperated string from key-value hashmap
    let mut valid_hats_list = hat_types_hashmap.keys().map(|s| &**s).collect::<Vec<_>>();
    // Sort alphabetically, ignoring case
    valid_hats_list.sort_by_key(|key| key.to_lowercase());
    let valid_hats_str = valid_hats_list.join(", ");

    (hat_types_hashmap, valid_hats_str)
}

#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CensorClientReturn {
    username: String,
    message: String,
    bot_reply_message: Vec<String>,
    send_users_message: bool,
}
pub async fn get_chat(
    username: String,
    message: String,
) -> Result<Option<CensorClientReturn>, Box<dyn Error>> {
    println!("test");
    let api_url: String = String::from("http://127.0.0.1:8086/request_censored_message");
    let request_body = json!({
        "username": username,
        "message": message
    });

    let client = reqwest::Client::new();
    println!("test2");
    let response = client.post(api_url).json(&request_body).send().await?;

    if !(&response.status().is_success()) {
        let error_message: String = format!(
            "[Censor Client Error - Response Status]\n```{}```",
            response.text().await?
        );
        eprint!("{}", &error_message);
        notify_admin(&error_message).await.ok();
        return Ok(Option::None);
    }
    let body = response.text().await?;

    match serde_json::from_str::<CensorClientReturn>(&body) {
        Ok(return_data) => Ok(Some(return_data)),
        Err(e) => {
            let error_message: String =
                format!("[Censor Client Error - JSON]\n{e:#?}\n```{body}```");
            eprint!("{}", &error_message);
            notify_admin(&error_message).await.ok();
            Ok(Option::None)
        }
    }
}

pub async fn twitch_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    hud_sender: UnboundedSender<HUDInstruction>,
    bot_config: BotConfig,
) {
    let twitch_config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_config.twitch_bot_username.clone(),
        Some(bot_config.twitch_bot_token.clone()),
    ));
    let (mut incoming_messages, raw_client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(twitch_config);

    let rc_client = Rc::new(raw_client);
    let client = &rc_client;

    //Begin Async Loop
    match client.join(bot_config.twitch_channel_name.clone()) {
        Ok(join) => join,
        Err(error) => panic!("Could not join the channel {error:?}"),
    }
    while let Some(message) = incoming_messages.recv().await {
        if let ServerMessage::Privmsg(msg) = message {
            println!("{}: {}", msg.sender.name, msg.message_text);
            if msg.message_text.starts_with('!') {
                let args = msg.message_text.replacen('!', "", 1);
                let clean_args: Vec<&str> = args.split_whitespace().collect();
                if clean_args.is_empty() {
                    continue;
                }
                let trigger = clean_args[0].to_lowercase();
                match trigger.as_ref() {
                    "ping" => {
                        if let Err(_e) = hud_sender.send(HUDInstruction::GenericMessage {
                            message: "ping".to_string(),
                        }) {
                            eprintln!("HUD Channel Error");
                        }
                        client
                            .say_in_reply_to(&msg, String::from("pong"))
                            .await
                            .unwrap();
                    }
                    "help" => {
                        client
                            .say_in_reply_to(
                                &msg,
                                format!(
                                    "For a full list of commands, visit {} . If you just want to play around, try '!m hello', '!move w 2', or '!warp poolside'",
                                    "https://sbf.fumocam.xyz/commands"
                                ),
                            )
                            .await.unwrap();
                    }
                    "m" => {
                        let mut msg_args = clean_args.clone();
                        msg_args.drain(0..1);
                        let message = msg_args.join(" ");
                        if message.is_empty() {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify a message! (i.e. !m hello)]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        let _discord_webook_result =
                            discord_log(&message, &msg.sender.name, true).await;

                        let author_name = msg.sender.name.to_string();

                        // TODO: config-held, env-driven array of mods
                        let mod_1 = env::var("TWITCH_MOD_1")
                            .expect("$TWITCH_MOD_1 is not set")
                            .to_lowercase();

                        let is_mod: bool = author_name.to_lowercase() == mod_1.to_lowercase();

                        if !is_mod
                            && ((message.starts_with('/')
                                && !(message.starts_with("/e"))
                                && !(message.starts_with("/animspeed")))
                                || message.starts_with('['))
                        {
                            // Disable command usage for non-mods
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from(
                                        "[You cannot run commands other than /e or /animspeed!]",
                                    ),
                                )
                                .await
                                .unwrap();
                            continue;
                        }

                        println!("{}: {message}", author_name.to_lowercase());

                        let success = check_active(&bot_config.game_name);
                        if !success {
                            notify_admin("Failed to find Roblox!").await.ok();
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Failed to find Roblox! Notified dev.]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        if message.starts_with('/') {
                            println!("Sending chat command from '{author_name}'\n{message}");
                            let chat_command_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.clone()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.clone(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::SystemChatMessage { message },
                                    },
                                ],
                            };
                            if let Err(_e) = queue_sender.send(chat_command_instructions) {
                                eprintln!("Chat Command Channel Error");
                            }
                        } else {
                            println!("Sending chat message\n{author_name}: {message}");
                            let chat_result_raw = get_chat(author_name, message).await;
                            let chat_result: CensorClientReturn;
                            match chat_result_raw {
                                Ok(valid_result) => {
                                    if let Some(result) = valid_result {
                                        chat_result = result;
                                        println!("gpt hjre");
                                    } else {
                                        client.say_in_reply_to(&msg, String::from("[Something went wrong, can't send a message. Contacting dev...]")).await.unwrap();
                                        continue;
                                    }
                                }
                                Err(error) => {
                                    let error_message: String =
                                        format!("[Censor Client Error - Main]\n```{error:#?}```");
                                    eprint!("{}", &error_message);
                                    notify_admin(&error_message).await.ok();

                                    client.say_in_reply_to(&msg, String::from("[Something went wrong, can't send a message. Contacting dev...]")).await.unwrap();
                                    continue;
                                }
                            }
                            let mut message_lock = false;
                            for bot_message in &chat_result.bot_reply_message {
                                if bot_message.contains("Spam detected") {
                                    message_lock = true;
                                }
                                println!("Replying: '{bot_message}'");
                                client
                                    .say_in_reply_to(&msg, bot_message.clone())
                                    .await
                                    .unwrap();
                            }
                            if message_lock {
                                if chat_result.message.is_empty() {
                                    // Already let users ingame know about lock, just twitch msg above and cancel
                                    continue;
                                }
                                // TODO: better code, this is needless rewriting
                                let chat_command_instructions = SystemInstruction {
                                    client: Some(client.clone()),
                                    chat_message: Some(msg.clone()),
                                    instructions: vec![
                                        InstructionPair {
                                            execution_order: 0,
                                            instruction: Instruction::CheckActive {
                                                window_title: bot_config.game_name.clone(),
                                            },
                                        },
                                        InstructionPair {
                                            execution_order: 1,
                                            instruction: Instruction::SystemChatMessage {
                                                message: chat_result.message.to_string(),
                                            },
                                        },
                                    ],
                                };
                                if let Err(_e) = queue_sender.send(chat_command_instructions) {
                                    eprintln!("User Chat Channel Error");
                                }
                            } else {
                                // Standard message, no messagelock
                                // TODO: cleaner code in tandem with above
                                if !chat_result.send_users_message {
                                    continue;
                                }
                                let censored_username: String =
                                    format!("{}:", chat_result.username);
                                let censored_message: String = chat_result.message.to_string();
                                println!("{censored_username}: {censored_message}");

                                let chat_command_instructions = SystemInstruction {
                                    client: Some(client.clone()),
                                    chat_message: Some(msg.clone()),
                                    instructions: vec![
                                        InstructionPair {
                                            execution_order: 0,
                                            instruction: Instruction::CheckActive {
                                                window_title: bot_config.game_name.clone(),
                                            },
                                        },
                                        InstructionPair {
                                            execution_order: 1,
                                            instruction: Instruction::UserChatMessage {
                                                message: censored_message,
                                                author: censored_username,
                                            },
                                        },
                                    ],
                                };
                                if let Err(_e) = queue_sender.send(chat_command_instructions) {
                                    eprintln!("User Chat Channel Error");
                                }
                            }
                        }
                    }
                    "a" => {
                        let mod_1 = env::var("TWITCH_MOD_1")
                            .expect("$TWITCH_MOD_1 is not set")
                            .to_lowercase();

                        let author_name = msg.sender.name.to_string().to_lowercase();

                        if author_name.to_lowercase() != mod_1.to_lowercase() {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[You do not have permissions to run this!]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }

                        let mut msg_args = clean_args.clone();
                        msg_args.drain(0..1);
                        let message = msg_args.join(" ");
                        if message.is_empty() {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify a message! (i.e. !a hello)]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }

                        let capitalized_message = capitalize_string(&message);
                        let formatted_message = format!("[{capitalized_message}]");
                        println!("Sending announce\n{formatted_message}");
                        let announce_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::SystemChatMessage {
                                        message: formatted_message,
                                    },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(announce_instructions) {
                            eprintln!("Announce Command Channel Error");
                        }
                    }
                    "dev" => {
                        let message = format!(
                            "{}: {}",
                            &msg.sender.name,
                            args.replacen("dev ", "", 1).replacen("dev", "", 1),
                        );
                        if message.is_empty() {
                            client
                                .say_in_reply_to(&msg, String::from("[Specify a message, this command is for emergencies! (Please do not misuse it)]"))
                                .await
                                .unwrap();
                            continue;
                        }
                        let result = notify_admin(&message).await;
                        if result.is_ok() {
                            client
                                .say_in_reply_to(&msg, String::from("[Notified dev! As a reminder, this command is only for emergencies. If you were unaware of this and used the command by mistake, please write a message explaining that or you may be timed-out/banned.]"))
                                .await
                                .unwrap();
                        } else {
                            client
                                .say_in_reply_to(&msg, String::from("[Error! Was unable to notify dev. Please join the Discord and ping CamDev.]"))
                                .await
                                .unwrap();
                        }
                    }
                    "move" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify a direction! (i.e. !move w 1)]"),
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
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Invalid direction! Specify a proper direction. (w, a, s, d)]"),
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
                                        client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[2])).await.unwrap();
                                        continue;
                                    }
                                    amount = val;
                                }
                                Err(_why) => {
                                    client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[2])).await.unwrap();
                                    continue;
                                }
                            }
                        }

                        let move_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::MoveDirection {
                                        direction,
                                        duration: amount,
                                    },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(move_instructions) {
                            eprintln!("MoveDirection Channel Error");
                        }
                    }
                    "warp" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(&msg, String::from("[Specify a location!]"))
                                .await
                                .unwrap();
                            client
                                .say_in_reply_to(
                                    &msg,
                                    format!("({})", bot_config.valid_tp_locations),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        let desired_location = bot_config.tp_locations.get(clean_args[1]);

                        if let Some(desired_location) = desired_location {
                            let warp_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.clone()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.clone(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::SystemChatMessage {
                                            message: format!("[Warping to {desired_location}!]")
                                                .to_string(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::Leap {
                                            // Clear any sitting effects
                                            forward_amount: 1.0,
                                            spacebar_amount: 1.0,
                                            direction: 'w',
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: format!("warp {desired_location}").to_string(),
                                        },
                                    },
                                ],
                            };
                            if let Err(_e) = queue_sender.send(warp_instructions) {
                                eprintln!("Warp Channel Error");
                            }
                        } else {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    format!("[{} is not a valid location!]", clean_args[1]),
                                )
                                .await
                                .unwrap();
                            client
                                .say_in_reply_to(
                                    &msg,
                                    format!("({})", bot_config.valid_tp_locations),
                                )
                                .await
                                .unwrap();
                        }
                    }
                    "left" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify degrees to rotate! (i.e. !left 90)]"),
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
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 90.]", clean_args[1])).await.unwrap();
                                continue;
                            }
                        }
                        let camera_left_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::MoveCameraX { x_ratio: amount },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(camera_left_instructions) {
                            eprintln!("Camera Left Channel Error");
                        }
                    }
                    "right" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify degrees to rotate! (i.e. !right 90)]"),
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
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 90.]", clean_args[1]), ).await.unwrap();
                                continue;
                            }
                        }
                        let camera_right_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(camera_right_instructions) {
                            eprintln!("Camera Right Channel Error");
                        }
                    }
                    "up" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify degrees to rotate! (i.e. !up 45)]"),
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
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1])).await.unwrap();
                                continue;
                            }
                        }
                        let camera_up_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::MoveCameraY { y_ratio: amount },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(camera_up_instructions) {
                            eprintln!("Camera Up Channel Error");
                        }
                    }
                    "down" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify degrees to rotate! (i.e. !down 45)]"),
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
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1]) ).await.unwrap();
                                continue;
                            }
                        }
                        let camera_down_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(camera_down_instructions) {
                            eprintln!("Camera Down Channel Error");
                        }
                    }
                    "zoomin" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify percent to zoom in! (i.e. !zoomin 50)]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        const MAX_AMOUNT: f64 = 1_000_000.0;
                        #[allow(unused_assignments)]
                        let mut amount = 45.0;
                        let num = clean_args[1].parse::<f64>();
                        match num {
                            Ok(val) => {
                                if val > MAX_AMOUNT || val <= 0.0 {
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 45.]", clean_args[1])).await.unwrap();
                                continue;
                            }
                        }
                        let camera_zoom_in_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(camera_zoom_in_instructions) {
                            eprintln!("Camera Zoom In Channel Error");
                        }
                    }
                    "zoomout" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify percent to zoom in! (i.e. !zoomin 50)]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        const MAX_AMOUNT: f64 = 1_000_000.0;
                        #[allow(unused_assignments)]
                        let mut amount = 45.0;
                        let num = clean_args[1].parse::<f64>();
                        match num {
                            Ok(val) => {
                                if val > MAX_AMOUNT || val <= 0.0 {
                                    client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                    continue;
                                }
                                amount = val;
                            }
                            Err(_why) => {
                                client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number, i.e. 50.]", clean_args[1])).await.unwrap();
                                continue;
                            }
                        }
                        let camera_zoom_out_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(camera_zoom_out_instructions) {
                            eprintln!("Camera Zoom Out Channel Error");
                        }
                    }
                    "leap" => {
                        const MAX_AMOUNT: f64 = 2.0;
                        let mut forward_amount: f64 = 1.0;
                        let mut spacebar_amount: f64 = 1.0;
                        let mut direction: char = 'w';
                        let mut direction_first_arg = false;
                        if clean_args.len() > 1 {
                            let arg_1 = clean_args[1].parse::<f64>();
                            match arg_1 {
                                Ok(val) => {
                                    if val > MAX_AMOUNT || val <= 0.0 {
                                        client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[1])).await.unwrap();
                                        continue;
                                    }
                                    forward_amount = val;
                                }
                                Err(_why_float) => {
                                    let direction_arg =
                                        clean_args[1].to_lowercase().parse::<char>();
                                    match direction_arg {
                                        Ok(direction_val) => {
                                            if VALID_DIRECTIONS.contains(&direction_val) {
                                                direction = direction_val;
                                                direction_first_arg = true;
                                            } else {
                                                client.say_in_reply_to(&msg, format!("[{} is not a valid direction! Please specify a valid direction or leave it blank.]", clean_args[1])).await.unwrap();
                                                continue;
                                            }
                                        }
                                        Err(_why_char) => {
                                            client.say_in_reply_to(&msg, format!("[{} is not a valid number/direction! Please specify a number or leave it blank.]", clean_args[1])).await.unwrap();
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        if clean_args.len() > 2 {
                            let arg_2 = clean_args[2].parse::<f64>();
                            match arg_2 {
                                Ok(val) => {
                                    if val > MAX_AMOUNT || val <= 0.0 {
                                        client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[2]),).await.unwrap();
                                        continue;
                                    }
                                    if direction_first_arg {
                                        forward_amount = val;
                                    } else {
                                        spacebar_amount = val;
                                    }
                                }
                                Err(_why) => {
                                    client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[2])).await.unwrap();
                                    continue;
                                }
                            }
                        }
                        if clean_args.len() > 3 && direction_first_arg {
                            let arg_3 = clean_args[3].parse::<f64>();
                            match arg_3 {
                                Ok(val) => {
                                    if val > MAX_AMOUNT || val <= 0.0 {
                                        client.say_in_reply_to(&msg, format!("[{} is too high/low! Please specify a number between 0 or {MAX_AMOUNT}.]", clean_args[3])).await.unwrap();
                                        continue;
                                    }
                                    spacebar_amount = val;
                                }
                                Err(_why) => {
                                    client.say_in_reply_to(&msg, format!("[{} is not a valid number! Please specify a number or leave it blank.]", clean_args[3])).await.unwrap();
                                    continue;
                                }
                            }
                        }
                        let leap_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::Leap {
                                        forward_amount,
                                        spacebar_amount,
                                        direction,
                                    },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(leap_instructions) {
                            eprintln!("Leap Channel Error");
                        }
                    }
                    "hidemouse" => {
                        let hide_mouse_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![InstructionPair {
                                execution_order: 0,
                                instruction: Instruction::HideMouse {},
                            }],
                        };
                        if let Err(_e) = queue_sender.send(hide_mouse_instructions) {
                            eprintln!("Hide Mouse Channel Error");
                        }
                    }
                    "jump" => {
                        let jump_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::Leap {
                                        forward_amount: 0.0,
                                        spacebar_amount: 1.0,
                                        direction: 'w',
                                    },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(jump_instructions) {
                            eprintln!("Jump Channel Error");
                        }
                    }
                    "grief" => {
                        let jump_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::Grief {},
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(jump_instructions) {
                            eprintln!("Grief Channel Error");
                        }
                    }
                    "refresh" => {
                        let refresh_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(refresh_instructions) {
                            eprintln!("Refresh Channel Error");
                        }
                    }
                    "die" => {
                        let die_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(die_instructions) {
                            eprintln!("Die Channel Error");
                        }
                    }
                    "explode" => {
                        let explode_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(explode_instructions) {
                            eprintln!("Explode Channel Error");
                        }
                    }
                    "sit" => {
                        let sit_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::Sit {},
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(sit_instructions) {
                            eprintln!("Sit Channel Error");
                        }
                    }
                    "size" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify a size! The default is 'base'. (doll, shimmy, base, deka)]"),
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
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Invalid size! Specify a size, the default is 'base'. (doll, shimmy, base, deka)]"),
                                )
                                .await
                                .unwrap();
                            continue;
                        }
                        let size_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
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
                        if let Err(_e) = queue_sender.send(size_instructions) {
                            eprintln!("Size Channel Error");
                        }
                    }
                    "hat" => {
                        if clean_args.len() < 2 {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    String::from("[Specify a hat, or use !removehat]."),
                                )
                                .await
                                .unwrap();
                            client
                                .say_in_reply_to(&msg, format!("({})", bot_config.valid_hat_types))
                                .await
                                .unwrap();
                            continue;
                        }
                        let requested_hat_type = clean_args[1].to_lowercase();
                        let desired_hat = bot_config.hat_types.get(&requested_hat_type);

                        if let Some(desired_hat) = desired_hat {
                            let hat_instructions = SystemInstruction {
                                client: Some(client.clone()),
                                chat_message: Some(msg.clone()),
                                instructions: vec![
                                    InstructionPair {
                                        execution_order: 0,
                                        instruction: Instruction::CheckActive {
                                            window_title: bot_config.game_name.clone(),
                                        },
                                    },
                                    InstructionPair {
                                        execution_order: 1,
                                        instruction: Instruction::ConsoleCommand {
                                            command: format!("changehat me {desired_hat}")
                                                .to_string(),
                                        },
                                    },
                                ],
                            };
                            if let Err(_e) = queue_sender.send(hat_instructions) {
                                eprintln!("Hat Channel Error");
                            }
                        } else {
                            client
                                .say_in_reply_to(
                                    &msg,
                                    format!("[{} is not a valid hat!]", &requested_hat_type),
                                )
                                .await
                                .unwrap();
                            client
                                .say_in_reply_to(&msg, format!("({})", bot_config.valid_hat_types))
                                .await
                                .unwrap();
                        }
                    }
                    "removehat" => {
                        let remove_hat_instructions = SystemInstruction {
                            client: Some(client.clone()),
                            chat_message: Some(msg.clone()),
                            instructions: vec![
                                InstructionPair {
                                    execution_order: 0,
                                    instruction: Instruction::CheckActive {
                                        window_title: bot_config.game_name.clone(),
                                    },
                                },
                                InstructionPair {
                                    execution_order: 1,
                                    instruction: Instruction::ConsoleCommand {
                                        command: String::from("changehat me none"),
                                    },
                                },
                            ],
                        };
                        if let Err(_e) = queue_sender.send(remove_hat_instructions) {
                            eprintln!("Removehat Channel Error");
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
                            .say_in_reply_to(&msg, message.to_string())
                            .await
                            .unwrap();
                    }
                    _ => {
                        client
                            .say_in_reply_to(
                                &msg,
                                String::from(
                                    "[Not a valid command! Type !help for a list of commands.]",
                                ),
                            )
                            .await
                            .unwrap();
                    }
                }
            }

            let _result = discord_log(&msg.message_text, &msg.sender.name, false).await;
        }
    }
}

#[derive(Clone)]
pub struct BotConfig {
    game_name: String,
    game_executable: String,
    game_id: i64,
    hat_types: HashMap<String, String>,
    tp_locations: HashMap<String, String>,
    twitch_bot_username: String,
    twitch_bot_token: String,
    player_token: String,
    twitch_channel_name: String,
    valid_hat_types: String,
    valid_tp_locations: String,
}

#[must_use]
pub fn init_config() -> BotConfig {
    let game_name: String = "Roblox".to_string();
    let game_executable: String = "RobloxPlayerBeta.exe".to_string();
    let game_id: i64 = 7_363_647_365;
    let twitch_channel_name: String = "sbfcam".to_string();
    let twitch_bot_username: String = "sbfcamBOT".to_string();

    let (tp_locations, valid_tp_locations) = get_warp_locations();
    let (hat_types, valid_hat_types) = get_hat_types();

    //Oauth (generated with https://twitchtokengenerator.com/):
    let twitch_bot_token = env::var("TWITCH_BOT_TOKEN").expect("$TWITCH_BOT_TOKEN is not set");
    let player_token = env::var("PLAYER_TOKEN").expect("$PLAYER_TOKEN is not set");

    BotConfig {
        game_name,
        game_executable,
        game_id,
        hat_types,
        tp_locations,
        twitch_bot_username,
        twitch_bot_token,
        player_token,
        twitch_channel_name,
        valid_hat_types,
        valid_tp_locations,
    }
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
                            window_title: bot_config.game_name.clone(),
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
            if let Err(_e) = queue_sender.send(anti_afk_instructions) {
                eprintln!("Anti-AFK Channel Error");
            }
        }

        let advert_instructions = SystemInstruction {
            client: None,
            chat_message: None,
            instructions: vec![
                InstructionPair {
                    execution_order: 0,
                    instruction: Instruction::CheckActive {
                        window_title: bot_config.game_name.clone(),
                    },
                },
                InstructionPair {
                    execution_order: 1,
                    instruction: Instruction::SystemChatMessage {
                        message: "You can control this bot live!".to_string(),
                    },
                },
                InstructionPair {
                    execution_order: 2,
                    instruction: Instruction::SystemChatMessage {
                        message: "Go to its Roblox profile and click the purple T witch icon!"
                            .to_string(),
                    },
                },
            ],
        };
        if let Err(_e) = queue_sender.send(advert_instructions) {
            eprintln!("Anti-AFK Channel Error");
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
    let api_url: String = format!("https://games.roblox.com/v1/games/{game_id}/servers/Public");
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
    println!("Body:\n{debug_body}");

    if !(body_json.is_object()) {
        eprint!("Error, body not an object\n{body}");
        return Ok(Option::None);
    }
    let body_obj = body_json.as_object().unwrap();
    if !(body_obj.contains_key("data")) {
        eprint!("Error, missing data\n{body}");
        return Ok(Option::None);
    }
    let response_data = body_obj.get("data").unwrap();
    if !(response_data.is_array()) {
        eprint!("Error, data not array\n{body}");
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
            players,
        };
        instance_list.push(instance);
    }

    Ok(Some(instance_list))
}

fn get_current_server(player_token: &String, instance_list: Vec<GameInstance>) -> GameInstance {
    for instance in instance_list {
        println!("{}", instance.id);
        if instance.players.contains(player_token) {
            return instance;
        }
    }
    GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    }
}

fn check_in_server(player_token: &String, instance_list: Vec<GameInstance>) -> bool {
    let current_server = get_current_server(player_token, instance_list);
    current_server.id != *UNKNOWN_SERVER_ID
}

fn get_best_server(instance_list: Vec<GameInstance>) -> GameInstance {
    let mut best_server = GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    };
    for instance in instance_list {
        if best_server.players.len() < instance.players.len() {
            best_server = instance;
        }
    }
    best_server
}

fn check_in_best_server(player_token: &String, instance_list: Vec<GameInstance>) -> bool {
    const LOW_SWITCH: usize = 2;
    const LOW_SWITCH_THRESH: usize = 7;
    const HIGH_SWITCH: usize = 5;
    let best_server = get_best_server(instance_list.clone());
    if best_server.id == *UNKNOWN_SERVER_ID {
        // Somehow, our instance list is empty. Stay in current server for safety.
        return true;
    }

    let mut current_server = GameInstance {
        id: UNKNOWN_SERVER_ID.to_string(),
        players: vec![],
    };
    for instance in instance_list {
        println!("{}", instance.id);
        if instance.players.contains(player_token) {
            current_server = instance;
            break;
        }
    }
    if best_server.id == *UNKNOWN_SERVER_ID {
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
    println!("[Server difference {difference}]");
    println!("[Required switch difference {required_difference}]");
    difference <= required_difference
}

fn _force_rejoin(
    queue_sender: &UnboundedSender<SystemInstruction>,
    bot_config: &BotConfig,
    instance_list: Vec<GameInstance>,
) {
    let best_server = get_best_server(instance_list.clone());
    let current_server = get_current_server(&bot_config.player_token, instance_list);

    let mut instructions: Vec<InstructionPair> = vec![];
    if current_server.id != UNKNOWN_SERVER_ID && check_active(&bot_config.game_name) {
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
            server_id: best_server.id,
            is_server_hop: false,
        },
    });
    instructions.push(InstructionPair {
        execution_order: 5,
        instruction: Instruction::CheckActive {
            window_title: bot_config.game_name.clone(),
        },
    });
    instructions.push(InstructionPair {
        execution_order: 6,
        instruction: Instruction::SystemChatMessage {
            message: "[Force-Rejoin complete! Ready to accept commands.]".to_string(),
        },
    });

    let join_instruction = SystemInstruction {
        client: None,
        chat_message: None,
        instructions,
    };
    if let Err(_e) = queue_sender.send(join_instruction) {
        eprintln!("JoinServer Channel Error");
    }
}

async fn force_rejoin(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
) -> bool {
    let get_instances_result = get_instances(bot_config.game_id).await;
    match get_instances_result {
        Ok(instance_list_option) => {
            if let Some(instance_list) = instance_list_option {
                let best_server = get_best_server(instance_list.clone());
                if best_server.id == *UNKNOWN_SERVER_ID {
                    eprintln!("[Roblox API check failed]");
                    false
                } else {
                    _force_rejoin(&queue_sender, &bot_config, instance_list);
                    true
                }
            } else {
                eprintln!("[Roblox API check failed]");
                false
            }
        }
        Err(_error) => {
            eprintln!("[Roblox API check failed]");
            false
        }
    }
}

async fn server_check_logic(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
    bot_state: BotState,
    instance_list: Vec<GameInstance>,
) {
    let is_server_hopping = {
        // Get BotState value and release lock immediately
        let state = bot_state.read().unwrap();
        state.is_server_hopping
    };

    if is_server_hopping {
        return;
    }

    let is_initial_boot = {
        // Get BotState value and release lock immediately
        let state = bot_state.read().unwrap();
        state.initial_boot
    };

    let in_server = check_in_server(&bot_config.player_token, instance_list.clone());
    let in_best_server = if in_server {
        check_in_best_server(&bot_config.player_token, instance_list.clone())
    } else {
        false
    };

    if in_server && in_best_server {
        if is_initial_boot {
            {
                // Write to app state and release lock
                let mut state = bot_state.write().unwrap();
                state.initial_boot = false;
            }
        }
        return;
    }

    let best_server = get_best_server(instance_list.clone());
    let current_server = get_current_server(&bot_config.player_token, instance_list.clone());
    let difference = best_server.players.len() - current_server.players.len();

    let instructions: Vec<InstructionPair>;

    // Server hop (more people)
    if in_server {
        if is_initial_boot {
            {
                // Write to app state and release lock
                let mut state = bot_state.write().unwrap();
                state.initial_boot = false;
            }
        }
        {
            // Write to app state and release lock
            let mut state = bot_state.write().unwrap();
            state.is_server_hopping = true;
        }
        instructions = vec![
            InstructionPair {
                execution_order: 0,
                instruction: Instruction::CheckActive {
                    window_title: bot_config.game_name.clone(),
                },
            },
            InstructionPair {
                execution_order: 1,
                instruction: Instruction::SystemChatMessage {
                    message: format!("[Moving servers! There is a server with {difference} more players. See you there!]"),
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
                    server_id: best_server.id.clone(),
                    is_server_hop: true
                },
            },
            InstructionPair {
                execution_order: 6,
                instruction: Instruction::CheckActive {
                    window_title: bot_config.game_name.clone(),
                },
            },
            InstructionPair {
                execution_order: 7,
                instruction: Instruction::SystemChatMessage {
                    message: "[Server relocation complete! Ready to accept commands.]".to_string(),
                },
            },
        ];
    }
    // Restart/Crash
    else {
        let rejoin_mesage: &str;

        if is_initial_boot {
            //Restart
            println!("Restart detected");
            rejoin_mesage = "[Restart complete! Ready to accept commands.]";
            {
                // Write to app state and release lock
                let mut state = bot_state.write().unwrap();
                state.initial_boot = false;
            }
            log_admin("Restart detected").await.ok();
        } else {
            // Crash
            println!("Potential crash detected");
            rejoin_mesage = "[Crash recovery complete! Ready to accept commands.]";
            let is_active = check_active(&bot_config.game_name.clone());
            let exe_status = if is_active { "Running" } else { "Not found" };
            let message = format!("Likely crash detected | {exe_status}");
            notify_admin(&message).await.ok();
        }

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
                    server_id: best_server.id.clone(),
                    is_server_hop: false,
                },
            },
            InstructionPair {
                execution_order: 3,
                instruction: Instruction::CheckActive {
                    window_title: bot_config.game_name.clone(),
                },
            },
            InstructionPair {
                execution_order: 4,
                instruction: Instruction::SystemChatMessage {
                    message: rejoin_mesage.to_string(),
                },
            },
        ];
    }

    let join_instruction = SystemInstruction {
        client: None,
        chat_message: None,
        instructions,
    };
    if let Err(_e) = queue_sender.send(join_instruction) {
        eprintln!("JoinServer Channel Error");
    }
}

async fn server_check_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    bot_config: BotConfig,
    bot_state: BotState,
) {
    let interval_minutes = 3;
    let mut interval = tokio::time::interval(Duration::from_millis(interval_minutes * 60 * 1000));
    loop {
        interval.tick().await;
        let get_instances_result = get_instances(bot_config.game_id).await;
        match get_instances_result {
            Ok(instance_list_option) => match instance_list_option {
                Some(instance_list) => {
                    let best_server = get_best_server(instance_list.clone());
                    if best_server.id == *UNKNOWN_SERVER_ID {
                        eprint!("[Roblox API check failed]");
                    } else {
                        server_check_logic(
                            queue_sender.clone(),
                            bot_config.clone(),
                            bot_state.clone(),
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

fn join_game_selenium(game_id: i64, instance_id: &str) {
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
            instance_id,
            "--driver",
            &driver,
        ])
        .current_dir("../python")
        .output()
        .expect("failed to execute process");
    println!("Selenium subprocess finished");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
}

#[must_use]
pub fn restart_warn(restart_warn_ticker: i32) -> i32 {
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
        const TWO_MINUTES: u64 = 120_000; // 1000 * 60 * 2, Could be one minute, but safety padding is nice
        if winapi::um::sysinfoapi::GetTickCount64() < TWO_MINUTES {
            return -1;
        }
    }

    if minute == FIVE_MINUTE_WARNING {
        FIVE_MINUTE_WARNING
    } else if minute == ONE_MINUTE_WARNING {
        ONE_MINUTE_WARNING
    } else if minute == REBOOT_TIME {
        REBOOT_TIME
    } else {
        -1
    }
}

pub fn restart_logic(
    queue_sender: &UnboundedSender<SystemInstruction>,
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
                window_title: bot_config.game_name,
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
        instructions,
    };
    if let Err(_e) = queue_sender.send(restart_system_instruction) {
        eprintln!("Restart chat command error");
    }
}

pub async fn clock_tick_loop(
    queue_sender: UnboundedSender<SystemInstruction>,
    _hud_sender: UnboundedSender<HUDInstruction>,
    bot_config: BotConfig,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    let mut restart_warn_ticker = -1; // TODO: This needs a way cleaner system
    loop {
        interval.tick().await;

        // Restart logic
        let restart_return = restart_warn(restart_warn_ticker);
        if restart_warn_ticker != restart_return && restart_return != -1 {
            restart_warn_ticker = restart_return;
            restart_logic(&queue_sender, bot_config.clone(), restart_return);
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
            println!("Pixel Response: {return_val}");
            return_val
        }
        Err(e) => {
            eprintln!("Error! {e:#?}");
            false
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
    println!("cv_get_backpack_hover");
    get_pixel(575, 680, 10, 5, 179, 179, 179)
}
fn cv_get_navbar(window_title: &str) -> bool {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.47, 0.99);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.47, 0.99);
    println!("cv_get_navbar");
    get_pixel(575, 680, 10, 5, 255, 255, 255)
}
fn cv_get_navbar_hidden(window_title: &str) -> bool {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_hide(&mut enigo);
    thread::sleep(DELAY);
    mouse_hide(&mut enigo);
    println!("cv_get_navbar_hidden");
    !(get_pixel(575, 680, 10, 5, 255, 255, 255))
}

fn hotfix_close_motd(window_title: &str) {
    check_active(window_title);
    const DELAY: Duration = Duration::from_millis(500);
    let mut enigo = Enigo::new();
    mouse_move(&mut enigo, 0.2, 0.78);
    thread::sleep(DELAY);
    mouse_move(&mut enigo, 0.2, 0.78);
    mouse_click(&mut enigo);
    thread::sleep(DELAY * 2);
    mouse_click(&mut enigo);
    println!("hotfix_close_motd");
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

    true
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
                println!("[HUD] ClientMessage: {message}");
            }
            HUDInstruction::ClientBinaryMessage { binary_message } => {
                println!("[HUD] ClientBinaryMessage: {binary_message:?}");
            }
            HUDInstruction::GenericMessage { message } => {
                println!(
                    "[HUD] Sending message to all connected clients ({}): {}",
                    clients.len(),
                    message
                );
                for client_responder in clients.values() {
                    client_responder.send(simple_websockets::Message::Text(message.clone()));
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
                for client_responder in clients.values() {
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
                println!("[HUD] A client connected with id #{client_id}");
                if let Err(_e) = hud_sender.send(HUDInstruction::AddClient {
                    client_id,
                    responder,
                }) {
                    eprintln!("[HUD] Client Connect Channel Error");
                }
            }
            simple_websockets::Event::Disconnect(client_id) => {
                println!("[HUD] A client connected with id #{client_id}");
                if let Err(_e) = hud_sender.send(HUDInstruction::RemoveClient { client_id }) {
                    eprintln!("[HUD] Client Disconnect Channel Error");
                }
            }
            simple_websockets::Event::Message(client_id, message) => {
                println!("[HUD] Received a message from client #{client_id}: {message:?}");
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
                if let Err(_e) = hud_sender.send(hud_message_instruction) {
                    eprintln!("[HUD] Client Message Channel Error");
                }
            }
        }
    }
}
// END HUD STUFF
pub struct BotStateModel {
    initial_boot: bool,
    is_server_hopping: bool,
}
type BotState = Arc<RwLock<BotStateModel>>;
#[must_use]
pub fn init_state() -> BotState {
    Arc::new(RwLock::new(BotStateModel {
        initial_boot: true,
        is_server_hopping: false,
    }))
}

// =================
// [Production Main]
// =================

#[tokio::main]
pub async fn main() {
    dotenv::from_filename("..\\.env").ok();

    let bot_config = init_config();
    let bot_state = init_state();
    //check_active(&bot_config.game_name);
    // click_console_input();
    //run_console_command(&bot_config.game_name, "test");

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
    let queue_processor_task = queue_processor(
        queue_receiver,
        hud_sender.clone(),
        bot_config.clone(),
        bot_state.clone(),
    );
    let server_check_task =
        server_check_loop(queue_sender.clone(), bot_config.clone(), bot_state.clone());
    let clock_tick_task =
        clock_tick_loop(queue_sender.clone(), hud_sender.clone(), bot_config.clone());
    let hud_loop_task = hud_loop(hud_receiver);
    let hud_ws_server_task = hud_ws_server(hud_sender.clone());

    check_active(&bot_config.game_name.clone());

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

// //===========
// //[Test Main]
// //===========
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
//     check_active(&bot_config.game_name);
//     //leap(1.0, 1.0, 'w');
// }
