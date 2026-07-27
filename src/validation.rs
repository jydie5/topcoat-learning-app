pub const TITLE_MAX: usize = 120;
pub const BODY_MAX: usize = 2_000;

pub fn todo(title: &str, body: &str) -> Result<(String, String), &'static str> {
    let title = title.trim();
    let body = body.trim();
    if title.is_empty() {
        return Err("タイトルを入力してください。");
    }
    if title.chars().count() > TITLE_MAX {
        return Err("タイトルは120文字以内にしてください。");
    }
    if body.chars().count() > BODY_MAX {
        return Err("本文は2000文字以内にしてください。");
    }
    Ok((title.to_owned(), body.to_owned()))
}

pub fn signup(
    display_name: &str,
    email: &str,
    password: &str,
) -> Result<(String, String), &'static str> {
    let display_name = display_name.trim();
    let email = email.trim().to_lowercase();
    if display_name.is_empty() || display_name.chars().count() > 60 {
        return Err("表示名は1〜60文字で入力してください。");
    }
    if !email.contains('@') || email.chars().count() > 254 {
        return Err("有効なメールアドレスを入力してください。");
    }
    if password.chars().count() < 8 {
        return Err("パスワードは8文字以上にしてください。");
    }
    Ok((display_name.to_owned(), email))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_todo_title() {
        assert!(todo("   ", "").is_err());
    }

    #[test]
    fn normalizes_signup_email() {
        let (_, email) = signup("Ada", " ADA@Example.COM ", "password1").unwrap();
        assert_eq!(email, "ada@example.com");
    }
}
