mod rpc;



use crate::{error::{Error, Result}, job::Job, share::Share};

use rpc::{
    request::{KeepAlivedParams, LoginParams, Request, SubmitParams},
    response::{LoginResult, Response, StatusResult},
};
use serde::Deserialize;
use std::{
    io::{BufReader, BufWriter},
    net::TcpStream,

    sync::{mpsc::{self, Receiver}, Arc, Mutex},
    thread,
};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum PoolMessage {
    Response(Response<StatusResult>),
    NewJob(Request<Job>),
}
#[derive(Debug)]
pub struct Stratum {
    login_id: String,
    writer: BufWriter<TcpStream>,
    job_rx: Receiver<Job>,
}

impl Stratum {
    #[tracing::instrument]
    pub fn login(url: &str, user: &str, pass: &str) -> Result<Self> {
        let stream = TcpStream::connect(url)?;
        stream.set_read_timeout(None)?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream.try_clone()?);

        let (job_tx, job_rx) = mpsc::channel();

        rpc::send(
            &mut writer,
            &Request::<LoginParams>::new(LoginParams {
                login: user.into(),
                pass: pass.into(),
            }),
        )?;
        let response = rpc::recv::<Response<LoginResult>>(&mut reader)?;
        if let Some(result) = response.result {
            tracing::info!("success");
            let LoginResult { id, job, .. } = result;
            job_tx.send(job).map_err(Error::from)?;
            thread::spawn(move || {
                let span = tracing::info_span!("listener");
                let _enter = span.enter();
                loop {
                    let msg = match rpc::recv::<PoolMessage>(&mut reader) {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::error!("Listener error: {}", e);
                            break;
                        }
                    };
                    match msg {
                        PoolMessage::Response(response) => {
                            if let Some(err) = response.error {
                                tracing::warn!("{}", err.message);
                            } else {
                                match response.result.unwrap().status.as_str() {
                                    "OK" => tracing::info!("accepted"),
                                    "KEEPALIVED" => tracing::debug!("keepalived"),
                                    _ => todo!(),
                                }
                            }
                        }
                        PoolMessage::NewJob(request) => {
                            tracing::info!("new job");
                            if let Err(e) = job_tx.send(request.params) {
                                tracing::warn!("Failed to send job: {}", e);
                            }
                        }
                    }
                }
            });
            Ok(Self {
                login_id: id,
                writer,
                job_rx,
            })
        } else {
            let msg = response.error.map(|e| e.message).unwrap_or_default();
            tracing::warn!("{}", msg);
            Err(Error::Stratum(msg))
        }
    }
    pub fn submit(&mut self, share: Share) -> Result<()> {
        rpc::send(
            &mut self.writer,
            &Request::<SubmitParams>::new(SubmitParams {
                id: self.login_id.clone(),
                job_id: share.job_id,
                nonce: share.nonce,
                result: share.hash,
            }),

        ).map_err(Error::from)
    }
    pub fn keep_alive(&mut self) -> Result<()> {
        rpc::send(
            &mut self.writer,
            &Request::<KeepAlivedParams>::new(KeepAlivedParams {
                id: self.login_id.clone(),
            }),

        ).map_err(Error::from)
    }
    pub fn try_recv_job(&self) -> Result<Job> {
        self.job_rx.try_recv().map_err(Error::from)
    }
}
