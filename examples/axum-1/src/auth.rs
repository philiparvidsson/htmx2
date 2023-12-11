pub trait Auth {
    fn auth(self: &mut Self, username: &str, password: &str) -> bool;
    fn auth_guard(self: &mut Self) -> Result<(), htmx2::HtmxResponse>;
    fn username(self: &mut Self) -> Option<String>;
}

impl Auth for htmx2::Htmx {
    fn auth(self: &mut Self, username: &str, password: &str) -> bool {
        // Log in with any user name since it's just a demo.
        if password == "foo123" {
            self.cookie("user_id").value(username).set();
            true
        } else {
            false
        }
    }

    fn auth_guard(self: &mut Self) -> Result<(), htmx2::HtmxResponse> {
        if self.cookie("user_id").get().is_none() {
            return Err(self.res.redirect("/sign-in"));
        }

        Ok(())
    }

    fn username(self: &mut Self) -> Option<String> {
        self.cookie("user_id").get()
    }
}
