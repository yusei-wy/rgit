use super::ObjectType;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use std::fmt;

#[derive(Debug)]
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
            x.splitn(2, |&x| x == b' ')
                .skip(1)
                .flatten()
                .map(|&x| x)
                .collect::<Vec<_>>()
        })
        .and_then(|x| User::from(x.as_slice()))?;

        let comitter = iter
            .next()
            .map(|x| {
                x.splitn(2, |&x| x == b' ')
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

#[derive(Debug)]
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

    #[test]
    fn uesr_to_string() {
        let u = User::from(b"user <user@test.com> 1609643433 +0900").unwrap();
        assert_eq!(u.to_string(), "user <user@test.com> 1609643433 +0900");
    }

    #[test]
    fn commit_from() {
        let oc = Commit::from(b"");
        assert!(oc.is_none());

        // first commit
        let cs = vec![
            "tree 01a0c85dd05755281466d29983dfcb15889e1a64",
            "author author <author@example.com> 1609642799 +0900",
            "comitter comitter <comitter@example.com> 1609642799 +0900",
            "",
            "first commit",
        ]
        .join("\n");
        let oc = Commit::from(cs.as_bytes());
        assert!(oc.is_some());
        let c = oc.unwrap();
        assert_eq!(c.tree, "01a0c85dd05755281466d29983dfcb15889e1a64");
        assert!(c.parent.is_none());

        let ts = DateTime::parse_from_rfc3339("2021-01-03T11:59:59+09:00").unwrap();
        let author = User::new(
            String::from("author"),
            String::from("author@example.com"),
            FixedOffset::west(0).from_utc_datetime(&ts.naive_utc()),
        );
        let comitter = User::new(
            String::from("comitter"),
            String::from("comitter@example.com"),
            FixedOffset::west(0).from_utc_datetime(&ts.naive_utc()),
        );
        assert_eq!(c.author.name, author.name);
        assert_eq!(c.author.email, author.email);
        assert_eq!(c.author.ts, author.ts);
        assert_eq!(c.comitter.name, comitter.name);
        assert_eq!(c.comitter.email, comitter.email);
        assert_eq!(c.comitter.ts, comitter.ts);

        let cs = vec![
            "tree adb7e67378d99ab8125f156442999f187db3d1a3",
            "parent 01a0c85dd05755281466d29983dfcb15889e1a64",
            "author author <author@example.com> 1609642799 +0900",
            "comitter comitter <comitter@example.com> 1609642799 +0900",
            "",
            "second commit",
        ]
        .join("\n");
        let oc = Commit::from(cs.as_bytes());
        assert!(oc.is_some());
        let c = oc.unwrap();
        assert_eq!(c.tree, "adb7e67378d99ab8125f156442999f187db3d1a3");
        assert_eq!(
            c.parent,
            Some(String::from("01a0c85dd05755281466d29983dfcb15889e1a64"))
        );
    }

    #[test]
    fn commit_as_bytes() {
        let cs = vec![
            "tree adb7e67378d99ab8125f156442999f187db3d1a3",
            "parent 01a0c85dd05755281466d29983dfcb15889e1a64",
            "author author <author@example.com> 1609642799 +0900",
            "comitter comitter <comitter@example.com> 1609642799 +0900",
            "",
            "second commit",
        ]
        .join("\n");
        let c = Commit::from(cs.as_bytes()).unwrap();

        let content = format!("{}", c.to_string());
        let header = format!("commit {}\0", content.len());

        assert_eq!(c.as_bytes(), format!("{}{}", header, content).into_bytes(),);
    }

    #[test]
    fn commit_to_string() {
        let cs = vec![
            "tree adb7e67378d99ab8125f156442999f187db3d1a3",
            "parent 01a0c85dd05755281466d29983dfcb15889e1a64",
            "author author <author@example.com> 1609642799 +0900",
            "comitter comitter <comitter@example.com> 1609642799 +0900",
            "",
            "second commit",
        ]
        .join("\n");
        let c = Commit::from(cs.as_bytes()).unwrap();
        assert_eq!(c.to_string(), cs + "\n");
    }
}
