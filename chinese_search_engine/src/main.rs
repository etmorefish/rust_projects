use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use clap::{command, Parser};
use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use regex::Regex;

static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());

#[derive(Debug)]
struct ChineseSearchEngine {
    /// 初始化文档、文档路径、倒排索引、文档词频、文档总数
    docs: HashMap<String, String>, // 存储文档ID到文档内容的映射
    doc_path: HashMap<String, String>, //存储文档ID到文档路径的映射，便于定位文档
    inverted_index: HashMap<String, HashSet<String>>, // 倒排索引，存储词到文档ID集合的映射
    doc_term_freq: HashMap<String, HashMap<String, u32>>, // 文档词频，存储每个文档中各词的出现频率
    doc_count: i32,                    // 文档总数
}

impl ChineseSearchEngine {
    /// 初始化搜索引擎
    fn new() -> Self {
        ChineseSearchEngine {
            docs: HashMap::new(),
            doc_path: HashMap::new(),
            inverted_index: HashMap::new(),
            doc_term_freq: HashMap::new(),
            doc_count: 0,
        }
    }

    /// 添加文档
    /// 为新文档更新文档总数、内容和路径
    /// 对文档内容进行分词并更新倒排索引和词频
    fn add_document(&mut self, doc_id: String, doc_path: String, doc_content: String) {
        self.docs.insert(doc_id.clone(), doc_content.clone());
        self.doc_path.insert(doc_id.clone(), doc_path);
        self.doc_count += 1;

        // let jieba = Jieba::new();
        let tokens = JIEBA
            .cut_for_search(doc_content.as_str(), false)
            .into_iter()
            .filter(|t| *t != " ")
            .map(|t| t.to_string())
            .collect::<Vec<String>>();
        let unique_tokens = tokens.iter().collect::<HashSet<_>>();
        for token in unique_tokens {
            self.inverted_index
                .entry(token.to_string())
                .or_default()
                .insert(doc_id.clone());
        }
        let mut term_freq = HashMap::new();
        for token in tokens {
            *term_freq.entry(token.to_string()).or_insert(0) += 1;
        }
        self.doc_term_freq.insert(doc_id.clone(), term_freq);
    }

    /// 计算逆文档频率(IDF)，用于评估词的重要性
    fn compute_idf(&self, term: String) -> f64 {
        let nt = self.inverted_index.get(&term).unwrap().len();
        ((1 + self.doc_count) as f64 / (1 + nt) as f64).ln()
    }

    /// 返回与查询词匹配的文档中的一段文本。
    /// 这个实现尝试找到所有关键词匹配的最佳覆盖范围。
    fn get_text_preview(&self, doc_id: String, query: String) {}

    /// 搜索功能，支持多关键字查询
    fn search(&self, query: String) -> Vec<(String, f64)> {
        // let jieba = Jieba::new();
        let query = preprocess_text(&query);
        let query_tokens = JIEBA
            .cut_for_search(query.as_str(), false)
            .into_iter()
            .filter(|t| *t != " ")
            .map(|t| t.to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<String>>();

        let mut docs_scores = HashMap::new();
        println!("{:?}", query_tokens);
        for token in query_tokens {
            if self.inverted_index.contains_key(&token) {
                let idf = self.compute_idf(token.clone());
                for doc_id in self.inverted_index.get(&token).unwrap() {
                    let doc_freq = self.doc_term_freq.get(doc_id).unwrap();
                    let cur_token_freq = doc_freq.get(&token).unwrap();
                    let cur_token_freq_total = doc_freq.values().fold(0, |acc, x| acc + x);
                    let tf = *cur_token_freq as f64 / cur_token_freq_total as f64;
                    let entry = docs_scores.entry(doc_id.clone()).or_insert(0.0);
                    *entry += tf * idf;
                }
            }
        }

        let mut result = Vec::new();
        for (doc_id, score) in docs_scores {
            result.push((doc_id, score));
        }

        result
    }
}

/// 遍历一个文件夹及其子文件夹
/// 过滤指定的目录
/// 过滤指定的文件后缀
fn visit_dirs_and_filter(
    path: &Path,
    ignore_dirs: &[&str],
    ignore_exts: &[&str],
) -> std::io::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name.to_str().unwrap_or(""),
                None => "",
            };

            // 检查是否忽略目录
            if path.is_dir() && ignore_dirs.contains(&file_name) {
                continue; // 跳过这些目录
            } else if path.is_dir() {
                // 递归遍历目录，并将结果合并到paths中
                let mut sub_paths = visit_dirs_and_filter(&path, ignore_dirs, ignore_exts)?;
                paths.append(&mut sub_paths);
            } else {
                // 检查文件扩展名是否在忽略列表中
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ignore_exts.contains(&ext) {
                        continue; // 忽略特定后缀的文件
                    }
                }
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

/// 预处理文本，将所有的特殊符号替换成空格（" "）,有如下好处
/// 1、避免词粘连：直接移除符号可能会导致原本由符号隔开的单词粘连成一个新的、无意义的词汇。使用空格替代可以保持词汇的边界，从而避免这种情况。
/// 2、简化文本：这种方法简化了文本的结构，去除了对分词和词频分析无关紧要的字符，有助于提高后续处理的准确性和效率。
/// 3、保持词汇完整性：替换成空格而不是完全删除符号，有助于保持句子的原始结构和词汇的完整性，这对于理解文本意义和上下文非常重要。
fn preprocess_text(text: &str) -> String {
    // 使用正则表达式匹配非字母、非数字的字符，并将其替换为一个空格
    // let re = Regex::new(r"[\p{P}\p{S}]+").unwrap(); // 使用 Unicode 属性匹配标点和符号
    let re = Regex::new(r"[^\u4e00-\u9fa5\w\d]").unwrap(); // TODO: 正则表达式需要优化
    re.replace_all(text, " ").to_string()
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(
    name = "chinese_search_engine",
    version = "0.1.0",
    author = "leo <maolei025@qq.com>",
    about = "A simple Chinese search engine implemention with Rust ,can you imagine how easy it is?"
)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String, // Number of times to greet
                  // #[arg(short, long, default_value_t = 1)]
                  // count: u8,
}

fn main() {
    let args = Args::parse();
    let start = Instant::now();
    let mut engine = ChineseSearchEngine::new();
    // let path = Path::new(r"C:\Users\maol\Documents\tmp\ChineseSearchEngine\SchoolChinese");
    let path = Path::new(&args.path);

    let ignore_dirs = [".git", ".idea"];
    let ignore_exts = ["cpp", "py"];
    println!("开始加载文档!");
    let filtered_paths = visit_dirs_and_filter(path, &ignore_dirs, &ignore_exts).unwrap();
    for path in filtered_paths {
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).expect("Failed to read file");
            // 预处理文本
            let content = preprocess_text(&content);
            // 用一种方式来唯一标识每个文档，这里使用文件名
            let doc_id = path.file_name().unwrap().to_str().unwrap().to_string();
            println!("正在导入 {} .", doc_id);
            engine.add_document(doc_id, path.to_str().unwrap().to_string(), content);
        }
    }
    println!("文档加载完成!");
    let d1 = start.elapsed();
    println!("文档加载完成! 耗时 {:?}", d1); // 118.2912137s  -> 1.8479948s
    let res = engine.search("世界经济学是一门非常大的社会科学".to_string());
    println!("{:?}", res);
    let d2 = start.elapsed();
    println!("搜索耗时: {:?}", d2 - d1); // 1.0176191s  -> 6.8462ms
    println!("Done!");
}

mod tests {
    use super::*;

    #[allow(unused_imports)]
    use jieba_rs::{Jieba, KeywordExtract, TextRank, TFIDF};

    #[test]
    fn it_works() {
        let jieba = Jieba::new();
        let text = "世界经济学是一门非常大的社会科学";
        let text = "这是一个中英混合的例句，包含English words. With respect to memory usage, the impact of Unicode principally manifests through the use of Unicode character classes. Unicode character classes tend to be quite large. ";
        let words = jieba.cut(text, true);
        println!("{:?}", words);
        let words = jieba.cut_all(text);
        println!("{:?}", words);
        let words = jieba.cut_for_search(text, true);
        println!("{:?}", words);
        // assert_eq!(words, vec!["我们", "中", "出", "了", "一个", "叛徒"]);
        // ["世界", "经济学", "是", "一门", "非常", "大", "的", "社会科学"]
        // ["世", "世界", "界", "经", "经济", "经济学", "济", "济学", "学", "是", "一", "一门", "门", "非", "非常", "常", "大", "的", "社", "社会", "社会科学", "会", "科", "科学", "学"]
        // ["世界", "经济", "济学", "经济学", "是", "一门", "非常", "大", "的", "社会", "科学", "社会科学"]
        println!();

        let keyword_extractor = TFIDF::new_with_jieba(&jieba);
        let top_k = keyword_extractor.extract_tags("今天纽约的天气真好啊", 1, vec![]);
        println!("TFIDF：{:?}", top_k);

        let keyword_extractor = TextRank::new_with_jieba(&jieba);
        let top_k = keyword_extractor.extract_tags(
            "此外，公司拟对全资子公司吉林欧亚置业有限公司增资4.3亿元，增资后，吉林欧亚置业注册资本由7000万元增加到5亿元。吉林欧亚置业主要经营范围为房地产开发及百货零售等业务。目前在建吉林欧亚城市商业综合体项目。2013年，实现营业收入0万元，实现净利润-139.13万元。",
            6,
            vec![String::from("ns"), String::from("n"), String::from("vn"), String::from("v")],
        );
        println!("KeywordExtract：{:?}", top_k);

        // KeywordExtract：这是 jieba_rs 库中的一个特征提取器，用于从文本中提取关键词。它使用了 TextRank 算法，这是一种基于图排序的算法，可以用来发现文本中最重要的单词或短语。
        // TextRank：这是 TextRank 算法的实现，它可以用于关键词提取或自动摘要生成。TextRank 算法通过构建一个基于词频和词之间关系的图，然后使用图排序算法（如 PageRank）来确定文本中各个词的重要性。
        // TFIDF：这是词频-逆文档频率（Term Frequency-Inverse Document Frequency）的实现，它是一种用于信息检索和文本挖掘的常用加权技术。TFIDF 有助于评估一个词在文档集合中的重要性。
    }

    #[test]
    fn test_directory() {
        let path = Path::new(r"C:\Users\maol\Documents\tmp\ChineseSearchEngine\SchoolChinese");
        let ignore_dirs = [".git", ".idea"];
        let ignore_exts = ["cpp", "py"];
        let filtered_paths = visit_dirs_and_filter(path, &ignore_dirs, &ignore_exts).unwrap();
        for path in filtered_paths {
            println!("{:?}", path);
            match fs::read_to_string(path) {
                Ok(contents) => println!("File contents: {}", contents),
                Err(e) => println!("Error reading file: {:?}", e),
            }
        }
    }

    #[test]
    fn test_regex() {
        let re = Regex::new(
            r"(?x)
        (?P<year>\d{4})  # the year
        -
        (?P<month>\d{2}) # the month
        -
        (?P<day>\d{2})   # the day
        ",
        )
        .unwrap();

        let caps = re.captures("2010-03-14").unwrap();
        println!("{:#?}", caps);
        assert_eq!("2010", &caps["year"]);
        assert_eq!("03", &caps["month"]);
        assert_eq!("14", &caps["day"]);
    }

    #[test]
    fn test_process_text() {
        let text = "
        农历24节气是中国传统农历中的重要时间节点，用来标志季节的变化和农事活动的时间安排。每个节气都有其独特的意义和文化传统。以下是农历24节气及其代表的意思：

        1. 立春 (lì chūn) - 春季开始：立春是春季的开始，表示寒冬渐退，万物开始生长。
        
        2. 雨水 (yǔ shuǐ) - 春雨时节：雨水节气预示春雨将滋润大地，有利于农作物的生长。
        hello world!

        https://github.com/
        
        这些节气在中国文化中有着丰富的传统和习俗，与农事、节令食物、节庆等方面有关，反映了中国人民对自然环境的敏感和对季节变化的认知。
";

        let path = Path::new(
            r"C:\Users\maol\Documents\tmp\ChineseSearchEngine\SchoolChinese\c\rustlings.md",
        );
        let text = fs::read_to_string(path).expect("read_to_string failed");
        // 使用正则表达式匹配多余的空白行
        let re_empty_lines = Regex::new(r"[^\u4e00-\u9fa5\w\d]").unwrap();
        // 删除文本中的特定标点符号
        let text = text.replace("！", "").replace("？", "").replace("#", "");
        // 替换多余的空白行为单个换行符
        let text = re_empty_lines.replace_all(&text, " ");
        println!("{}", text);
    }

    #[test]
    fn test_vec() {
        let mut v = vec![1, 2, 2, 2, 3, 4, 5, 2, 4];
        // v.sort();
        // v.dedup();  // if sorted，use it
        let s = v
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<i32>>();
        println!("{:?}", s);
    }
}
