pub fn open(url: &str) -> Result<std::process::Child, std::io::Error> {
    let browser_path = r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#;
    std::process::Command::new(browser_path).arg(url).spawn()
}