use regex::Regex;

pub fn clean_error_output(error_output: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[mG]").unwrap();
    let without_color_outputs = re.replace_all(error_output, "");

    let lines: Vec<&str> = without_color_outputs.split("\n").collect();
    let clean_message: String = lines
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<&str>>()
        .join("\n");

    clean_message
}
