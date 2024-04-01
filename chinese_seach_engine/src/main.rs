use jieba_rs::Jieba;


fn main() {}

mod tests {
    #[allow(unused_imports)]
    use jieba_rs::{Jieba, KeywordExtract, TextRank, TFIDF};

    #[test]
    fn it_works() {
        let jieba = Jieba::new();
        let text = "世界经济学是一门非常大的社会科学";
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
        let top_k = keyword_extractor.extract_tags(
            "今天纽约的天气真好啊",
            1,
            vec![],
        );
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
}
