use console::style;

const SUCCESS_PREFIX: &str = "✅";
const ERROR_PREFIX: &str = "❌";
const DIVIDER_WIDTH: usize = 60;
const DIVIDER_CHAR: &str = "─";
const TABLE_VERTICAL: &str = "│";
const LABEL_WIDTH: usize = 15;

pub(crate) fn print_divider() {
    println!("{}", style(DIVIDER_CHAR.repeat(DIVIDER_WIDTH)).dim());
}

pub(crate) fn print_header(text: &str) {
    println!("\n{}", style(text).bold());
}

pub(crate) fn print_table_row(label: &str, value: impl AsRef<str>) {
    println!(
        "{} {:<width$} {}",
        style(TABLE_VERTICAL).dim(),
        style(label).bold(),
        value.as_ref(),
        width = LABEL_WIDTH
    );
}

pub(crate) fn print_error(message: impl AsRef<str>) {
    println!(
        "\n{} {}",
        style(ERROR_PREFIX).red(),
        style(message.as_ref()).red().bold()
    );
}

pub(crate) fn print_success(message: impl AsRef<str>) {
    println!(
        "\n{} {}",
        style(SUCCESS_PREFIX).green(),
        style(message.as_ref()).green().bold()
    );
}

pub(crate) fn truncate_arn(arn: &str) -> String {
    if arn.len() > 50 {
        format!("{}...{}", &arn[..25], &arn[arn.len() - 22..])
    } else {
        arn.to_string()
    }
}
