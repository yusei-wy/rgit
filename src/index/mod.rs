use chrono::{DateTime, TimeZone, Utc};
use std::fmt;

pub struct Index {
    pub entries: Vec<Entry>,
}

impl Index {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    pub fn from(bytes: &[u8]) -> Option<Self> {
        // インデックスファイルじゃない
        if &bytes[0..4] != b"DIRC" {
            return None;
        }

        // version 2 にだけ対応
        if hex_to_num(&bytes[4..8]) != 2 {
            return None;
        }

        let entry_num = hex_to_num(&bytes[8..12]);
        let entries = (0..entry_num)
            .try_fold((0, Vec::new()), |(offs, mut vec), _| {
                let entry = Entry::from(&bytes[(12 + offs)..])?;
                let size = entry.size();
                vec.push(entry);
                Some((offs + size, vec))
            })
            .map(|(_, entries)| entries)?;

        Some(Self::new(entries))
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries.iter().try_for_each(|e| write!(f, "{}\n", e))
    }
}

pub struct Entry {
    pub c_time: DateTime<Utc>,
    pub m_time: DateTime<Utc>,
    pub dev: u32,
    pub inode: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub hash: Vec<u8>,
    pub name: String,
}

impl Entry {
    pub fn from(bytes: &[u8]) -> Option<Self> {
        let c_time = hex_to_num(&bytes[0..4]);
        let c_time_nano = hex_to_num(&bytes[4..8]);
        let m_time = hex_to_num(&bytes[8..12]);
        let m_time_nano = hex_to_num(&bytes[12..16]);
        let dev = hex_to_num(&bytes[16..20]);
        let inode = hex_to_num(&bytes[20..24]);
        let mode = hex_to_num(&bytes[24..28]);
        let uid = hex_to_num(&bytes[28..32]);
        let gid = hex_to_num(&bytes[32..36]);
        let size = hex_to_num(&bytes[36..40]);
        let hash = Vec::from(&bytes[40..60]);
        let name_size = hex_to_num(&bytes[60..62]);
        let name = String::from_utf8(Vec::from(&bytes[62..(62 + name_size as usize)])).ok()?;

        Some(Self {
            c_time: Utc.timestamp(c_time.into(), c_time_nano),
            m_time: Utc.timestamp(m_time.into(), m_time_nano),
            dev,
            inode,
            mode,
            uid,
            gid,
            size,
            hash,
            name,
        })
    }

    pub fn size(&self) -> usize {
        let size = 62 + self.name.len();
        size + (8 - size % 8)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} 0\t{}",
            num_to_mode(self.mode as u16),
            hex::encode(&self.hash),
            self.name
        )
    }
}

// バイト列になっている値を1つの整数として変換する
// ex: hex_to_num(&[0x00, 0x00, 0x02, 0x62]) -> -0x0262
fn hex_to_num(hex: &[u8]) -> u32 {
    hex.iter()
        .rev()
        .fold((0u32, 1u32), |(sum, offs), &x| {
            (sum + (x as u32 * offs), offs << 8)
        })
        .0
}

fn num_to_mode(val: u16) -> String {
    let file_type = val >> 13;
    let (user, group, other) = {
        let permission = val & 0x01ff;
        let user = (permission & 0x01c0) >> 6;
        let group = (permission & 0x0038) >> 3;
        let other = permission & 0x0007;

        (user, group, other)
    };

    format!("{:03b}{}{}{}", file_type, user, group, other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_num_test() {
        assert_eq!(hex_to_num(&[]), 0);
        assert_eq!(hex_to_num(&[0x00, 0x00, 0x00, 0x02]), 2);
        assert_eq!(hex_to_num(&[0x00, 0x00, 0x02, 0x62]), 610);
    }
}
