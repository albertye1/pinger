use build_html::{Html, HtmlContainer, HtmlPage};
use std::fs::{File, remove_file};
use std::io::{Error, Write};
use std::{thread::sleep, time::Duration};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let path = "index.html";
    let mut output = File::create(path)?;
    let host_list = [
        "aly.sh",
        "florinia.aly.sh",
        "shade.albie.cat",
        "radomus.albie.cat",
    ];
    let mut handlers = JoinSet::new();
    let sleeptime = Duration::from_secs(5);

    loop {
        let mut page = HtmlPage::new()
            .with_title("Status")
            .with_stylesheet("index.css");
        page = page.with_header(1, "current status:");
        for url in host_list {
            handlers.spawn(handle_ping(&url));
        }
        while let Some(res) = handlers.join_next().await {
            match res {
                Ok(reply) => {
                    let mut text = format!("server {} is {}", reply[0], reply[1]);
                    if reply[1] == "up" {
                        text = format!("{} with response time {}", text, reply[2]);
                    }
                    page = page.with_paragraph(text);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            };
        }
        match remove_file(path) {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
            }
        }
        output = File::create(path)?;
        match write!(output, "{}", page.to_html_string()) {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
            }
        }
        sleep(sleeptime);
    }
}

async fn handle_ping(url: &str) -> Vec<String> {
    let ip = dns_lookup::lookup_host(url).unwrap();
    println!("ip: {}", ip[0]);
    let data = [1, 2, 3, 4];
    let timeout = Duration::from_secs(1);
    let options = ping_rs::PingOptions {
        ttl: 128,
        dont_fragment: true,
    };
    let res = ping_rs::send_ping(&ip[0], timeout, &data, Some(&options));
    match res {
        Ok(reply) => {
            println!(
                "reply from {}: bytes={} time={}ms ttl={}",
                reply.address,
                data.len(),
                reply.rtt,
                options.ttl
            );
            let mut ret = Vec::<String>::new();
            let rtt = &reply.rtt.to_string();
            ret.push(url.to_string());
            ret.push("up".to_string());
            ret.push(rtt.clone());
            return ret;
        }
        Err(e) => {
            println!("{:?}", e);
            let mut ret = Vec::<String>::new();
            ret.push(url.to_string());
            ret.push("down".to_string());
            return ret;
        }
    };
}
