use std::env;
use std::fs;
use std::process;
use minigrep::Config;

fn main() {
    
    //构建配置类
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    //读取文件
    let content = fs::read_to_string(config.file_path).unwrap_or_else(|err| {
        eprintln!("read file err: {err}");
        process::exit(1);
    });

    //判断是否忽略大小写
    let result;
    if config.ignore_case {
        result = minigrep::search_case_insensitive(&config.query, &content);
    } else {
        result = minigrep::search(&config.query, &content);
    }

    //输出
    for line in result {
        println!("{line}");
    }
}
