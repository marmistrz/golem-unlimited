#![allow(dead_code)]

use actix_web::{client, HttpMessage};
use error::Error;
use futures::{future, Future};
use gu_net::rpc::peer::PeerInfo;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug, Default, Builder)]
#[builder(setter(into))]
pub struct SessionInfo {
    name: String,
    environment: String,
}

/// Represents a connection to a single hub.
#[derive(Clone)]
pub struct Driver {
    driver_inner: Arc<DriverInner>,
}

struct DriverInner {
    url: String,
}

impl Driver {
    /// creates a driver from a given URL
    pub fn from_addr(addr: &str) -> Driver {
        Driver {
            driver_inner: Arc::new(DriverInner {
                url: format!("http://{}/", addr),
            }),
        }
    }
    /// creates a new hub session
    pub fn new_session(
        &self,
        session_info_builder: &SessionInfoBuilder,
    ) -> impl Future<Item = HubSession, Error = Error> + '_ {
        let sessions_url = format!("{}{}", self.driver_inner.url, "sessions");
        let request = client::ClientRequest::post(sessions_url.clone())
            .json(
                session_info_builder
                    .build()
                    .expect("Invalid SessionInfo object."),
            ).expect("Error");
        let driver_for_session = self.clone();
        request
            .send()
            .map_err(|x| {
                println!("request.send() err: {:?}", x);
                Error::ErrorTODO
            }).and_then(|response| {
                println!("response {:?}", response);
                response.body().map_err(|x| {
                    println!("body() error {}", x);
                    Error::ErrorTODO
                })
            }).and_then(|body| {
                println!("BODY:{:?}", body);
                future::ok(HubSession { driver: driver_for_session })
            })
    }
    pub fn auth_app(&self, _app_name: String, _token: Option<String>) {}
    /// returns all peers connected to the hub
    pub fn list_peers(&self) -> impl Future<Item = impl Iterator<Item = PeerInfo>, Error = Error> {
        let url = format!("{}{}", self.driver_inner.url, "peer");
        let request = client::ClientRequest::get(url.clone())
            .finish()
            .expect(format!("Unknown URL: {}", url).as_str());
        request
            .send()
            .map_err(|_| Error::ErrorTODO)
            .and_then(|response| response.json().map_err(|_| Error::ErrorTODO))
            .and_then(|answer_json: Vec<PeerInfo>| future::ok(answer_json.into_iter()))
    }
}

pub struct HubSession {
    driver: Driver,
}
