
use std::env;
use std::error::Error;
use md5::compute;

use dotenvy::dotenv;
use ratatui::crossterm::event::{self, Event};
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};
use reqwest::redirect::Policy;
use serde::Deserialize;

#[derive(Deserialize)]
struct Device {
    devType: String,
    deviceId: String,
    devName: String
}


fn main() -> Result<(), Box<dyn(Error)>>{
    dotenv()?;
    let password: String = env::var("PASSWORD").expect(".env file not containing password");
    println!("pass = {}", password);
    let hashed_password = format!("{:x}", compute(password));
    println!("hashed pass = {}", hashed_password);

    let client = reqwest::blocking::Client::builder().redirect(Policy::none()).build()?;
    let params = format!("username=admin&password={}",hashed_password);
    let response = client.post("http://192.168.0.1/login/Auth")
                                   .header("Content-Type", "application/x-www-form-urlencoded")
                                   .body(params)
                                   .send()?;
    let cookies = response.cookies();
    println!("Status code = {}", response.status());
    let mut cookie_password: String = String::new();
    for cookie in cookies{
        println!("{}={}", cookie.name(),cookie.value());
        if cookie.name() == "password" {
            cookie_password = cookie.value().to_string();
            unsafe {
                env::set_var("COOKIE_PASSWORD", cookie_password.clone());
            }
        }
    }
    setup();
    Ok(())
}

fn setup(){
    color_eyre::install();
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
}

fn run(mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let devices: Vec<Device> = get_devices_connected().unwrap();
    // for device in devices{
    //     frame.render_widget(format!("{}", device.devName), frame.area());
    // }
    let texts: Vec<Line> = devices.iter().enumerate().map(|(index, device)| Line::from(format!("{} - {} / {} ", index + 1 ,  device.devName, device.deviceId))).collect();
    let paragraph = Paragraph::new(texts);
    frame.render_widget(paragraph, frame.area());
}

fn get_devices_connected() -> Result<Vec<Device>, Box<dyn Error>> {
    let cookie_password: String = env::var("COOKIE_PASSWORD")?;
    let client = reqwest::blocking::Client::builder().redirect(Policy::none()).build()?;
    let response = client.get("http://192.168.0.1/goform/GetParentCtrlList?0.022112934996180056")
                        .header("Cookie", format!("password={}", &cookie_password))
                        .send()?;
    let devices: Vec<Device> = response.json::<Vec<Device>>()?;
    Ok(devices)
}

