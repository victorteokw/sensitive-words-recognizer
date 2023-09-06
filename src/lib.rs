use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::Chars;
use once_cell::sync::Lazy;

static SENSITIVE_WORD_MAP: Lazy<HashMap<char, SensitiveWordMap>> = Lazy::new(|| {
    let set = read_sensitive_word_file("sensitive.txt");
    build_sensitive_word_map(set)
});

pub enum MatchType {
    MinMatchType,
    //最小匹配规则
    MaxMatchType, //最大匹配规则
}

#[derive(Debug)]
struct SensitiveWordMap {
    word: char,
    is_end: char,
    word_map: Option<HashMap<char, Box<SensitiveWordMap>>>,
}

/// 读取敏感词库中的内容，将内容添加到set集合中
fn read_sensitive_word_file(path: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::<String>::new();
    match File::open(path) {
        Ok(f) => {
            let reader = BufReader::new(f);
            let lines = reader.lines();
            for line in lines.map(|x| x.unwrap()) {
                set.insert(line);
            }
        }
        Err(e) => panic!("can't open sensitive words file: {}", e),
    }
    set
}

/// 递归地修改map
fn recursive_build_map(map: &mut SensitiveWordMap, chars: &mut Chars, count: &mut usize) {
    if let Some(ch) = chars.next() {
        *count -= 1;
        if let Some(now_map) = map.word_map.as_mut() {
            let contains_key = now_map.contains_key(&ch);

            if contains_key {
                if let Some(m) = now_map.get_mut(&ch) {
                    recursive_build_map(&mut *m, &mut *chars, count);
                }
            } else {
                let is_end = if *count == 0 { '1' } else { '0' };
                let swm = SensitiveWordMap {
                    word: ch,
                    is_end,
                    word_map: Some(HashMap::<char, Box<SensitiveWordMap>>::new()),
                };
                now_map.insert(ch, Box::new(swm));
                if let Some(m) = now_map.get_mut(&ch) {
                    recursive_build_map(&mut *m, &mut *chars, count);
                }
            }
        }
    }
}

/// 读取敏感词库，将敏感词放入HashMap中，构建一个DFA算法模型
///  {
///   '信': SensitiveWordMap {
///       word: '信',
///       is_end: '0',
///       word_map: Some({
///           '用': SensitiveWordMap {
///               word: '用',
///               is_end: '0',
///               word_map: Some({
///                   '卡': SensitiveWordMap {
///                       word: '卡',
///                       is_end: '0',
///                       word_map: Some({
///                           '套': SensitiveWordMap {
///                               word: '套',
///                               is_end: '0',
///                               word_map: Some({
///                                   '现': SensitiveWordMap {
///                                       word: '现',
///                                       is_end: '1',
///                                       word_map: Som e({})
///                                   }
///                               })
///                           },
///                           '代': SensitiveWordMap {
///                               word: '代',
///                               is_end: '0',
///                               word_map: Some({
///                                   '付': SensitiveWordMap {
///                                       word: '付',
///                                       is_end: '1',
///                                       word_map: Some({})
///                                   },
///                                   '还': SensitiveWordMap {
///                                       word: '还',
///                                       is_end: '1',
///                                       word_map: Some({})
///                                   }
///                               })
///                           }
///                       })
///                   }
///               })
///           }
///       })
///   }
///
fn build_sensitive_word_map(set: BTreeSet<String>) -> HashMap<char, SensitiveWordMap> {
    let mut sensitive_word_map = HashMap::<char, SensitiveWordMap>::new();

    let iterator = set.iter();
    for key in iterator {
        let len = key.chars().count();
        let mut count = len;
        let mut key_chars = key.chars();
        //读取每行的首个字符
        if let Some(first_char) = key_chars.next() {
            count -= 1;
            if let Some(word_map) = sensitive_word_map.get_mut(&first_char) {
                //读取下一个字符
                recursive_build_map(&mut *word_map, &mut key_chars, &mut count);
            } else {
                let is_end = if len == 1 { '1' } else { '0' };

                let now_map = SensitiveWordMap {
                    word: first_char,
                    is_end,
                    word_map: Some(HashMap::<char, Box<SensitiveWordMap>>::new()),
                };
                sensitive_word_map.insert(first_char, now_map);

                if let Some(now_map) = sensitive_word_map.get_mut(&first_char) {
                    recursive_build_map(&mut *now_map, &mut key_chars, &mut count);
                }
            }
        }
    }

    sensitive_word_map
}

/// 递归查找map
///
fn recursive_find_map(
    swm: &SensitiveWordMap,
    txt_vec: &[char],
    i: &mut usize,
    match_flag: &mut usize,
    last_match_length: &mut usize,
    match_type: &MatchType,
) {
    if let Some(word) = txt_vec.get(*i) {
        if let Some(wm) = &swm.word_map {
            if let Some(next_swm) = wm.get(word) {
                *match_flag += 1;

                if swm.is_end == '1' {
                    *last_match_length = *match_flag;
                    match match_type {
                        MatchType::MinMatchType => {
                            return;
                        }
                        MatchType::MaxMatchType => (),
                    }
                }

                if next_swm.is_end == '1' {
                    *last_match_length = *match_flag;
                    match match_type {
                        MatchType::MinMatchType => {
                            return;
                        }
                        MatchType::MaxMatchType => (),
                    }
                }

                if let Some(nwm) = &next_swm.word_map {
                    if nwm.is_empty() {
                        *last_match_length = *match_flag;
                        match match_type {
                            MatchType::MinMatchType => {
                                return;
                            }
                            MatchType::MaxMatchType => (),
                        }
                    }
                }

                *i += 1;
                recursive_find_map(
                    &next_swm,
                    txt_vec,
                    i,
                    match_flag,
                    last_match_length,
                    match_type,
                );
            }
        }
    }
}

/// 查文字中是否包含检敏感字符,如果存在，则返回敏感词字符的长度，不存在返回0
///
fn check_sensitive_word(txt: &str, begin_index: usize, match_type: &MatchType) -> usize {
    let mut match_flag = 0;
    let mut last_match_length = 0;
    // let mut word: char;
    let txt_vec: Vec<char> = txt.chars().collect();
    // let len = txt.len();
    if let Some(word) = &txt_vec.get(begin_index) {
        if let Some(swm) = SENSITIVE_WORD_MAP.get(&word) {
            match_flag += 1;
            if (*swm).is_end == '1' {
                last_match_length = match_flag;

                match match_type {
                    MatchType::MinMatchType => {
                        return last_match_length;
                    }
                    MatchType::MaxMatchType => (),
                }
            }

            //递归查找
            let mut j = begin_index + 1;
            recursive_find_map(
                swm,
                &txt_vec,
                &mut j,
                &mut match_flag,
                &mut last_match_length,
                match_type,
            );
        }
    }
    last_match_length
}

/// 获取文字中的敏感词
///
pub fn find_sensitive_word(txt: &str, match_type: &MatchType) -> BTreeSet<String> {
    let mut sensitive_word_set = BTreeSet::<String>::new();
    let len = txt.chars().count();
    let txt_vec: Vec<char> = txt.chars().collect();
    let mut i = 0;
    while i < len {
        let length = check_sensitive_word(&txt, i, match_type);
        if length > 0 {
            //存在,加入list中
            sensitive_word_set.insert(txt_vec[i..i + length].iter().collect());
            i += length - 1; //减1的原因，是因为循环会自增
        }
        i += 1;
    }

    sensitive_word_set
}

/// 替换敏感字字符
/// # Examples
/// ```
/// let result = rust_by_example::dfa::replace_sensitive_word("信用卡之家", &MatchType::MinMatchType, '*')
/// assert_eq!(result,"**卡之家");
/// ```
pub fn replace_sensitive_word(txt: &str, match_type: &MatchType, replace_char: char) -> String {
    let set: BTreeSet<String> = find_sensitive_word(txt, match_type);
    let mut replace_str = String::from(txt);
    for word in set {
        let len = word.chars().count();
        let replace_chars: String = vec![replace_char; len].iter().collect();
        replace_str = replace_str.replace(word.as_str(), &replace_chars);
    }

    replace_str
}

#[test]
fn filter_sensitive_words() {
    let str_vec = vec![
        "花呗信用卡代还OK套现",
        "套花呗分期代付",
        "马上套现信用卡",
        "期货套利",
        "空手套白狼",
        "守信用卡脖子",
        "坚定信心,同舟共济,科学防治,精准施策",
        "D+1还是T+1秒到结算免结算费",
        "Fuck you!",
        "Son of Bitch",
    ];

    println!("replace_sensitive_word......");
    for str in &str_vec {
        let replace_str = replace_sensitive_word(str, &MatchType::MinMatchType, '*');

        println!("{} --> {}", str, replace_str);
    }
}