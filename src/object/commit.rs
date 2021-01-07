use super::ObjectType;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use std::fmt;

pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author: User,
    pub comitter: User,
    pub message: String,
}

impl Commit {
    pub fn new(
        tree: String,
        parent: Option<String>,
        author: User,
        comitter: User,
        message: String,
    ) -> Self {
        Self {
            tree,
            parent,
            author,
            comitter,
            message,
        }
    }

    pub fn from(bytes: &[u8]) -> Option<Self> {
        // 各プロパティが改行区切り
        // commit message の間に空行が含まれるので空文字列を filter
        let mut iter = bytes.split(|&x| x == b'\n').filter(|x| x != b"");

        let tree = iter
            .next()
            .map(|x| {
                x.splitn(2, |&x| x == b' ')
                    .skip(1) // 最初は tree で決まっているので不要
                    .flatten()
                    .map(|&x| x)
                    .collect::<Vec<_>>()
            })
            .and_then(|x| String::from_utf8(x).ok())?;

        let parent = &iter
            .next()
            .map(|x| {
                x.splitn(2, |&x| x == b' ')
                    .map(Vec::from)
                    .map(|x| String::from_utf8(x).ok().unwrap_or_default())
                    .collect::<Vec<_>>()
            })
            .ok_or(Vec::new())
            .and_then(|x| match x[0].as_str() {
                "parent" => Ok(x[1].clone()),
                _ => Err([x[0].as_bytes(), b" ", x[1].as_bytes()].concat()), // parent じゃないなら元の形式に戻して Err で返す
            });

        let author = match parent {
            Ok(_) => iter.next().map(|x| Vec::from(x)), // parent なら iter から
            Err(v) => Some(v.clone()),                  // Err ならその値を使う
        }
        .map(|x| {
            x.splitn(2, |&x| x != b' ')
                .skip(1)
                .flatten()
                .map(|&x| x)
                .collect::<Vec<_>>()
        })
        .and_then(|x| User::from(x.as_slice()))?;

        let comitter = iter
            .next()
            .map(|x| {
                x.splitn(2, |&x| x != b' ')
                    .skip(1)
                    .flatten()
                    .map(|&x| x)
                    .collect::<Vec<_>>()
            })
            .and_then(|x| User::from(x.as_slice()))?;

        let message = iter
            .next()
            .map(Vec::from)
            .and_then(|x| String::from_utf8(x).ok())?;

        Some(Self::new(
            tree,
            parent.clone().ok(),
            author,
            comitter,
            message,
        ))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let content = format!("{}", self);
        let header = format!("{} {}\0", ObjectType::Commit.to_string(), content.len());
        let val = format!("{}{}", header, content);

        Vec::from(val.as_bytes())
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tree = format!("{} {}", ObjectType::Tree.to_string(), self.tree);
        let parent = self
            .parent
            .clone()
            .map(|x| format!("parent {}\n", x))
            .unwrap_or_default();
        let author = format!("author {}", self.author);
        let comitter = format!("comitter {}", self.comitter);

        write!(
            f,
            "{}\n{}{}\n{}\n\n{}\n",
            tree, parent, author, comitter, self.message,
        )
    }
}

pub struct User {
    pub name: String,
    pub email: String,
    pub ts: DateTime<FixedOffset>,
}

impl User {
    pub fn new(name: String, email: String, ts: DateTime<FixedOffset>) -> Self {
        Self { name, email, ts }
    }

    pub fn from(bytes: &[u8]) -> Option<Self> {
        let name = String::from_utf8(
            bytes
                .into_iter()
                .take_while(|&&x| x != b'<') // 関数が true を返す間の要素を得る
                .map(|&x| x)
                .collect(),
        )
        .map(|x| String::from(x.trim())) // 最後の空白はいらない
        .ok()?;

        let info = String::from_utf8(
            bytes
                .into_iter()
                .skip_while(|&&x| x != b'<')
                .map(|&x| x)
                .collect(),
        )
        .ok()?;

        let mut into_iter = info.splitn(3, " "); // <EMAIL> TIME_STAMP OFFSET の3つだけ

        let email = into_iter
            .next()
            .map(|x| String::from(x.trim_matches(|x| x == '<' || x == '>')))?;
        let ts = Utc.timestamp(into_iter.next().and_then(|x| x.parse::<i64>().ok())?, 0);
        let offset = into_iter
            .next()
            .and_then(|x| x.parse::<i32>().ok())
            .map(|x| {
                if x < 0 {
                    FixedOffset::west(x / 100 * 60 * 60)
                } else {
                    FixedOffset::east(x / 100 * 60 * 60)
                }
            })?;

        Some(Self::new(
            name,
            email,
            offset.from_utc_datetime(&ts.naive_utc()),
        ))
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} <{}> {} {:+05}",
            self.name,
            self.email,
            self.ts.timestamp(),
            self.ts.offset().local_minus_utc() / 36
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_from() {
        let name = "user";
        let email = "user@example.com";
        let ts = Utc.timestamp(0, 0);

        let ou = User::from(b"");
        assert!(ou.is_none());

        // name only
        let ou = User::from(b"user");
        assert!(ou.is_none());

        // name and email
        let ou = User::from(b"user <user@example.com>");
        assert!(ou.is_none());

        // name and email and timestamp
        let ou = User::from(b"user <user@example.com> 0");
        assert!(ou.is_none());

        // TODO: offset のテストが不十分

        // west
        let ou = User::from(b"user <user@example.com> 0 10");
        assert!(ou.is_some());
        let u = ou.unwrap();
        assert_eq!(u.name, name);
        assert_eq!(u.email, email);
        assert_eq!(u.ts, ts);

        // east
        let ou = User::from(b"user <user@example.com> 0 -10");
        assert!(ou.is_some());
        let u = ou.unwrap();
        assert_eq!(u.ts, ts);
    }
}
