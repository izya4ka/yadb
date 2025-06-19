use std::io::Write;

use log::Record;

pub fn log_format(
    writer: &mut dyn Write,
    now: &mut flexi_logger::DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    write!(
        writer,
        "[{}] [{}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        &record.args()
    )
}

pub fn print_logo() {
    println!(
        "
▓██   ██▓ ▄▄▄      ▓█████▄  ▄▄▄▄   
 ▒██  ██▒▒████▄    ▒██▀ ██▌▓█████▄ 
  ▒██ ██░▒██  ▀█▄  ░██   █▌▒██▒ ▄██
  ░ ▐██▓░░██▄▄▄▄██ ░▓█▄   ▌▒██░█▀  
  ░ ██▒▓░ ▓█   ▓██▒░▒████▓ ░▓█  ▀█▓
   ██▒▒▒  ▒▒   ▓▒█░ ▒▒▓  ▒ ░▒▓███▀▒
 ▓██ ░▒░   ▒   ▒▒ ░ ░ ▒  ▒ ▒░▒   ░ 
 ▒ ▒ ░░    ░   ▒    ░ ░  ░  ░    ░ 
 ░ ░           ░  ░   ░     ░      
 ░ ░                ░            ░ 
 "
    )
}
