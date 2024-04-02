```shell
//运行命令油三个参数， 第一个query， 第二个 file_path, 第三个 是否忽略大小写
cargo run -- nobody poem.txt 1  

//环境变量的优先级大于 参数
IGNORE_CASE=1 cargo run -- The poem.txt 0  

```

