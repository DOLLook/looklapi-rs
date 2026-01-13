use super::redipool::get_conn;
use redis::{Commands, Pipeline, RedisError, RedisResult};

// 批量执行redis命令
pub async fn multi_exec(key: &str, commands: &[(String, Vec<&str>)]) -> RedisResult<()> {
    if commands.is_empty() {
        return Ok(());
    }

    let mut conn = get_conn(key).await?;
    let mut pipe = Pipeline::new();

    for (cmd_name, args) in commands {
        match cmd_name.as_str() {
            "SET" => {
                if args.len() >= 2 {
                    pipe.cmd("SET").arg(&args[0]).arg(&args[1]);
                }
            }
            "GET" => {
                if !args.is_empty() {
                    pipe.cmd("GET").arg(&args[0]);
                }
            }
            "DEL" => {
                if !args.is_empty() {
                    pipe.cmd("DEL").arg(&args[0]);
                }
            }
            "HSET" => {
                if args.len() >= 3 {
                    pipe.cmd("HSET").arg(&args[0]).arg(&args[1]).arg(&args[2]);
                }
            }
            "HGET" => {
                if args.len() >= 2 {
                    pipe.cmd("HGET").arg(&args[0]).arg(&args[1]);
                }
            }
            "HMSET" => {
                if !args.is_empty() {
                    let mut cmd = pipe.cmd("HMSET").arg(&args[0]);
                    for arg in &args[1..] {
                        cmd.arg(arg);
                    }
                }
            }
            "LPUSH" => {
                if !args.is_empty() {
                    let mut cmd = pipe.cmd("LPUSH").arg(&args[0]);
                    for arg in &args[1..] {
                        cmd.arg(arg);
                    }
                }
            }
            "RPUSH" => {
                if !args.is_empty() {
                    let mut cmd = pipe.cmd("RPUSH").arg(&args[0]);
                    for arg in &args[1..] {
                        cmd.arg(arg);
                    }
                }
            }
            _ => {
                // 其他命令可以根据需要添加
            }
        }
    }

    let _: () = pipe.query(&mut conn)?;
    Ok(())
}

// 执行lua脚本
pub async fn eval(db_index: u8, script: &str, keys: &[&str], args: &[&str]) -> RedisResult<String> {
    let key = if !keys.is_empty() {
        keys[0]
    } else {
        ""
    };

    let mut conn = get_conn(key).await?;
    // 使用通用命令执行eval
    let result: String = redis::cmd("EVAL").arg(script).arg(keys.len()).arg(keys).arg(args).query(&mut conn)?;
    Ok(result)
}