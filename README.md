# sensitive-words-recognizer

This is a sensitive word crate.

The Features:

- replace_sensitive_word()

```
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
```