extern crate dbus;

use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::{MatchRule,MessageType};
//~ use dbus::strings::Member;
use std::process;
use std::time::Duration;

fn main() -> () {
    let mut conn = Connection::new_session().unwrap_or_else( |_| {
        println!("failed to connect to session bus");
        process::exit(1);
    });
    
    let mut mr = MatchRule::new_signal("org.freedesktop.Notifications", "Notify");
    mr.msg_type = Some(MessageType::MethodCall);
    
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/org/freedesktop/DBus", Duration::from_millis(5000));
    let _: () = proxy.method_call("org.freedesktop.DBus", "AddMatch", (mr.match_str(),)).unwrap_or_else( |_| {
        println!("failed on method call 'AddMatch'");
        process::exit(1);
    });
        
    
    let filter = vec!("type='method_call',interface='org.freedesktop.Notifications',member='Notify'",);
    let _: () = proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (filter, 0u32)).unwrap_or_else( |_| {
        println!("failed on method call 'BecomeMonitor'");
        process::exit(1);
    });
    
    //~ let member_name = Member::new("Notify").unwrap();
    conn.start_receive(mr, Box::new(|msg, _| {
        //~ let member_name = Member::new("Notify");
        let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
        println!("message: {} | {} : {}", sender, title, body);
        
        //~ if msg.member().unwrap() == member_name {
            //~ let (sender, _, _, title, body): (&str, u32, &str, &str, &str) = msg.read5().unwrap();
            //~ println!("message: {} | {} : {}", sender, title, body);
        //~ }
        true
    }));
    
    loop {
        conn.process(Duration::from_millis(1000)).unwrap_or_else( |_| {
            println!("connection timed out");
            process::exit(1);
        });
    }
}

