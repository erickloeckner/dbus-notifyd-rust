use std::env::{self, args};
use std::fs;
use std::path;
use std::process::{self, Command, Stdio};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;

extern crate dbus;

use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::{MatchRule,MessageType};
//~ use dbus::strings::Member;

extern crate gtk;
extern crate glib;
use gio::prelude::*;
use gtk::prelude::*;

const PROJECT_NAME: &str = "dbus-notifyd-rust";
const VERSION: &str = "v. 1.0.2";

use serde_derive::Deserialize;
#[derive(Deserialize)]
struct Config {
    //~ main_config: MainConfig,
    match_string: String,
    window_title: String,
}

//~ #[derive(Deserialize)]
//~ struct MainConfig {
    //~ match_string: String,
//~ }

fn main() -> () {
    let mut css_path = find_cwd(PROJECT_NAME);
    css_path.push("theme.css");
    let style = fs::read_to_string(css_path.to_str().unwrap()).unwrap();
    
    let application =
        gtk::Application::new(Some("com.github.dbus-notifyd-rust"), Default::default())
            .expect("Initialization failed...");
    
    application.connect_activate(move |app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(style.as_str().as_bytes())
            .expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn enumerate_audio() -> Vec<String> {
    let mut out = Vec::new();
    let cmd = Command::new("pactl")
        .arg("list")
        .output();
    
    match cmd {
        Ok(v) => {
            //~ println!("{:?}", v.stdout.as_slice());
            let cmd_stdout = String::from_utf8(v.stdout).unwrap();
            for line in cmd_stdout.lines() {
                //~ println!("{}: len: {}", i, line.len());
                if line.len() > 4 {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    //~ println!("split line...");
                    if fields[0] == "Name:" {
                        let parts: Vec<_> = fields[1].split('.').collect();
                        if parts[0] == "alsa_output" && parts.last().unwrap() != &"monitor" {
                            //~ println!("{}", fields[1]);
                            out.push(String::from(fields[1]));
                        }
                    }
                }
            }
            //~ println!("{:?}", cmd_stdout);
        },
        Err(_) => println!("command not found"),
    }
    out
}

fn find_cwd(project_name: &str) -> path::PathBuf {
    let mut path_out = env::current_exe().unwrap();
    let mut res = path_out.pop();
    loop {
        if res {
            if path_out.as_path().file_name().unwrap().to_str().unwrap() == project_name {
                break;
            }
            res = path_out.pop();
        } else {
            break;
        }
    }
    path_out
}

fn build_ui(application: &gtk::Application) {        
    let mut cwd = find_cwd(PROJECT_NAME);
    cwd.push("config.toml");
    let config_raw = fs::read_to_string(cwd.to_str().unwrap()).unwrap();
    let config: Config = toml::from_str(&config_raw).unwrap_or_else(|err| {
        println!("error parsing config: {}", err);
        process::exit(1);
    });
    //~ let match_label = String::from(config.match_string.as_str());
    //~ let win_label = String::from(config.window_title.as_str());
    let match_string = String::from(config.match_string.as_str());
    cwd.pop();
    
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let tx_clone = tx.clone();
    
    let _dbus_thread = thread::spawn(move || {
        let mut conn = Connection::new_session().unwrap();
        
        let mut mr = MatchRule::new_signal("org.freedesktop.Notifications", "Notify");
        mr.msg_type = Some(MessageType::MethodCall);
        
        let proxy = conn.with_proxy("org.freedesktop.DBus", "/org/freedesktop/DBus", Duration::from_millis(5000));
        let _: () = proxy.method_call("org.freedesktop.DBus", "AddMatch", (mr.match_str(),)).unwrap_or_else( |_| {
            println!("failed on method call 'AddMatch'");
            //~ process::exit(1);
        });
        
        let filter = vec!("type='method_call',interface='org.freedesktop.Notifications',member='Notify'",);
        let _: () = proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (filter, 0u32)).unwrap_or_else( |_| {
            println!("failed on method call 'BecomeMonitor'");
            //~ process::exit(1);
        });
        
        //~ let member_name = Member::new("Notify").unwrap();
        conn.start_receive(mr, Box::new(move |msg, _| {
            //~ let member_name = Member::new("Notify");
            let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
            
            //~ if title == "Inbound Call" || body == "Inbound Call" {
            if title == match_string || body == match_string {
                tx.send(String::from(body)).unwrap();
                
                //~ println!("message: {} | {} : {}", sender, title, body);
                //~ let cwd = find_cwd(PROJECT_NAME);
                //~ let mut script = find_cwd(PROJECT_NAME);
                //~ script.push("dbus-notifyd_command.sh");
                //~ let _output = Command::new(script.to_str().unwrap())
                    //~ .current_dir(cwd.to_str().unwrap())
                    //~ .stdout(Stdio::null())
                    //~ .output()
                    //~ .expect("Failed to execute command");
                
                //~ msg_tx_c.send(true);
            }
            
            //~ if msg.member().unwrap() == member_name {
                //~ let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
                //~ println!("message: {} | {} : {}", sender, title, body);
            //~ }
            true
        }));
        
        loop {
            conn.process(Duration::from_millis(1000)).unwrap();
            //~ println!("loop");
        }
    });
    
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(&config.window_title);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 32);
    window.set_resizable(false);
    window.set_keep_above(true);
    
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    //~ main_box.set_size_request(120, 100);
    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    
    let label = gtk::Label::new(Some(&format!("{}  Listening for '{}'", VERSION, config.match_string)));
    label.set_widget_name("grad_label");
    
    let menu_button = gtk::MenuButton::new();
    let menu = gtk::Menu::new();
    //~ let menu_1 = gtk::MenuItem::new_with_label("foo");
    //~ let menu_2 = gtk::MenuItem::new_with_label("bar");
    //~ menu.append(&menu_1);
    //~ menu.append(&menu_2);
    
    let audio_devices = enumerate_audio();
    let audio_selection = Rc::new(RefCell::new(String::from(audio_devices[0].as_str())));
    
    let mut menu_vec = Vec::new();
    
    for (ind, dev) in audio_devices.iter().enumerate() {
        let menu_i = gtk::MenuItem::new_with_label(dev);
        if ind == 0 {
            //~ label.clear();
            //~ label.push_str(&format!("* {}", dev));
            menu_i.set_widget_name("selected");
        } else {
            menu_i.set_widget_name("deselected");
        }
        
        //~ match ind {
            //~ 0 => { let menu_i = gtk::MenuItem::new_with_label(&format!("* {}", dev)); },
            //~ _ => { let menu_i = gtk::MenuItem::new_with_label(dev); },
        //~ }
        
        //~ let menu_i_c = menu_i.clone();
        //~ let out = String::from(dev);
        
        menu_vec.push(menu_i);
        
        /*
        let audio_selection_c = audio_selection.clone();
        menu_i.connect_activate(move |_| {
            //~ println!("clicked {}", out);
            //~ let new_label = &format!("* {}", menu_i_c.get_label().unwrap().as_str());
            //~ menu_i_c.set_label(new_label);
            //~ menu_i_c.set_sensitive(false);
            audio_selection_c.borrow_mut().clear();
            audio_selection_c.borrow_mut().push_str(&out);
            //~ println!("audio_selection_c: {}", audio_selection_c.borrow());
        });
        menu.append(&menu_i);
        */
    }
    
    for (ind, val) in menu_vec.iter().enumerate() {
        //~ let val_clone = val.clone();
        let vec_other: Vec<_> = menu_vec.iter()
            .enumerate()
            .filter(|(i, x)| *i != ind)
            //~ .for_each(|(i, x)| 
            .collect();
        let mut other_clones = Vec::new();
        for (_, val) in vec_other {
            other_clones.push(val.clone());
        }
        //~ println!("{}, {:?}", ind, vec_clone);
        let val_clone = val.clone();
        let audio_selection_c = audio_selection.clone();
        val.connect_activate(move |_| {
            for i in other_clones.iter() {
                //~ println!("other menuitem: {}", i.get_label().unwrap());
                i.set_widget_name("deselected");
            }
            val_clone.set_widget_name("selected");
            audio_selection_c.borrow_mut().clear();
            audio_selection_c.borrow_mut()
                .push_str(val_clone.get_label().unwrap().as_str());
        });
        menu.append(val);
    }
    
    menu.set_widget_name("menu_popup");
    menu.show_all();
    menu_button.set_popup(Some(&menu));
    menu_button.set_widget_name("grad_btn");
    
    let test = gtk::Button::new_with_label("test");
    test.set_widget_name("grad_btn");
    test.connect_clicked(move |_| {
        tx_clone.send(String::from("")).unwrap();
    });
    
    title_box.pack_start(&label, true, true, 0);
    title_box.pack_start(&test, true, true, 0);
    title_box.pack_start(&menu_button, true, true, 0);
    main_box.pack_start(&title_box, true, true, 0);
    
    
    rx.attach(None, move |text| {
        //~ println!("{} | {}", text, audio_selection.borrow());
        //~ println!("device='{}'", audio_selection.borrow());
        
        let cwd = find_cwd(PROJECT_NAME);
        //~ let mut script = find_cwd(PROJECT_NAME);
        //~ script.push("dbus-notifyd_command.sh");
        //~ let _output = Command::new(script.to_str().unwrap())
            //~ .current_dir(cwd.to_str().unwrap())
            //~ .stdout(Stdio::null())
            //~ .output()
            //~ .expect("Failed to execute command");
            
        let _output2 = Command::new("gst-launch-1.0")
            .arg("-q")
            .arg("filesrc")
            .arg("location=./ring.wav")
            .arg("!")
            .arg("wavparse")
            .arg("!")
            .arg("audioconvert")
            .arg("!")
            .arg("audioresample")
            .arg("!")
            .arg("pulsesink")
            .arg(format!("device={}", audio_selection.borrow()))
            .current_dir(cwd.to_str().unwrap())
            .stdout(Stdio::null())
            .output()
            .expect("Failed to execute command");
            
        glib::Continue(true)
    });
    
    window.add(&main_box);
    //~ window.add(&text_view_box);
    window.show_all();
}
