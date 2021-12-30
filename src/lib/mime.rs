#[derive(Copy, Clone)]
pub struct Mime<'a> {
    mime: &'a str,
    lang: &'a str,
    charset: &'a str,
}
impl<'a> std::fmt::Display for Mime<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut m = "".to_string();
        if !self.charset.is_empty() {
            m += &format!("; charset={}", self.charset);
        }
        if !self.lang.is_empty() {
            m += &format!("; lang={}", self.lang);
        }
        write!(f, "{}{}", self.mime, m)
    }
}

impl<'a> Default for Mime<'a> {
    fn default() -> Mime<'a> {
        Mime {
            mime: "text/gemini",
            lang: "",
            charset: "",
        }
    }
}

impl<'a> Mime<'a> {
    pub fn new() -> Mime<'a> {
        Mime {
            ..Default::default()
        }
    }
    pub fn set_mime(&mut self, m: &'a str) {
        self.mime = m;
    }
    pub fn set_lang(&mut self, lang: &'a str) {
        self.lang = lang;
    }
    pub fn set_cs(&mut self, cs: &'a str) {
        self.charset = cs;
    }
}
