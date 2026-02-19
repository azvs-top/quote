use std::io::{self, Write};

/// 在执行高风险操作（update/delete）前进行再次确认。
///
/// 仅当用户输入 `yes` 或 `y`（不区分大小写）时返回 `true`。
pub(super) fn confirm_yes(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt} type 'yes' to continue: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim().to_ascii_lowercase();
    Ok(value == "yes" || value == "y")
}
