static SECRET_KEY: &str = "ZA1XSWSekret128cdevfraASDFlkjhHg";

pub fn hash<T: AsRef<str>>(pwd: T) -> Result<String, String> {
    bcrypt::hash(&format!("{}{}", pwd.as_ref(), SECRET_KEY), 10).map_err(|_| "hash password error".into())
}

pub fn compare<H: AsRef<str>, P: AsRef<str>>(pwd: P, hash: H) -> bool {
    matches!(
        bcrypt::verify(&format!("{}{}", pwd.as_ref(), SECRET_KEY), hash.as_ref()),
        Ok(true)
    )
}
