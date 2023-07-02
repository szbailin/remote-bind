use core::client::Client;
use core::log::*;

pub static mut SERVER: Option<String> = None;
pub static mut PORT: u16 = 0;
pub static mut PASSWORD: Option<String> = None;
pub static mut LOCAL_SERVICE: Option<String> = None;

fn print_help() {
    println!(
        r#"Help:
{} [server:port] [bind_port] [password] [local_service:port]
"#,
        std::env::args().nth(0).unwrap()
    );
}

fn main() {
    let mut args = std::env::args();
    if args.len() < 5 {
        return print_help();
    }
    unsafe {
        SERVER = Some(args.nth(1).unwrap());
        PORT = match args.next().unwrap().parse() {
            Ok(p) => p,
            Err(_) => {
                println!(
                    "端口号错误: {}, 请选择一个1~65535之间的端口号\n",
                    std::env::args().nth(2).unwrap()
                );
                return print_help();
            }
        };
        PASSWORD = args.next().map(|s| s.to_string());
        LOCAL_SERVICE = args.next().map(|s| s.to_string());
    }
    loop {
        serv();
        std::thread::sleep(std::time::Duration::from_millis(5000));
    }
}

fn serv() {
    let server = unsafe { SERVER.as_ref().unwrap() };
    let port = unsafe { PORT };
    let password = unsafe { PASSWORD.as_ref().unwrap() };
    let local_service = unsafe { LOCAL_SERVICE.as_ref().unwrap() };
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(boot(server.into(), port, password.into(), local_service.into()));
}

async fn boot(server: String, port: u16, password: String, local_service: String) {
    i!("正在连接服务器：{server}");
    let mut c = match Client::new(server.clone(), password).await {
        Ok(v) => v,
        Err(e) => {
            return e!("连接失败！{}", e.to_string());
        }
    };
    i!("正在绑定端口：{port}");
    match c.bind(port).await {
        Ok(()) => {
            let host = server.split(":").next().unwrap();
            i!("服务已绑定: {} -> {}:{}", local_service, host, port);
            c.proxy(local_service, |_task| {
                async move {
                    // task.abort();
                }
            }).await;
        }
        Err(e) => e!("绑定失败！{}", e.to_string()),
    };
}