use std::{
    env, fs,
    io::{self, Error, ErrorKind, Write},
    str::Chars,
    thread,
    time::Duration,
};
/*
 * UTF-8
 * # 1byte(000-127): 0*******
 * # 2byte(192-223): 110***** 10******
 * # 3byte(224-239): 1110**** 10****** 10******
 * # 4byte(240-247): 11110*** 10****** 10****** 10******
 */

macro_rules! print_char {
    /*
     *    标准输出中的这种行缓冲机制
     *    在遇到换行符之前,输出的内容并不会隐式的刷新
     *    这就导致 print! 宏和 println! 宏实际上并不完全相同
     */
    ($($arg:tt)*) =>{
        print!($($arg)*);
        io::stdout().flush().unwrap();
    }
}

fn typist(s: &str, t: u64) {
    let mut chars: Chars = s.chars();
    loop {
        let char: Option<char> = chars.next();
        match char {
            Some(char) => {
                print_char!("{}", char);
                thread::sleep(Duration::from_millis(t));
            }
            None => break,
        }
    }
}

/*
```shell
cargo run ./test.txt 25
```
*/
fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect(); // 接收命令行传入
    // map_err() 将其他错误转换成 std::io::Error并附上自定义的错误描述
    // 检查参数数量
    if args.len() < 3 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "需要两个参数:文件路径和延迟毫秒数",
        ));
    }

    // 解析时间参数,将可能的解析错误转换为 io::Error
    let time: u64 = args[2]
        .parse()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "时间参数必须是有效的整数"))?;

    // 读取文件内容,错误自动向上传播
    let content: Vec<u8> = fs::read(&args[1])?;

    // 将字节转换为 UTF-8 字符串,错误转换为 io::Error
    let text: &str =
        std::str::from_utf8(&content).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    print!("\x1b[3J\x1b[2J\x1b[H");
    typist(text, time);
    Ok(())
}
