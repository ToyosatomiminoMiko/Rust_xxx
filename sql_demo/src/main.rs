/*
2026.05.31.04:42:12
*/

use rusqlite::{params, Connection, OptionalExtension, Result};
use std::env;

const TABLE_NAME: &str = "test_table";

#[derive(Debug)]
struct User {
    id: u32,
    username: String,
    info: String,
    group: u32,
}

impl User {
    fn new(id: u32, username: String, info: String, group: u32) -> User {
        User {
            id,
            username,
            info,
            group,
        }
    }

    fn from_row(row: &rusqlite::Row<'_>) -> Result<User> {
        Ok(User::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    }
}

/// 初始化数据库:创建缺失的表,保证程序可以独立运行.
fn init_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS test_table (
            id INTEGER NOT NULL PRIMARY KEY,
            username VARCHAR(256) NOT NULL,
            info VARCHAR(256) NOT NULL,
            \"group\" INTEGER NOT NULL
        );",
    )
}

/// 查询下一条可用 id.这里用当前最大值 + 1,适合单机示例.
fn next_id(conn: &Connection) -> Result<u32> {
    let id: i64 = conn.query_row(
        &format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {TABLE_NAME}"),
        [],
        |row| row.get(0),
    )?;

    Ok(id as u32)
}

/// 增:新增用户,返回新用户的 id.
fn create_user(conn: &Connection, username: &str, info: &str, group: u32) -> Result<u32> {
    let id = next_id(conn)?;
    conn.execute(
        &format!(
            "INSERT INTO {TABLE_NAME} (id, username, info, \"group\")
             VALUES (?1, ?2, ?3, ?4)"
        ),
        params![id, username, info, group],
    )?;

    Ok(id)
}

/// 查:按 id 查询单个用户.
fn get_user(conn: &Connection, id: u32) -> Result<Option<User>> {
    conn.query_row(
        &format!(
            "SELECT id, username, info, \"group\"
             FROM {TABLE_NAME} WHERE id = ?1"
        ),
        params![id],
        User::from_row,
    )
    .optional()
}

/// 查:列出全部用户.
fn list_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT id, username, info, \"group\"
         FROM {TABLE_NAME} ORDER BY id"
    ))?;
    let user_iter = stmt.query_map([], User::from_row)?;

    user_iter.collect()
}

/// 改:按 id 更新用户信息.
fn update_user(conn: &Connection, id: u32, username: &str, info: &str, group: u32) -> Result<bool> {
    let changed = conn.execute(
        &format!(
            "UPDATE {TABLE_NAME}
             SET username = ?1, info = ?2, \"group\" = ?3
             WHERE id = ?4"
        ),
        params![username, info, group, id],
    )?;

    Ok(changed > 0)
}

/// 删:按 id 删除用户.
fn delete_user(conn: &Connection, id: u32) -> Result<bool> {
    let changed = conn.execute(
        &format!("DELETE FROM {TABLE_NAME} WHERE id = ?1"),
        params![id],
    )?;

    Ok(changed > 0)
}

fn print_usage(program: &str) {
    println!("用法:");
    println!("  {program} list");
    println!("  {program} get <id>");
    println!("  {program} create <username> <info> <group>");
    println!("  {program} update <id> <username> <info> <group>");
    println!("  {program} delete <id>");
}

fn print_user(user: &User) {
    println!(
        "id: {}, username: {}, info: {}, group: {}",
        user.id, user.username, user.info, user.group
    );
}

fn main() -> Result<()> {
    /*
    2022.09.13
    Debian: librust-curl-sys+openssl-sys-dev
    Arch: pkg-config openssl
    Fedora: openssl-devel
    */
    // 打开 SQLite 数据库文件(如果不存在会自动创建)
    let conn = Connection::open("./object.db")?;
    init_database(&conn)?;

    let args: Vec<String> = env::args().collect();
    let Some(command) = args.get(1).map(String::as_str) else {
        print_usage(&args[0]);
        return Ok(());
    };

    match command {
        "list" => {
            for user in list_users(&conn)? {
                print_user(&user);
            }
        }
        "get" => {
            let id = args
                .get(2)
                .ok_or_else(|| rusqlite::Error::InvalidParameterCount(3, args.len()))?
                .parse::<u32>()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            match get_user(&conn, id)? {
                Some(user) => print_user(&user),
                None => println!("没有找到 id 为 {id} 的用户"),
            }
        }
        "create" => {
            let [_, _, username, info, group] = parse_create_args(&args)?;
            let group: u32 = group.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;
            let id = create_user(&conn, username, info, group)?;
            println!("已创建用户,id = {id}");
        }
        "update" => {
            let [_, _, id, username, info, group] = parse_update_args(&args)?;
            let id: u32 = id.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;
            let group: u32 = group.parse().map_err(|_| rusqlite::Error::InvalidQuery)?;

            if update_user(&conn, id, username, info, group)? {
                println!("已更新 id 为 {id} 的用户");
            } else {
                println!("没有找到 id 为 {id} 的用户");
            }
        }
        "delete" => {
            let id = args
                .get(2)
                .ok_or_else(|| rusqlite::Error::InvalidParameterCount(3, args.len()))?
                .parse::<u32>()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            if delete_user(&conn, id)? {
                println!("已删除 id 为 {id} 的用户");
            } else {
                println!("没有找到 id 为 {id} 的用户");
            }
        }
        _ => print_usage(&args[0]),
    }

    Ok(())
}

fn parse_create_args(args: &[String]) -> Result<[&str; 5]> {
    if args.len() != 5 {
        return Err(rusqlite::Error::InvalidParameterCount(5, args.len()));
    }

    Ok([&args[0], &args[1], &args[2], &args[3], &args[4]])
}

fn parse_update_args(args: &[String]) -> Result<[&str; 6]> {
    if args.len() != 6 {
        return Err(rusqlite::Error::InvalidParameterCount(6, args.len()));
    }

    Ok([&args[0], &args[1], &args[2], &args[3], &args[4], &args[5]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        Connection::open_in_memory().expect("无法创建内存数据库")
    }

    #[test]
    fn init_database_creates_empty_table() {
        let conn = test_connection();
        init_database(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_table", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn create_and_get_user() {
        let conn = test_connection();
        init_database(&conn).unwrap();

        let id = create_user(&conn, "alice", "hello", 7).unwrap();
        assert_eq!(id, 1);

        let user = get_user(&conn, id).unwrap().expect("用户应该存在");
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "alice");
        assert_eq!(user.info, "hello");
        assert_eq!(user.group, 7);

        assert!(get_user(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn list_users_orders_by_id() {
        let conn = test_connection();
        init_database(&conn).unwrap();

        create_user(&conn, "alice", "a", 1).unwrap();
        create_user(&conn, "bob", "b", 2).unwrap();

        let users = list_users(&conn).unwrap();
        let ids: Vec<u32> = users.iter().map(|user| user.id).collect();

        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn update_user_changes_fields() {
        let conn = test_connection();
        init_database(&conn).unwrap();
        let id = create_user(&conn, "alice", "old info", 7).unwrap();

        let updated = update_user(&conn, id, "alice", "new info", 8).unwrap();
        assert!(updated);

        let user = get_user(&conn, id).unwrap().unwrap();
        assert_eq!(user.info, "new info");
        assert_eq!(user.group, 8);

        assert!(!update_user(&conn, 999, "nobody", "x", 0).unwrap());
    }

    #[test]
    fn delete_user_removes_row() {
        let conn = test_connection();
        init_database(&conn).unwrap();
        let id = create_user(&conn, "alice", "hello", 7).unwrap();

        let deleted = delete_user(&conn, id).unwrap();
        assert!(deleted);
        assert!(get_user(&conn, id).unwrap().is_none());

        assert!(!delete_user(&conn, id).unwrap());
    }

    #[test]
    fn next_id_starts_from_one_and_increases() {
        let conn = test_connection();
        init_database(&conn).unwrap();

        assert_eq!(next_id(&conn).unwrap(), 1);

        create_user(&conn, "alice", "hello", 7).unwrap();
        assert_eq!(next_id(&conn).unwrap(), 2);
    }
}
