use std::collections::HashMap;
use std::str::FromStr;

use colored::*;

use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;

use clap::{Parser, Subcommand};
use mime::Mime;
use reqwest::header;
use reqwest::Client;
use reqwest::Response;
use reqwest::Url;

/// A native httpie implemention with Rust ,can you imagine how easy it is?
#[derive(Debug, Parser)] // requires `derive` feature
#[command(
    name = "httpie",
    version = "0.1.0",
    author = "mao <maol@nanofilm@com.cn>",
    about = "A native httpie implemention with Rust ,can you imagine how easy it is?"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// get Subcommand. eg: httpie get https://httpbin.org/get
    #[command(arg_required_else_help = true)]
    Get {
        /// The url for http request.
        #[arg(value_parser = parse_url)]
        url: String,
    },
    /// post Subcommand eg: httpie post https://httpbin.org/post username=admin password=123456
    #[command(arg_required_else_help = true)]
    Post {
        /// The url for http request.
        #[arg(value_parser = parse_url)]
        url: String,
        /// The body for http request.
        #[arg(value_parser = parse_kv_pair)]
        body: Vec<KvPair>,
    },
    // Delete(Delete),
}

// #[derive(Debug, Args)]
// #[command(args_conflicts_with_subcommands = true)]
// struct Delete {
//     #[arg(short, long)]
//     msg: String,
// }

#[derive(Debug, Clone)]
struct KvPair {
    k: String,
    v: String,
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

/// FromStr 是 Rust 标准库定义的 trait,实现它之后，就可以调用字符串的 parse() 泛型函数，很方便地处理字符串到某个类型的转换了。
impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('='); // 这会得到一个迭代器
        let err = || anyhow!(format!("invalid kv pair, {}", s));
        Ok(Self {
            //从迭代器中取第一个结果作为key,迭代器返回Some(T)/None
            //我们将其转换成0k(T)/E「r(E),然后用？处理错误
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

/// 处理 get 子命令
async fn get(client: Client, url: &String) -> Result<()> {
    let resp = client.get(url).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

/// 处理 post 子命令
async fn post(client: Client, url: &String, body: &Vec<KvPair>) -> Result<()> {
    let mut data = HashMap::new();
    for pair in body {
        data.insert(&pair.k, &pair.v);
    }
    let resp = client.post(url).json(&data).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

/// 打印服务器返回的版本号 + 状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

/// 打印服务器返回的 HTTP header
fn print_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {}", name.as_str().green(), value.to_str().unwrap());
    }
    print!("\n");
}

/// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            let json_data: serde_json::Value = serde_json::from_str(&body.as_str()).unwrap();
            let pretty_json = serde_json::to_string_pretty(&json_data).unwrap();
            println!("{}", pretty_json.green());
        }
        // 其它 mime type，我们直接输出
        _ => println!("{}", body),
    }
}

/// 将服务器返回的 content-type 解析成Mime类型
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

/// 打印整个响应
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 生成一个 Http 客户端
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "Httpie/0.1.0".parse()?);
    headers.insert("X-POWERED-BY", "Rust Httpie".parse()?);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let result = match &cli.command {
        Commands::Get { url } => get(client, url).await,
        Commands::Post { url, body } => post(client, url, body).await,
    };
    result?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pretty_json() -> Result<()> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, "Httpie/0.1.0".parse()?);
        headers.insert("X-POWERED-BY", "Rust Httpie".parse()?);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let url = "https://httpbin.org/post".to_string();
        let body = vec![KvPair {
            k: "name".to_string(),
            v: "ml".to_string(),
        }];
        // let _ = post(client, &url, &body).await;

        let mut data = HashMap::new();
        for pair in &body {
            data.insert(&pair.k, &pair.v);
        }
        let resp = client.post(url).json(&data).send().await?;
        let body = resp.text().await?;
        // println!("{}", serde_json::json!(body).to_string().green());
        // let res = serde_json::to_string_pretty(&body).unwrap();
        let json_data: serde_json::Value = serde_json::from_str(&body.as_str()).unwrap();

        // let json_data: serde_json::Value = resp.json().await?;
        // 将JSON对象美化后打印
        let pretty_json = serde_json::to_string_pretty(&json_data)?;
        println!("{}", pretty_json.green());
        Ok(())
    }
}
