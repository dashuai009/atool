use bitflags::bitflags;
use rand::seq::SliceRandom; // Import the trait that provides the choose method
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct GenFlags: u32 {
        const LowerCase = 0b00000001;
        const UpperCase = 0b00000010;
        const Digits = 0b00000100;
        const Spe = 0b00001000;
    }
}

#[tauri::command]
pub fn gen_pwd_cmd(flag: u32, pwd_len: u32) -> String {
    let flag  = GenFlags::from_bits(flag).unwrap_or(GenFlags::empty());
    gen_pwd(flag, pwd_len)
}


fn gen_pwd(flag: GenFlags, pwd_len: u32) -> String {
    let mut pool: Vec<char> = vec![];
    // println!("flag = {:?}", flag);
    for f in flag.iter() {
        let mut cs: Vec<_> = match f {
            GenFlags::LowerCase => ('a'..='z').collect(),
            GenFlags::UpperCase => ('A'..='Z').collect(),
            GenFlags::Digits => ('0'..='9').collect(),
            GenFlags::Spe => ('!'..='/')
                .chain(':'..='@')
                .chain('['..='`')
                .chain('{'..='~')
                .collect(),
            _ => {
                vec![]
            }
        };
        pool.append(&mut cs);
    }
    // println!("pool.len() = {}, pool = {:?}", pool.len(), pool);

    let mut rng = rand::thread_rng(); // Create a random number generator
    let mut res = String::new();
    for _i in 0..pwd_len {
        let random_element = pool.choose(&mut rng);
        res.push_str(
            &random_element
                .map(|c| c.to_string())
                .unwrap_or("".to_string()),
        );
    }
    return res;
}

#[cfg(test)]
mod test {
    use super::GenFlags;

    #[test]
    pub fn test_1() {
        let flag = GenFlags::LowerCase | GenFlags::UpperCase | GenFlags::Digits | GenFlags::Spe;
    }
}
