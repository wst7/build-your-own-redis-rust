pub fn echo(args: Vec<String>) -> Result<String, String> {
    Ok(format!("+{}\r\n", args[0]))
}
