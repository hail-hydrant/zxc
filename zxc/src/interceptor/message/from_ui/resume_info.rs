use serde::Deserialize;

use super::{FileType, FtSpecific};
use crate::proxy::server_info::json::ServerInfoJson;

/* Description:
 *      Info needed to resume a connection in pause state
 *
 * Format:
 *
 *      common:
 *          { id: 0, modified: false, ft: *}
 *
 *      request:
 *          { req: {
 *              need_response: Option<bool>,
 *              update: Option<bool>,
 *              server_info: {
 *                    host: String,
 *                    scheme: String,
 *                    sni: String } } }
 *
 *      response:
 *          { res: { update: bool } }
 *
 *      websocket request:
 *          { wreq: {
 *              need_response: Option<bool> } }
 *
 *      websocket response:
 *          { wres }
 */

#[derive(Debug, Deserialize)]
pub struct ResumeInfo {
    pub id: usize,
    modified: Option<bool>,
    ft: FtSpecific,
}

impl ResumeInfo {
    pub fn modified(&self) -> bool {
        self.modified.is_some()
    }

    // For filetypes req and wreq only, if b:need_response is set in ui, true
    // by default false
    pub fn need_response(&self) -> bool {
        match self.ft {
            FtSpecific::Req {
                need_response,
                ..
            }
            | FtSpecific::WReq {
                need_response,
            } => need_response.is_some(),
            _ => false,
        }
    }

    // For filetypes req and res only, if b:update is set in ui, false
    // by default true for ws
    pub fn update(&self) -> bool {
        match self.ft {
            FtSpecific::Req {
                update,
                ..
            }
            | FtSpecific::Res {
                update,
            } => update.is_none(),
            _ => true,
        }
    }

    /* When HandleOneOneError::ReadFromServer occurs, reconnect to server and
     * resume from ProxyState::ReadModFile state which requires a ResumeInfo
     *
     * As the frame is already updated and rewritten according to the user.
     *
     * By setting update to some, resume_info.update() returns false, so the
     * state can transition to Send directly
     */
    pub fn request() -> Self {
        Self {
            id: 0,
            modified: None,
            ft: FtSpecific::Req {
                server_info: None,
                need_response: None,
                update: Some(true),
            },
        }
    }

    pub fn file_type(&self) -> FileType {
        match self.ft {
            FtSpecific::Req {
                ..
            } => FileType::Req,
            FtSpecific::Res {
                ..
            } => FileType::Res,
            FtSpecific::WReq {
                ..
            } => FileType::Wreq,
            FtSpecific::WRes => FileType::Wres,
        }
    }

    pub fn is_wreq(&self) -> bool {
        matches!(self.ft, FtSpecific::WReq { .. })
    }

    // for ft req only
    pub fn into_server_info(mut self) -> Option<ServerInfoJson> {
        if let FtSpecific::Req {
            server_info,
            ..
        } = &mut self.ft
        {
            server_info.take()
        } else {
            None
        }
    }
}
