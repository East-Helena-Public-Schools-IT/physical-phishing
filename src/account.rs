use std::collections::HashSet;
use std::fmt::Display;
use std::sync::Arc;

use argon2::Argon2;
use password_hash::rand_core::OsRng;
use password_hash::{PasswordHash, SaltString};
use poem::http::StatusCode;
use poem::{
    web::cookie::SameSite,
    middleware::CookieJarManager,
    session::CookieConfig,
    IntoResponse, Request, Endpoint, Middleware
};
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::FILESYSTEM_ROOT;

pub static COOKIE_JAR_NAME: &str = "clocks";


pub mod simple_storage {
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::RwLock;
    use tracing::{debug, error, warn};
    use super::{Account, AccountDB};

    #[derive(Clone)]
    pub struct FilesystemDB {
        data: Arc<RwLock<HashMap<String, Account>>>,
    }
    
    impl AccountDB for FilesystemDB {
        

        async fn account_with_name(&self, name: &str) -> Option<Account> {
            let data = self.data.read().await;
            data.get(name).cloned()
        }

        async fn init() -> Self {
            let path = "./import.csv";
            let db: Arc<RwLock<HashMap<String, Account>>> = Default::default();
            let csv = csv::Reader::from_path(path);

            let mut lock = db.write().await;

            match csv {
                Ok(mut r) => {
                    for i in r.deserialize::<Account>() {
                        match i {
                            Ok(acct) => {
                                debug!("Importing: {:?}", acct.username);
                                lock.insert(acct.username.clone(), acct);
                            },
                            Err(e) => {
                                error!("Error whilst importing account. {}", e);
                            },
                        }
                    }
                },
                Err(e) => {
                    // TODO, if the file isn't found create random admin credentials and print them out.
                    error!("Could not read csv: \"{:?}\". {}", path, e);
                    warn!("Continuing without loading any accounts!");
                }
            }
            drop(lock);

            Self { data: db }
        }
        
        type Output = Self;
    }
}

/// Methods for interacting with a database
pub trait AccountDB {
    type Output;

    fn account_with_name(&self, name: &str) -> impl std::future::Future<Output = Option<Account>> + Send;

    async fn init() -> Self::Output;
}


#[derive(Clone, Deserialize)]
pub struct Account {
    username: String,
    hashed_password: String,
    salt: String,
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}",self.username,self.hashed_password,self.salt) 
    }
}


impl Account {
    pub fn try_new(username: &str, password: &str) -> Result<Self, password_hash::Error> {
        let salt = SaltString::generate(OsRng);
        match PasswordHash::generate(Argon2::default(), password, &salt) {
            Ok(hash) => {
                return Ok(Self {
                    username: username.to_owned(),
                    hashed_password: hash.to_string(),
                    salt: salt.as_str().to_string(),
                });
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    async fn is_valid_creds<A>(
        accounts: &A,
        username: &str,
        password: &str,
    ) -> bool where A: AccountDB,
    {
        // find account with same username
        if let Some(account) = accounts.account_with_name(username).await {
            // retrieve the salt
            if let Ok(salt) = SaltString::from_b64(&account.salt) {
                // hash the incoming password to see if it is the same as the saved one
                if let Ok(hash) = PasswordHash::generate(Argon2::default(), password, &salt) {
                    if hash.to_string() == account.hashed_password {
                       return true;
                    }
                }
            }
        }
        false
    }
}

type SessionDB = Arc<RwLock<HashSet<String>>>;
#[derive(Clone)]
pub struct AppState<A>
where A: AccountDB
{
    active_sessions: SessionDB,
    cookie_config: Arc<CookieConfig>,
    accounts: A,
}

impl<A> AppState<A>
where A: AccountDB<Output = A> 
{
    pub async fn new() -> Self {
        let cc = CookieConfig::default()
            .name(COOKIE_JAR_NAME)
            .same_site(SameSite::Strict)
            .max_age(None);

        Self {
            cookie_config: Arc::new(cc),
            active_sessions: SessionDB::default(),
            accounts: A::init().await,
        }
    }
}

impl<A: AccountDB + Clone + Send + Sync, E: Endpoint> Middleware<E> for AppState<A> {
    type Output = poem::middleware::CookieJarManagerEndpoint<ServerSessionEndpoint<E, A>>;

    fn transform(&self, ep: E) -> Self::Output {
        CookieJarManager::new().transform(ServerSessionEndpoint {
            // These are both Arc, thus cloning is cheap
            config: self.cookie_config.clone(),
            sessions: self.active_sessions.clone(),
            endpoint: ep,
            accounts: self.accounts.clone(),
        })
    }
}

pub struct ServerSessionEndpoint<E, A> {
    endpoint: E,
    config: Arc<CookieConfig>,
    sessions: SessionDB,
    accounts: A
}

impl<E, A> Endpoint for ServerSessionEndpoint<E, A>
where E: Endpoint,
      A: AccountDB + Clone + Send+ Sync,
{
    type Output = poem::Response;

    async fn call(&self, req: Request) -> Result<Self::Output, poem::Error> {
        let cookie_jar = req.cookie().clone();
        // the config holds the keys to the cookie jar if it's locked. That's why we need self.config
        let client_session_id = self.config.get_cookie_value(&cookie_jar);

        // Up next:
        // check if the client's session id is valid
        // if it doesn't exist try to sign in
        // if the session is invalid and signing in doesn't work - reject request

        if let Some(id) = client_session_id {
            let server_sessions = self.sessions.read().await;
            if server_sessions.get(&id).is_some() {
                // Client's session is found on the server.
                // Go ahead and call the endpoint.
                let res = self.endpoint.call(req).await?;
                return Ok(res.into_response());
            }
        }
        // couldn't validate req via the cookie
        // try to login the user
        
        // Try to log the user in
        if let Some(username) = req.header("username") { // These are case-insensitive keys
            if let Some(password) = req.header("password") {
                if Account::is_valid_creds(&self.accounts, username, password).await {
                    // set session id
                    let uuid = Uuid::default().to_string();
                    /* server */ self.sessions.write().await.insert(uuid.clone());
                    /* client */ self.config.set_cookie_value(&cookie_jar, &uuid);
                    let res = self.endpoint.call(req).await?;
                    return Ok(res.into_response());
                }
            }
        }


        // this request couldn't be validated in any way.
        // tell them to login
        let x = poem::endpoint::StaticFileEndpoint::new(FILESYSTEM_ROOT.to_owned() + "/login.html");
        let mut res = x.call(req).await?;
        res.set_status(StatusCode::UNAUTHORIZED);
        Ok(res)
   }
}

