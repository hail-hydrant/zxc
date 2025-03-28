pub mod error;
pub mod handlers;
pub mod read_write;
pub mod role;
pub mod transition;
use std::fmt::Display;

use bytes::BytesMut;
use role::GetRole;
use transition::can_communicate::CanCommunicate;
use transition::drop_msg::DropMsg;
use transition::frame_to_payload::FrameToPayload;
use transition::read_modified_file::add_raw::AddRaw;
use transition::read_modified_file::read_mod_file;
use transition::resume_intercept::update_resume_info::UpdateResumeInfo;
use transition::rewrite::Rewrite;
use transition::update_frame::bytes_to_frame::BytesToFrame;
use transition::update_frame::should_rewrite::ShouldRewrite;
use transition::update_frame::update_frame_state;
use transition::write_history::{GetHistory, SendHistory, write_history};
use transition::write_log::file_ops::FileOps;
use transition::write_log::log::Log;
use transition::write_log::update_log_extension::UpdateLogExt;
use transition::write_log::write_log;
pub mod additional_handler_info;

use self::error::ProxyStateError;
use self::read_write::ReadWrite;
use self::transition::intercept::*;
use self::transition::resume_intercept::resume_intercept;
use self::transition::rewrite::rewrite_log;
use self::transition::should_intercept::*;
use self::transition::should_log::*;
use super::server_info::json::ServerInfoJson;
use crate::async_step::AsyncStep;
use crate::commander::Protocol;
use crate::id::Id;
use crate::interceptor::message::from_ui::resume_info::ResumeInfo;

// All possible Handler States
// In order
pub enum ProxyState<T> {
    Receive(T),
    ShouldLog(T),
    WriteHistory(T),
    Log(T),
    ShouldIntercept(T),
    Intercept(T),
    ResumeIntercept(T),
    ReadModFile(T, ResumeInfo),
    UpdateFrame(T, BytesMut, ResumeInfo),
    ReWrite(T, ResumeInfo),
    NewConnection(T, ServerInfoJson), // http only
    Send(T),
    Drop(T),
    End(T),
    SwitchProtocol(T, Protocol),     // http only
    ServerClose(T, ProxyStateError), // http only
}

/* Description:
 *      AsyncStep trait implementation for Proxy State Transition.
 *      Each transition requires a trait.
 *      Generic type T implements required traits.
 *      Trait Bound is used to perform state transition.
 *      Each transition is mapped to their respective errors.
 *
 * Args:
 *      self
 *
 * Returns:
 *      Ok(Self)
 *
 * Error:
 *      ProxyStateError
 */

impl<T> AsyncStep for ProxyState<T>
where
    Self: Sized,
    T: ReadWrite
        + ReadWrite<State = Self>
        + CanCommunicate
        + GetRole
        + CanLog
        + ShouldLog
        + UpdateLogExt
        + Log
        + GetHistory
        + SendHistory
        + ShouldIntercept
        + QueryCommanderShouldIntercept
        + Intercept
        + BytesToFrame
        + ShouldRewrite
        + FrameToPayload
        + FileOps
        + UpdateResumeInfo
        + AddRaw
        + Id
        + Rewrite
        + DropMsg,
    <T as ReadWrite>::Error: Into<ProxyStateError>,
{
    type Error = ProxyStateError;

    async fn next(self) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match self {
            // 1. Read -> ShouldLog | ServerClose
            Self::Receive(conn) => conn.read().await.map_err(Into::into),
            // 2. ShouldLog -> WriteHistory | Send
            Self::ShouldLog(conn) => should_log(conn).await,
            // 3. WriteHistory -> Log
            Self::WriteHistory(conn) => write_history(conn).await,
            // 4. Log -> ShouldIntercept
            Self::Log(conn) => write_log(conn).await,
            // 5. ShouldIntercept -> Intercept | Send
            Self::ShouldIntercept(conn) => should_intercept(conn).await,
            // 6. Intercept -> ResumeIntercept | Drop
            Self::Intercept(conn) => intercept(conn).await,
            // 7. Drop -> End
            Self::Drop(conn) => {
                T::continue_on_drop()?;
                Ok(Self::Receive(conn))
            }
            // 7. ResumeIntercept -> ReadModFile | NewConnection | Send
            Self::ResumeIntercept(conn) => resume_intercept(conn).await,
            // 8. ReadModFile -> UpdateFrame | Send
            Self::ReadModFile(conn, resume_info) => {
                read_mod_file(conn, resume_info).await
            }
            // 9. UpdateFrame -> ReWrite | Send
            Self::UpdateFrame(conn, buf, resume_info) => {
                update_frame_state(conn, buf, resume_info)
            }
            // 10. ReWrite -> Send
            Self::ReWrite(conn, resume_info) => {
                rewrite_log(conn, resume_info).await
            }
            // 11. Send -> End | ReadFail | Read (full duplex)
            Self::Send(conn) => conn.write().await.map_err(Into::into),
            _ => Ok(self),
        }
    }

    // Checks if the state has ended
    fn is_ended(&self) -> bool {
        matches!(self, Self::End(_))
            || matches!(self, Self::ServerClose(_, _))
            || matches!(self, Self::NewConnection(_, _))
    }
}

impl<T> Display for ProxyState<T>
where
    T: GetRole,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (conn, val) = match self {
            Self::Receive(conn) => (conn, "receive"),
            Self::ShouldLog(conn) => (conn, "should_log"),
            Self::WriteHistory(conn) => (conn, "write_history"),
            Self::Log(conn) => (conn, "log"),
            Self::ShouldIntercept(conn) => (conn, "should_intercept"),
            Self::Intercept(conn) => (conn, "intercept"),
            Self::ResumeIntercept(conn) => (conn, "resume_intercept"),
            Self::ReadModFile(conn, _) => (conn, "read_mod_file"),
            Self::UpdateFrame(conn, ..) => (conn, "update_frame"),
            Self::ReWrite(conn, _) => (conn, "rewrite"),
            Self::Send(conn) => (conn, "send"),
            Self::SwitchProtocol(conn, _) => (conn, "switch_protocol"),
            _ => return Err(std::fmt::Error),
        };
        write!(f, "{}| {}", conn.role(), val)
    }
}
