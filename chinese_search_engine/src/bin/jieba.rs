use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
};

// 构建词典
fn build_dict() -> HashMap<String, f32> {
    let mut dict_words = HashMap::new();
    dict_words.insert("世界".to_string(), 0.5);
    dict_words.insert("经济".to_string(), 0.5);
    dict_words.insert("经济学".to_string(), 0.8);
    dict_words.insert("是".to_string(), 0.2);
    dict_words.insert("一门".to_string(), 0.1);
    dict_words.insert("社会".to_string(), 0.3);
    dict_words.insert("科学".to_string(), 0.4);
    dict_words
}

// 加载用户自定义词典
fn load_user_dict(user_dict_path: &str, dict_words: &mut HashMap<String, f32>) -> io::Result<()> {
    let file = File::open(user_dict_path)?;
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() == 2 {
            let word = words[0].to_string();
            let weight = words[1].parse::<f32>().unwrap_or(0.0);
            dict_words.insert(word, weight);
        } else {
            let word = words[0].to_string();
            let weight = 0.1;
            dict_words.insert(word, weight);
        }
    }
    Ok(())
}

// 最大正向匹配算法
fn max_forward_matching(
    text: &str,
    dict_words: HashMap<String, f32>,
    max_word_length: usize,
) -> Vec<String> {
    let _ = dict_words;
    let mut words = Vec::new();
    let mut start = 0;

    while start < text.chars().count() {
        let mut matched = false;
        let mut end = start + max_word_length;

        if end > text.chars().count() {
            end = text.chars().count();
        }

        while start < end {
            let word = text
                .chars()
                .skip(start)
                .take(end - start)
                .collect::<String>();
            if dict_words.contains_key(&word) {
                words.push(word.clone());
                start = end;
                matched = true;
                break;
            }
            end -= 1;
        }

        if !matched {
            let single_char: String = text.chars().nth(start).unwrap().to_string();
            words.push(single_char);
            start += 1;
        }
    }

    words
}

fn main() {
    let mut dict_words = build_dict();
    let text = "世界经济学是一门非常大的社会科学";

    if let Err(e) = load_user_dict("chinese_seach_engine/src/user_dict.txt", &mut dict_words) {
        eprintln!("Failed to load user dictionary: {}", e);
    }
    let s = max_forward_matching(text, dict_words, 5);
    println!("{:?}", s);
}

mod tests {
    use super::*;

    #[test]
    fn test_range() {
        for i in (1..=5).rev().step_by(1) {
            println!("{}", i);
        }

        let s = "hello world";
        println!("{}", s.len());
        let _slice = &s[0..5]; // slice 包含 "hello"
                               // assert_eq!(slice, "hello");
                               // assert_eq!(s.len(), 11);

        let text = "世界经济学是一门非常大的社会科学";
        println!("{}", text.chars().count());
        println!("{}", &text[3..6]);
    }

    #[test]
    fn test_max_forward_matching() {
        let text = "世界经济学是一门非常大的社会科学";
        let dict_words = build_dict();

        let res = max_forward_matching(text, dict_words, 5);
        println!("{:?}", res);
    }
}
