use std::{
    collections::HashSet,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use log::{debug, error, info, warn};
use thirtyfour::{error::WebDriverError, By, DesiredCapabilities, WebDriver};

async fn work(url: &str) {
    debug!("Starting Firefox");
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://127.0.0.1:9515", caps).await.unwrap();

    let caps = DesiredCapabilities::chrome();
    let dr1 = WebDriver::new("http://127.0.0.1:9515", caps).await.unwrap();

    driver.get(url).await.unwrap();
    dr1.get("https://love.noxue.com/?i=杨过&u=小龙女")
        .await
        .unwrap();
    loop {
        let ele = match driver
            .find_element(By::Css(".webcast-chatroom___items"))
            .await
        {
            Ok(ele) => ele,
            Err(e) => {
                warn!("{}", e);
                match e {
                    WebDriverError::NoSuchElement(..) => {
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    _ => {
                        panic!("{}", e);
                    }
                }
            }
        };

        let mut msgs = vec![];
        let mut maps = HashSet::new();
        loop {
            // sleep 1s
            thread::sleep(std::time::Duration::from_millis(1000));

            debug!("============开始获取===========");
            // 遍历所有的节点
            let nodes = match ele
                .find_elements(By::Css(".webcast-chatroom___item[data-id]"))
                .await
            {
                Ok(nodes) => nodes,
                Err(e) => match e {
                    WebDriverError::NoSuchElement(..) => {
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    _ => {
                        panic!("{}", e);
                    }
                },
            };

            debug!("============获取到{}个节点==========", nodes.len());
            for node in nodes {
                let data_id = match node.attr("data-id").await {
                    Ok(id) => match id {
                        Some(id) => id,
                        None => continue,
                    },
                    Err(e) => {
                        debug!("{}", e);
                        continue;
                    }
                };
                if maps.contains(&data_id) {
                    continue;
                }
                debug!("============开始获取span==========");
                let spans = match node.find_elements(By::Css("span")).await {
                    Ok(spans) => spans,
                    Err(e) => match e {
                        WebDriverError::NoSuchElement(..) => {
                            thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        }
                        _ => {
                            panic!("{}", e);
                        }
                    },
                };
                debug!("============获取到{}个span==========", spans.len());
                debug!("======================");
                // debug!("data-id:{:?}", data_id);
                // debug!("spans:{:#?}", spans.().await);
                let mut pos = None;
                for i in 0..spans.len() {
                    let text = match spans[i].text().await {
                        Ok(text) => text,
                        Err(e) => {
                            debug!("{}", e);
                            continue;
                        }
                    };
                    if text.contains("：") {
                        pos = Some(i);
                        break;
                    }
                }
                let pos = if let Some(pos) = pos {
                    pos
                } else {
                    debug!("pos is None");
                    continue;
                };

                let name = match spans[pos].text().await {
                    Ok(name) => name.trim_end_matches("：").to_string(),
                    Err(e) => {
                        warn!("{}", e);
                        match e {
                            WebDriverError::NoSuchElement(..) => {
                                thread::sleep(std::time::Duration::from_secs(1));
                                continue;
                            }
                            _ => {
                                panic!("{}", e);
                            }
                        }
                    }
                };

                if spans.get(pos + 1).is_none() {
                    debug!("{}", "no text");
                    continue;
                }

                let text = match spans[pos + 1].text().await {
                    Ok(text) => text,
                    Err(e) => {
                        debug!("{}", e);
                        match e {
                            WebDriverError::NoSuchElement(..) => {
                                thread::sleep(std::time::Duration::from_secs(1));
                                continue;
                            }
                            _ => {
                                panic!("{}", e);
                            }
                        }
                    }
                };

                maps.insert(data_id.clone());
                msgs.push(format!("{}：{}", name, text));
                info!("{}:{}", name, text);

                if !text.contains("520") || text.len() < 6 {
                    continue;
                }
                // 不学网 杨过520小龙女
                // 正则提取
                let re = match regex::Regex::new(r"^(.*?)520(.*?)$") {
                    Ok(re) => re,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                };
                let caps = match re.captures(&text) {
                    Some(caps) => caps,
                    None => {
                        error!("格式不正确，例子:杨过520小龙女");
                        continue;
                    }
                };
                let me = caps.get(1);
                let you = caps.get(2);
                if me.is_none() || you.is_none() {
                    error!("格式不正确，例子:杨过520小龙女");
                    continue;
                }

                let me = me.unwrap().as_str().trim();
                let you = you.unwrap().as_str().trim();
                if me.is_empty() || you.is_empty() {
                    error!("格式不正确，例子:杨过520小龙女");
                    continue;
                }

                if let Err(e) = dr1
                    .get(format!("https://love.noxue.com/?i={}&u={}", me, you))
                    .await
                {
                    error!("{}", e);
                    match e {
                        WebDriverError::NoSuchElement(..) => {
                            thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        }
                        _ => {
                            panic!("{}", e);
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    log4rs::init_file("log.yml", Default::default()).unwrap();
    // 获取命令行参数 url
    let args: Vec<String> = std::env::args().collect();
    let url = if args.len() > 1 {
        &args[1]
    } else {
        // 用法
        println!("Usage: {} url", args[0]);
        return;
    };
    work(url).await;
}
