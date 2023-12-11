use cookie::{
    time::{Duration, OffsetDateTime},
    Cookie, SameSite,
};

use crate::Htmx;

pub struct CookieBuilder<'a> {
    htmx: &'a mut Htmx,
    name: &'a str,
    cookie: Option<Cookie<'a>>,
}

impl<'a> CookieBuilder<'a> {
    pub fn new(htmx: &'a mut Htmx, name: &'a str) -> Self {
        Self {
            htmx,
            name,
            cookie: None,
        }
    }

    pub fn value(&mut self, value: &str) -> &mut Self {
        let mut cookie = Cookie::new(self.name, value.to_string());
        cookie.set_secure(true);
        cookie.set_same_site(SameSite::Strict);
        cookie.set_path("/");
        self.cookie = Some(cookie);

        self
    }

    pub fn expires(&mut self, value: cookie::Expiration) -> &mut Self {
        self.cookie
            .as_mut()
            .expect("Value must be set first")
            .set_expires(value);
        self
    }

    pub fn expire_in_one_hour(&mut self) -> &mut Self {
        let mut time = OffsetDateTime::now_utc();
        time += Duration::hours(1);
        self.cookie
            .as_mut()
            .expect("Value must be set first")
            .set_expires(time);
        self
    }

    pub fn expire_in_one_day(&mut self) -> &mut Self {
        let mut time = OffsetDateTime::now_utc();
        time += Duration::days(1);
        self.cookie
            .as_mut()
            .expect("Value must be set first")
            .set_expires(time);
        self
    }

    pub fn expire_in_one_week(&mut self) -> &mut Self {
        let mut time = OffsetDateTime::now_utc();
        time += Duration::weeks(1);
        self.cookie
            .as_mut()
            .expect("Value must be set first")
            .set_expires(time);
        self
    }

    pub fn secure(&mut self, value: bool) -> &mut Self {
        self.cookie
            .as_mut()
            .expect("Value must be set first")
            .set_secure(value);
        self
    }

    pub fn get(&mut self) -> Option<String> {
        if self.cookie.is_some() {
            panic!("Value must not be set")
        }

        self.htmx
            .req
            .get_header("Cookie")
            .unwrap_or_default()
            .split(";")
            .filter_map(|cookie| Cookie::parse(cookie).ok())
            .find(|cookie| cookie.name() == self.name)
            .map(|cookie| cookie.value().to_string())
    }

    pub fn set(&mut self) {
        self.htmx.res.set_header(
            "Set-Cookie",
            &self
                .cookie
                .as_ref()
                .expect("Value must be set first")
                .to_string(),
        );
    }
}
