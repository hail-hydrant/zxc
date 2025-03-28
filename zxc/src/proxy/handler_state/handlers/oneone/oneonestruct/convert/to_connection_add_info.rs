use super::*;
use crate::proxy::handler_state::additional_handler_info::AdditionalHandlerInfo;

/* OneOneStruct<Request> => Connection + AdditionalHandlerInfo
 *
 * Used:
 *      ProxyState::NewConnection
 */

impl<T, E> From<OneOneStruct<T, E, Request>>
    for (Connection<T, E>, AdditionalHandlerInfo)
where
    T: AsyncReadExt + AsyncWriteExt + Unpin,
    E: AsyncReadExt + AsyncWriteExt + Unpin,
{
    fn from(
        oneone: OneOneStruct<T, E, Request>,
    ) -> (Connection<T, E>, AdditionalHandlerInfo) {
        let addinfo = AdditionalHandlerInfo::new(
            oneone.log_id,
            oneone.payload.unwrap(),
            oneone.path.unwrap(),
            oneone.need_response,
            oneone.commander_recvr,
            oneone.server_info,
            oneone.history_sendr,
        );
        let conn = Connection {
            buf: oneone.buf,
            commander: oneone.commander_sendr,
            frame: oneone.frame,
            id: oneone.id,
            reader: oneone.reader,
            writer: oneone.writer,
        };
        (conn, addinfo)
    }
}
