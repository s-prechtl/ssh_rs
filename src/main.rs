use ssh2::Session;
use std::io::{prelude::*, stdin, stdout};
use std::net::TcpStream;
use std::path::Path;

fn ls(path: &str, channel: &mut ssh2::Channel) {
    println!("--- CONTENT IN {} ---", path);
    channel.exec(&format!("ls -l {}", path)).unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
}

fn prompt_password() -> String {
    let mut buffer = String::new();
    print!("Enter password: ");
    stdout().flush().unwrap();
    let _ = stdin().read_line(&mut buffer);
    return match buffer.trim_end() {
        "" => panic!("No password provided"),
        password => password.to_string(),
    };
}

fn main() {
    let tcp = TcpStream::connect("test.rebex.net:22").unwrap();
    let mut sess = Session::new().unwrap();
    let password = &prompt_password();

    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("demo", password).unwrap();

    let mut channel = sess.channel_session().unwrap();
    channel.exec("pwd").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("Current working directory: {}", s);

    channel = sess.channel_session().unwrap();
    ls(".", &mut channel);
    channel = sess.channel_session().unwrap();
    ls("./pub/example", &mut channel);

    println!("---  Downloading readme.txt and printing contents ---");
    let (mut remote_file, stat) = sess.scp_recv(Path::new("./readme.txt")).unwrap();
    println!("remote file size: {}", stat.size());
    let mut contents = Vec::new();
    remote_file.read_to_end(&mut contents).unwrap();

    println!("{}", String::from_utf8_lossy(&contents));

    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();

    let _ = channel.wait_close();
    println!("Connection closed {}", channel.exit_status().unwrap());
}
