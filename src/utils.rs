use std::thread;
use std::time::Duration;

pub fn spawn_thread(tx: &glib::Sender<String>) {
    eprintln!("spawing thread...");
    // TODO: use clone2 macro
    thread::spawn(clone_old!(tx => move|| {
        thread::sleep(Duration::from_millis(500));
        // send result to channel
        tx.send(format!("Text from another thread"))
            .expect("Couldn't send data to channel");
    }));
}