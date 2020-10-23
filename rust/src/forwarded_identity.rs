use rocket::http::Status;
use rocket::request::{FromRequest, Request};
use rocket::Outcome;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForwardedIdentity {
    pub email: String,
    pub forced: bool,
}

impl ForwardedIdentity {
    pub fn new<IS: Into<String>>(email: IS) -> Self {
        ForwardedIdentity {
            email: email.into(),
            forced: false,
        }
    }

    pub fn new_forced<IS: Into<String>>(email: IS) -> Self {
        ForwardedIdentity {
            email: email.into(),
            forced: true,
        }
    }

    pub fn forced(&self) -> bool {
        self.forced
    }
}

impl Display for ForwardedIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.email)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for ForwardedIdentity {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        // in env is set use it regardless
        let forced_user = std::env::var("SIMPLE_GAL_FORCED_USER");
        if forced_user.is_ok() {
            let forced_user = forced_user.unwrap();
            warn!("WARN forced user: {}", forced_user);
            return Outcome::Success(ForwardedIdentity::new_forced(forced_user));
        }

        let forwared_emails: Vec<_> = request.headers().get("X-Forwarded-Email").collect();

        if forwared_emails.len() == 1 {
            Outcome::Success(ForwardedIdentity::new(forwared_emails[0]))
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
