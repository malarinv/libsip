mod registration;
pub use self::registration::RegistrationManager;

mod messaging;
pub use self::messaging::{MessageHelper, MessageWriter};

mod invite;
pub use self::invite::InviteHelper;

use crate::{Header, Headers, Method, ResponseGenerator, SipMessage, Uri};

use std::io::Result as IoResult;

pub struct HeaderWriteConfig {
    pub user_agent: Option<String>,
    pub allowed_methods: Option<Vec<Method>>,
}

impl HeaderWriteConfig {
    pub fn write_headers_vec(&self, m: &mut Vec<Header>) {
        if let Some(agent) = &self.user_agent {
            m.push(Header::UserAgent(agent.into()));
        }
        if let Some(allowed) = &self.allowed_methods {
            m.push(Header::Allow(allowed.clone()));
        }
    }

    pub fn write_headers(&self, m: &mut Headers) {
        if let Some(agent) = &self.user_agent {
            m.push(Header::UserAgent(agent.into()));
        }
        if let Some(allowed) = &self.allowed_methods {
            m.push(Header::Allow(allowed.clone()));
        }
    }
}

impl Default for HeaderWriteConfig {
    fn default() -> HeaderWriteConfig {
        HeaderWriteConfig {
            user_agent: Some(format!("libsip {}", env!("CARGO_PKG_VERSION"))),
            allowed_methods: Some(vec![
                Method::Invite,
                Method::Cancel,
                Method::Bye,
                Method::Message,
            ]),
        }
    }
}

/// Simple SIP client for implementing softphones.
/// Currently the only thing implemented is registration
/// and sending text messages. The only other feature planned
/// is an interface for sending & receiving calls.
pub struct SoftPhone {
    header_cfg: HeaderWriteConfig,
    msg: MessageWriter,
    reg: RegistrationManager,
}

impl SoftPhone {
    /// Create a new SoftPhone client. `local_uri` is the SipUri that you listen on
    /// and `account_uri` is the uri of your SIP user account.
    pub fn new(local_uri: Uri, account_uri: Uri) -> SoftPhone {
        SoftPhone {
            header_cfg: HeaderWriteConfig::default(),
            msg: MessageWriter::new(account_uri.clone()),
            reg: RegistrationManager::new(account_uri, local_uri),
        }
    }

    /// Return a reference to the sip registration manager.
    pub fn registry(&self) -> &RegistrationManager {
        &self.reg
    }

    /// Return a mutable reference tp the sip registration manager.
    pub fn registry_mut(&mut self) -> &mut RegistrationManager {
        &mut self.reg
    }

    /// Return a reference to the message writer.
    pub fn messaging(&self) -> &MessageWriter {
        &self.msg
    }

    /// Return a mutable reference to the MessageWriter.
    pub fn messaging_mut(&mut self) -> &mut MessageWriter {
        &mut self.msg
    }

    pub fn header_cfg(&self) -> &HeaderWriteConfig {
        &self.header_cfg
    }

    pub fn header_cfg_mut(&mut self) -> &mut HeaderWriteConfig {
        &mut self.header_cfg
    }

    /// Simple pass through method to get a registration request.
    pub fn get_register_request(&mut self) -> IoResult<SipMessage> {
        Ok(self.reg.get_request(&self.header_cfg)?)
    }

    /// Set the received auth challenge request.
    pub fn set_register_challenge(&mut self, c: SipMessage) -> IoResult<()> {
        self.reg.set_challenge(c)?;
        Ok(())
    }

    /// Send a new Message to `uri`.
    pub fn write_message(&mut self, b: Vec<u8>, uri: Uri) -> IoResult<SipMessage> {
        Ok(self
            .msg
            .write_message(b, uri, self.reg.via_header(), &self.header_cfg)?)
    }

    pub fn cancel_response(&mut self, headers: &Headers) -> IoResult<(SipMessage, SipMessage)> {
        let mut out_headers = vec![];
        for header in headers.iter() {
            match header {
                Header::CSeq(a, b) => out_headers.push(Header::CSeq(*a, *b)),
                Header::CallId(call) => out_headers.push(Header::CallId(call.clone())),
                Header::From(from) => out_headers.push(Header::From(from.clone())),
                Header::To(to) => out_headers.push(Header::To(to.clone())),
                Header::Via(via) => out_headers.push(Header::Via(via.clone())),
                _ => {},
            }
        }

        Ok((
            ResponseGenerator::new()
                .code(200)
                .headers(out_headers.clone())
                .header(Header::ContentLength(0))
                .build()?,
            ResponseGenerator::new()
                .code(487)
                .headers(out_headers)
                .header(Header::ContentLength(0))
                .build()?,
        ))
    }
}
