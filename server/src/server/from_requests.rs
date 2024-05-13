use std::{future::Future, pin::Pin};

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::future::ok;
use tracing::{error, info};

use crate::user_system::model::UserInfo;

use super::{
    dto,
    error::{APIError, APIErrorType},
    AppState,
};

#[derive(Debug, Clone)]
pub struct UserPermission {
    owner: Option<UserInfo>,
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

    pub fn get_owner(&self) -> Result<UserInfo, APIError> {
        match &self.owner {
            Some(owner) => Ok(owner.clone()),
            None => Err(APIError::with(APIErrorType::NoPermission).note("User is not logging!")),
        }
    }

    pub fn exists_owner(&self) -> bool {
        self.owner.is_some()
    }

    pub fn is_guest(&self) -> bool {
        self.exists_owner() && self.owner.as_ref().unwrap().username == "guest"
    }
}

impl FromRequest for UserPermission {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let state = req.app_data::<web::Data<AppState>>().unwrap().clone();

        let auth_query = web::Query::<dto::AuthQuery>::from_query(req.query_string());
        let auth = if let Ok(q) = auth_query {
            Some(q.auth.to_owned())
        } else {
            'token: {
                if let Some(header) = req.headers().get("x-authorization") {
                    if let Ok(str) = header.to_str() {
                        break 'token Some(str.to_owned());
                    } else {
                        error!("Request header to_str failed!");
                    }
                };
                None
            }
        };
        if let Some(auth) = auth {
            return Box::pin(async move {
                info!("Token `{auth}` trying verify..");
                let owner = state.user_system.verify(&auth).await;
                Ok(UserPermission { owner })
            });
        } else {
            Box::pin(ok(UserPermission { owner: None }))
        }
    }
}
