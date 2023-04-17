use std::{future::Future, pin::Pin};

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::future::ok;

use crate::{myutil::DiosicID, user_system::UserSystem};

use super::{
    dto,
    error::{APIError, APIErrorType},
};

#[derive(Debug, Clone)]
pub struct UserPermission {
    owner: Option<dto::UserInfo>,
}

impl UserPermission {
    pub fn is_admin(&self) -> bool {
        if let Some(owner) = &self.owner {
            owner.is_admin
        } else {
            false
        }
    }

    pub fn have_permission_with(&self, username: &str) -> bool {
        if self.is_admin() {
            true
        } else if let Some(owner) = &self.owner {
            owner.username == username
        } else {
            false
        }
    }

    pub fn get_owner(&self) -> Result<dto::UserInfo, APIError> {
        match &self.owner {
            Some(owner) => Ok(owner.clone()),
            None => Err(APIError::with(APIErrorType::NoPermission).note("User is not logging!")),
        }
    }

    pub fn exists_owner(&self) -> bool {
        self.owner.is_some()
    }

    pub fn is_guest(&self)-> bool {
        self.exists_owner() && self.owner.as_ref().unwrap().username == "guest"
    }
}

impl FromRequest for UserPermission {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user_system = req.app_data::<web::Data<UserSystem>>().unwrap().clone();

        let auth_query = web::Query::<dto::AuthQuery>::from_query(req.query_string());
        if let Ok(q) = auth_query {
            Box::pin(async move {
                let owner = user_system.verify(&q.auth).await.map(|u| u.into());
                Ok(UserPermission { owner })
            })
        } else {
            let auth = req.cookie("authorization");
            match auth {
                Some(auth) => {
                    let id: DiosicID = auth.value().parse::<String>().unwrap().into();
                    Box::pin(async move {
                        let owner = user_system.verify(&id).await.map(|u| u.into());
                        Ok(UserPermission { owner })
                    })
                }
                None => Box::pin(ok(UserPermission { owner: None })),
            }
        }
    }
}
