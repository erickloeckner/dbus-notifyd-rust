use std::env::{self, args};
use std::process::{self, Command, Stdio};
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


fn main() -> () {
    let application =
        gtk::Application::new(Some("com.github.dbus-notifyd-rust"), Default::default())
            .expect("Initialization failed...");
    
    application.connect_activate(move |app| {
        //~ let provider = gtk::CssProvider::new();
        //~ provider
            //~ .load_from_data(style.as_str().as_bytes())
            //~ .expect("Failed to load CSS");
        //~ gtk::StyleContext::add_provider_for_screen(
            //~ &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            //~ &provider,
            //~ gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        //~ );
        
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn build_ui(application: &gtk::Application) {
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
        conn.start_receive(mr, Box::new(|msg, _| {
            //~ let member_name = Member::new("Notify");
            let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
            println!("message: {} | {} : {}", sender, title, body);
            
            let output = Command::new("touch")
                .arg("test_file")
                .stdout(Stdio::null())
                .output()
                .expect("Failed to execute command");
            
            //~ if msg.member().unwrap() == member_name {
                //~ let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
                //~ println!("message: {} | {} : {}", sender, title, body);
            //~ }
            true
        }));
        
        loop {
            conn.process(Duration::from_millis(1000)).unwrap();
        }
    });
    
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("DBus Notify Daemon");
    window.set_position(gtk::WindowPosition::Center);
    
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let label = gtk::Label::new(Some("Listening for 'Incoming Call'"));
    title_box.pack_start(&label, true, true, 0);
    
    main_box.pack_start(&title_box, true, true, 0);
    
    window.add(&main_box);
    window.show_all();
}
